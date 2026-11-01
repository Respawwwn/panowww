//! What the app needs from the operating system, behind one trait so the
//! Runner owner is tested with fakes: the input and capture adapters, the
//! permission checks, the displays.

use std::sync::Arc;

use panowww_core::ports::{Camera, CaptureError, DisplayId, DisplayInfo, Grabber};

use crate::dto::PermissionKind;

pub trait Backend: Send + Sync {
    /// A fresh `Camera`, made on the thread that will use it.
    fn camera(&self) -> Box<dyn Camera>;
    /// A fresh `Grabber`, made on the thread that will use it.
    fn grabber(&self) -> Box<dyn Grabber>;
    fn accessibility_granted(&self) -> bool;
    fn screen_recording_granted(&self) -> bool;
    /// Whether gamepad Games can run here (ViGEmBus present on Windows).
    fn gamepad_available(&self) -> bool;
    fn displays(&self) -> Result<Vec<DisplayInfo>, CaptureError>;
    /// Where a display sits in the global point space (top-left of the main
    /// display at the origin), for the overlay window. `None` when gone.
    fn display_origin(&self, id: &DisplayId) -> Option<(f64, f64)>;
    /// Asks the OS; returns whether the permission is granted now.
    fn request_permission(&self, permission: PermissionKind) -> bool;
    fn open_permission_settings(&self, permission: PermissionKind) -> std::io::Result<()>;
}

pub type SharedBackend = Arc<dyn Backend>;

/// The real operating system.
#[derive(Debug, Default)]
pub struct SystemBackend;

#[cfg(target_os = "macos")]
impl Backend for SystemBackend {
    fn camera(&self) -> Box<dyn Camera> {
        Box::new(panowww_input::macos::CgCamera::new())
    }

    fn grabber(&self) -> Box<dyn Grabber> {
        Box::new(panowww_capture::macos::SckGrabber)
    }

    fn accessibility_granted(&self) -> bool {
        panowww_input::macos::accessibility::check()
    }

    fn screen_recording_granted(&self) -> bool {
        panowww_capture::macos::permission::check()
    }

    fn gamepad_available(&self) -> bool {
        false
    }

    fn displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        Ok(panowww_capture::macos::list_displays()?
            .into_iter()
            .map(|d| d.info)
            .collect())
    }

    fn display_origin(&self, id: &DisplayId) -> Option<(f64, f64)> {
        panowww_capture::macos::displays::find_display(id)
            .ok()
            .map(|entry| (entry.bounds.origin.x, entry.bounds.origin.y))
    }

    fn request_permission(&self, permission: PermissionKind) -> bool {
        match permission {
            PermissionKind::Accessibility => panowww_input::macos::accessibility::request(),
            PermissionKind::ScreenRecording => panowww_capture::macos::permission::request(),
        }
    }

    fn open_permission_settings(&self, permission: PermissionKind) -> std::io::Result<()> {
        match permission {
            PermissionKind::Accessibility => panowww_input::macos::accessibility::open_settings(),
            PermissionKind::ScreenRecording => panowww_capture::macos::permission::open_settings(),
        }
    }
}

/// Windows: the `SendInput` Camera and the virtual pad (issue 23); capture
/// arrives with issue 24, so every run still stops at the Stream. Windows
/// has no input or capture permission to grant: UIPI is handled by running
/// elevated (spec §8.4), hence `true` for both.
#[cfg(target_os = "windows")]
impl Backend for SystemBackend {
    fn camera(&self) -> Box<dyn Camera> {
        Box::new(panowww_input::windows::SendInputCamera::new())
    }

    fn grabber(&self) -> Box<dyn Grabber> {
        Box::new(Unsupported)
    }

    fn accessibility_granted(&self) -> bool {
        true
    }

    fn screen_recording_granted(&self) -> bool {
        true
    }

    fn gamepad_available(&self) -> bool {
        panowww_input::windows::gamepad::check().is_ok()
    }

    fn displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        Ok(Vec::new())
    }

    fn display_origin(&self, _id: &DisplayId) -> Option<(f64, f64)> {
        None
    }

    fn request_permission(&self, _permission: PermissionKind) -> bool {
        true
    }

    fn open_permission_settings(&self, _permission: PermissionKind) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows has no permission to grant",
        ))
    }
}

/// Other operating systems: the workspace builds, every run is refused.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl Backend for SystemBackend {
    fn camera(&self) -> Box<dyn Camera> {
        Box::new(Unsupported)
    }

    fn grabber(&self) -> Box<dyn Grabber> {
        Box::new(Unsupported)
    }

    fn accessibility_granted(&self) -> bool {
        false
    }

    fn screen_recording_granted(&self) -> bool {
        false
    }

    fn gamepad_available(&self) -> bool {
        false
    }

    fn displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        Ok(Vec::new())
    }

    fn display_origin(&self, _id: &DisplayId) -> Option<(f64, f64)> {
        None
    }

    fn request_permission(&self, _permission: PermissionKind) -> bool {
        false
    }

    fn open_permission_settings(&self, _permission: PermissionKind) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "no settings on this OS",
        ))
    }
}

