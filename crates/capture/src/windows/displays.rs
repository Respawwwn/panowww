//! The connected displays: a stable id, the size in physical pixels, the
//! scale and whether HDR is on. `HMONITOR` changes with every display
//! change, so the id is the monitor's device interface path (vendor, model
//! and port), with the position as a tie-breaker for identical paths.
//!
//! Sizes are physical only in a per-monitor DPI aware process: the app
//! (tao) declares that itself, the example calls [`set_process_dpi_aware`].

use std::collections::HashSet;

use panowww_core::ports::{CaptureError, DisplayId, DisplayInfo};
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Dxgi::Common::DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput6};
use windows::Win32::Graphics::Gdi::{
    DISPLAY_DEVICEW, EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR,
    MONITORINFO, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor, MDT_EFFECTIVE_DPI,
    SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EDD_GET_DEVICE_INTERFACE_NAME, MONITORINFOF_PRIMARY,
};
use windows::core::{BOOL, Interface, PCWSTR};

/// One connected display, with what the capture needs to find it.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayEntry {
    pub info: DisplayInfo,
    /// The monitor handle, valid until the display configuration changes.
    pub hmonitor: isize,
    /// GDI device name, `\\.\DISPLAY1` and so on.
    pub device_name: String,
    /// Physical pixels of the virtual screen, origin top-left of the
    /// primary display.
    pub left: i32,
    pub top: i32,
}

/// Makes this process per-monitor DPI aware (v2), so monitor rectangles
/// come in physical pixels. The app does this through tao; examples and
/// tests call it first. Fails harmlessly when the awareness is already set.
pub fn set_process_dpi_aware() {
    // SAFETY: takes a constant context handle; the only failure is "already
    // set", which is fine.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

/// Every active display, primary first.
pub fn list_displays() -> Result<Vec<DisplayEntry>, CaptureError> {
    let mut handles: Vec<isize> = Vec::new();
    // SAFETY: the callback only pushes into the Vec passed as LPARAM, which
    // outlives the call.
    let ok = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(collect_monitor),
            LPARAM((&raw mut handles) as isize),
        )
    };
    if !ok.as_bool() {
        return Err(CaptureError::Unavailable(
            "EnumDisplayMonitors failed".to_owned(),
        ));
    }
    let hdr = hdr_monitors();
    let mut entries: Vec<DisplayEntry> = Vec::new();
    let mut primary_index = None;
    for hmonitor in handles {
        let Some((rect, device_name, primary)) = monitor_info(hmonitor) else {
            continue;
        };
        let base = device_path(&device_name).unwrap_or_else(|| device_name.clone());
        let id = if entries.iter().any(|e| e.info.id.0 == base) {
            format!("{base}@{},{}", rect.left, rect.top)
        } else {
            base
        };
        if primary {
            primary_index = Some(entries.len());
        }
        entries.push(DisplayEntry {
            info: DisplayInfo {
                id: DisplayId(id),
                width: side(rect.right - rect.left),
                height: side(rect.bottom - rect.top),
                scale: scale_of(hmonitor),
                hdr: hdr.contains(&hmonitor),
            },
            hmonitor,
            device_name,
            left: rect.left,
            top: rect.top,
        });
    }
    if let Some(index) = primary_index.filter(|i| *i != 0) {
        let primary = entries.remove(index);
        entries.insert(0, primary);
    }
    Ok(entries)
}

/// The display with this id, if connected.
pub fn find_display(id: &DisplayId) -> Result<DisplayEntry, CaptureError> {
    list_displays()?
        .into_iter()
        .find(|e| e.info.id == *id)
        .ok_or(CaptureError::DisplayGone)
}

unsafe extern "system" fn collect_monitor(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    // SAFETY: `lparam` is the `&mut Vec<isize>` passed by `list_displays`,
    // alive for the whole enumeration.
    let handles = unsafe { &mut *(lparam.0 as *mut Vec<isize>) };
    handles.push(hmonitor.0 as isize);
    BOOL(1)
}

/// The monitor rectangle, GDI device name and primary flag.
fn monitor_info(hmonitor: isize) -> Option<(RECT, String, bool)> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = u32::try_from(std::mem::size_of::<MONITORINFOEXW>()).ok()?;
    // SAFETY: a MONITORINFOEXW starts with a MONITORINFO whose cbSize says
    // which of the two structures the buffer is.
    let ok = unsafe {
        GetMonitorInfoW(
            HMONITOR(hmonitor as *mut _),
            (&raw mut info).cast::<MONITORINFO>(),
        )
    };
    if !ok.as_bool() {
        return None;
    }
    let name = wide_to_string(&info.szDevice);
    let primary = info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0;
    Some((info.monitorInfo.rcMonitor, name, primary))
}

/// The device interface path of the first monitor attached to a GDI
/// device: stable across reboots for the same monitor on the same port.
fn device_path(device_name: &str) -> Option<String> {
    let wide: Vec<u16> = device_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut device = DISPLAY_DEVICEW {
        cb: u32::try_from(std::mem::size_of::<DISPLAY_DEVICEW>()).ok()?,
        ..Default::default()
    };
    // SAFETY: `wide` is NUL-terminated and `device.cb` carries its size.
    let ok = unsafe {
        EnumDisplayDevicesW(
            PCWSTR(wide.as_ptr()),
            0,
            &raw mut device,
            EDD_GET_DEVICE_INTERFACE_NAME,
        )
    };
    if !ok.as_bool() {
        return None;
    }
    let path = wide_to_string(&device.DeviceID);
    (!path.is_empty()).then_some(path)
}

/// Effective DPI over 96, 1.0 when unknown.
fn scale_of(hmonitor: isize) -> f64 {
    let (mut x, mut y) = (0u32, 0u32);
    // SAFETY: two out-parameters for the DPI of a valid monitor handle.
    let ok = unsafe {
        GetDpiForMonitor(
            HMONITOR(hmonitor as *mut _),
            MDT_EFFECTIVE_DPI,
            &raw mut x,
            &raw mut y,
        )
    };
    if ok.is_ok() && x > 0 {
        f64::from(x) / 96.0
    } else {
        1.0
    }
}

/// The monitors whose current colour space is HDR10 (`G2084`, BT.2020).
fn hdr_monitors() -> HashSet<isize> {
    let mut set = HashSet::new();
    // SAFETY: plain COM factory creation.
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        return set;
    };
    for adapter_index in 0.. {
        // SAFETY: enumeration ends with DXGI_ERROR_NOT_FOUND.
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(adapter_index) }) else {
            break;
        };
        for output_index in 0.. {
            // SAFETY: same contract as above.
            let Ok(output) = (unsafe { adapter.EnumOutputs(output_index) }) else {
                break;
            };
            let Ok(output6) = output.cast::<IDXGIOutput6>() else {
                continue;
            };
            // SAFETY: plain accessor of a live output.
            if let Ok(desc) = unsafe { output6.GetDesc1() }
                && desc.ColorSpace == DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020
            {
                set.insert(desc.Monitor.0 as isize);
            }
        }
    }
    set
}

fn wide_to_string(wide: &[u16]) -> String {
    let end = wide.iter().position(|c| *c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

fn side(pixels: i32) -> u32 {
    u32::try_from(pixels).unwrap_or(0).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_strings_stop_at_the_first_nul() {
        let wide: Vec<u16> = "\\\\.\\DISPLAY1\0garbage".encode_utf16().collect();
        assert_eq!(wide_to_string(&wide), "\\\\.\\DISPLAY1");
        assert_eq!(wide_to_string(&[]), "");
    }

    #[test]
    fn sides_are_positive() {
        assert_eq!(side(1920), 1920);
        assert_eq!(side(0), 1);
        assert_eq!(side(-5), 1);
    }

    #[test]
    fn the_machine_has_at_least_one_display_with_a_stable_looking_id() {
        set_process_dpi_aware();
        let displays = list_displays().unwrap();
        assert_ne!(displays.len(), 0);
        let first = &displays[0];
        assert!(first.info.width > 0 && first.info.height > 0);
        assert!(first.info.scale >= 1.0);
        assert_ne!(first.info.id.0, "");
        assert_eq!(
            find_display(&first.info.id).unwrap().hmonitor,
            first.hmonitor
        );
    }
}