#[cfg(not(target_os = "macos"))]
struct Unsupported;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl Camera for Unsupported {
    fn key_down(
        &mut self,
        _: panowww_core::game::PhysicalKey,
    ) -> Result<(), panowww_core::ports::InputError> {
        Err(unsupported_input())
    }
    fn key_up(
        &mut self,
        _: panowww_core::game::PhysicalKey,
    ) -> Result<(), panowww_core::ports::InputError> {
        Err(unsupported_input())
    }
    fn mouse_delta(&mut self, _: i32, _: i32) -> Result<(), panowww_core::ports::InputError> {
        Err(unsupported_input())
    }
    fn button_down(
        &mut self,
        _: panowww_core::ports::MouseButton,
    ) -> Result<(), panowww_core::ports::InputError> {
        Err(unsupported_input())
    }
    fn button_up(
        &mut self,
        _: panowww_core::ports::MouseButton,
    ) -> Result<(), panowww_core::ports::InputError> {
        Err(unsupported_input())
    }
    fn park_cursor(
        &mut self,
        _: panowww_core::ports::Point,
    ) -> Result<(), panowww_core::ports::InputError> {
        Err(unsupported_input())
    }
    fn stick(&mut self, _: f64, _: f64) -> Result<(), panowww_core::ports::InputError> {
        Err(unsupported_input())
    }
    fn release_all(&mut self) -> Result<(), panowww_core::ports::InputError> {
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
impl Grabber for Unsupported {
    fn start(
        &mut self,
        _: panowww_core::ports::DisplayId,
        _: panowww_core::ports::Rect,
        _: panowww_core::ports::PreviewSink,
    ) -> Result<Box<dyn panowww_core::ports::Stream>, CaptureError> {
        Err(CaptureError::Unavailable(
            "not available on this OS yet".to_owned(),
        ))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn unsupported_input() -> panowww_core::ports::InputError {
    panowww_core::ports::InputError::Failed("not available on this OS yet".to_owned())
}

/// A backend for tests: fakes from `panowww_core::fakes`, permissions and
/// displays set by the test.
#[cfg(test)]
pub mod fake {
    use std::sync::{Arc, Mutex};

    use panowww_core::fakes::{FakeStream, RecordingCamera};
    use panowww_core::ports::{
        CaptureError, DisplayId, DisplayInfo, Grabber, PreviewSink, Rect, Stream,
    };

    use super::{Backend, Camera};
    use crate::dto::PermissionKind;

    #[derive(Debug)]
    pub struct FakeBackend {
        pub accessibility: Mutex<bool>,
        pub screen_recording: Mutex<bool>,
        pub displays: Mutex<Vec<DisplayInfo>>,
        /// Fail `Grabber::start` with this error.
        pub capture_failure: Mutex<Option<CaptureError>>,
    }

    impl FakeBackend {
        pub fn new() -> Arc<Self> {
            Arc::new(Self {
                accessibility: Mutex::new(true),
                screen_recording: Mutex::new(true),
                displays: Mutex::new(vec![DisplayInfo {
                    id: DisplayId("main".to_owned()),
                    width: 2560,
                    height: 1440,
                    scale: 2.0,
                    hdr: false,
                }]),
                capture_failure: Mutex::new(None),
            })
        }
    }

    struct FakeGrabber {
        failure: Option<CaptureError>,
    }

    impl Grabber for FakeGrabber {
        fn start(
            &mut self,
            _: DisplayId,
            _: Rect,
            preview: PreviewSink,
        ) -> Result<Box<dyn Stream>, CaptureError> {
            if let Some(error) = self.failure.take() {
                return Err(error);
            }
            // One preview frame so the UI path is exercised.
            preview(panowww_core::ports::PreviewFrame {
                jpeg: panowww_core::ports::Jpeg(vec![0xFF, 0xD8, 0xFF, 0xD9]),
                display_w: 2560,
                display_h: 1440,
                zone: Rect {
                    x: 0,
                    y: 0,
                    w: 10,
                    h: 10,
                },
            });
            Ok(Box::new(FakeStream::new(16, 9).moving()))
        }
    }

    impl Backend for FakeBackend {
        fn camera(&self) -> Box<dyn Camera> {
            Box::new(RecordingCamera::new())
        }

        fn grabber(&self) -> Box<dyn Grabber> {
            Box::new(FakeGrabber {
                failure: self.capture_failure.lock().unwrap().take(),
            })
        }

        fn accessibility_granted(&self) -> bool {
            *self.accessibility.lock().unwrap()
        }

        fn screen_recording_granted(&self) -> bool {
            *self.screen_recording.lock().unwrap()
        }

        fn gamepad_available(&self) -> bool {
            false
        }

        fn displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
            Ok(self.displays.lock().unwrap().clone())
        }

        fn display_origin(&self, id: &DisplayId) -> Option<(f64, f64)> {
            self.displays
                .lock()
                .unwrap()
                .iter()
                .any(|d| d.id == *id)
                .then_some((0.0, 0.0))
        }

        fn request_permission(&self, _: PermissionKind) -> bool {
            true
        }

        fn open_permission_settings(&self, _: PermissionKind) -> std::io::Result<()> {
            Ok(())
        }
    }
}
