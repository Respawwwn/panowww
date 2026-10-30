//! PNG encoding of Shots (spec §9): `png` with `Compression::Fast`, RGB8,
//! no alpha, on one worker thread fed through a bounded queue so the
//! Session thread never encodes and never drops a Shot.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use panowww_core::ports::Frame;

use crate::StoreError;

/// Shots waiting for the worker before `submit` blocks the Session thread.
pub const DEFAULT_QUEUE_CAPACITY: usize = 4;

/// `frame` as PNG bytes.
pub fn encode(frame: &Frame) -> Result<Vec<u8>, StoreError> {
    if !frame.is_well_formed() {
        return Err(StoreError::Png {
            path: PathBuf::new(),
            message: format!(
                "frame of {}×{} has {} bytes",
                frame.width,
                frame.height,
                frame.rgb.len()
            ),
        });
    }
    let mut bytes = Vec::with_capacity(frame.rgb.len() / 2);
    let mut encoder = png::Encoder::new(&mut bytes, frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().map_err(|e| png_error(&e))?;
    writer
        .write_image_data(&frame.rgb)
        .map_err(|e| png_error(&e))?;
    writer.finish().map_err(|e| png_error(&e))?;
    Ok(bytes)
}

/// Encodes `frame` and writes it to `path`, whole or not at all.
pub fn write(path: &Path, frame: &Frame) -> Result<(), StoreError> {
    let bytes = encode(frame).map_err(|e| match e {
        StoreError::Png { message, .. } => StoreError::Png {
            path: path.to_owned(),
            message,
        },
        other => other,
    })?;
    let temp = path.with_extension("png.tmp");
    std::fs::write(&temp, bytes).map_err(|e| StoreError::io("write", &temp, e))?;
    std::fs::rename(&temp, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        StoreError::io("rename", path, e)
    })
}

/// Reads a PNG written by [`write()`] (or any 8-bit RGB / RGBA PNG) back
/// into a frame.
pub fn read(path: &Path) -> Result<Frame, StoreError> {
    let file = std::fs::File::open(path).map_err(|e| StoreError::io("read", path, e))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().map_err(|e| png_at(path, &e))?;
    let mut buffer = vec![0; reader.output_buffer_size().unwrap_or_default()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|e| png_at(path, &e))?;
    buffer.truncate(info.buffer_size());
    let rgb = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgb, png::BitDepth::Eight) => buffer,
        (png::ColorType::Rgba, png::BitDepth::Eight) => buffer
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect(),
        (color, depth) => {
            return Err(StoreError::Png {
                path: path.to_owned(),
                message: format!("unsupported PNG layout {color:?} {depth:?}"),
            });
        }
    };
    Ok(Frame {
        width: info.width,
        height: info.height,
        rgb,
    })
}

fn png_error(error: &png::EncodingError) -> StoreError {
    StoreError::Png {
        path: PathBuf::new(),
        message: error.to_string(),
    }
}

fn png_at(path: &Path, error: &png::DecodingError) -> StoreError {
    StoreError::Png {
        path: path.to_owned(),
        message: error.to_string(),
    }
}

/// One Shot to write.
#[derive(Debug)]
pub struct PngJob {
    pub path: PathBuf,
    pub frame: Frame,
}

/// The worker thread and its bounded queue. [`PngWorker::submit`] blocks
/// while the queue is full (back-pressure), never drops, and reports the
/// first failed write so the Session stops instead of leaving a hole.
#[derive(Debug)]
pub struct PngWorker {
    sender: Option<SyncSender<PngJob>>,
    handle: Option<JoinHandle<()>>,
    failure: Arc<Mutex<Option<String>>>,
}

impl PngWorker {
    /// Starts the thread with room for `capacity` queued Shots.
    ///
    /// # Panics
    ///
    /// When the OS refuses to spawn a thread.
    pub fn start(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::sync_channel::<PngJob>(capacity.max(1));
        let failure = Arc::new(Mutex::new(None));
        let failed = Arc::clone(&failure);
        let handle = std::thread::Builder::new()
            .name("panowww-png".to_owned())
            .spawn(move || {
                for job in receiver {
                    if let Err(error) = write(&job.path, &job.frame) {
                        tracing::error!(%error, "PNG write failed");
                        *failed
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) =
                            Some(error.to_string());
                        // Stop here: the Session must not continue with a hole.
                        break;
                    }
                }
            })
            .expect("spawning the PNG worker thread");
        Self {
            sender: Some(sender),
            handle: Some(handle),
            failure,
        }
    }

    /// Queues a Shot, waiting while the queue is full. Fails once a
    /// previous write failed.
    pub fn submit(&self, job: PngJob) -> Result<(), StoreError> {
        if let Some(message) = self.failure_message() {
            return Err(StoreError::Png {
                path: job.path,
                message,
            });
        }
        let path = job.path.clone();
        match &self.sender {
            Some(sender) => sender.send(job).map_err(|_| StoreError::Png {
                path,
                message: self
                    .failure_message()
                    .unwrap_or_else(|| "the PNG worker stopped".to_owned()),
            }),
            None => Err(StoreError::Png {
                path,
                message: "the PNG worker is finished".to_owned(),
            }),
        }
    }

    /// Waits for every queued Shot to be on disk. Returns the first failure.
    pub fn finish(mut self) -> Result<(), StoreError> {
        self.sender.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        match self.failure_message() {
            Some(message) => Err(StoreError::Png {
                path: PathBuf::new(),
                message,
            }),
            None => Ok(()),
        }
    }

    fn failure_message(&self) -> Option<String> {
        self.failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_read_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("r01_c01.png");
        let frame = Frame {
            width: 4,
            height: 2,
            rgb: (0..24).collect(),
        };
        write(&path, &frame).unwrap();
        assert_eq!(&std::fs::read(&path).unwrap()[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(read(&path).unwrap(), frame);
        assert_eq!(
            std::fs::read_dir(dir.path()).unwrap().count(),
            1,
            "no temp file left"
        );
    }

    #[test]
    fn a_malformed_frame_is_refused_before_touching_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.png");
        let frame = Frame {
            width: 4,
            height: 2,
            rgb: vec![0; 5],
        };
        assert!(matches!(write(&path, &frame), Err(StoreError::Png { .. })));
        assert!(!path.exists());
    }

    #[test]
    fn the_queue_back_pressures_without_dropping() {
        let dir = tempfile::tempdir().unwrap();
        let worker = PngWorker::start(2);
        let jobs = 30;
        for i in 0..jobs {
            worker
                .submit(PngJob {
                    path: dir.path().join(format!("{i:02}.png")),
                    frame: Frame::solid(320, 180, [i, 0, 0]),
                })
                .unwrap();
        }
        worker.finish().unwrap();
        let written = std::fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(written, usize::from(jobs));
        assert_eq!(read(&dir.path().join("29.png")).unwrap().rgb[0], 29);
    }

    #[test]
    fn a_failed_write_stops_the_worker_and_surfaces_on_submit_and_finish() {
        let dir = tempfile::tempdir().unwrap();
        let worker = PngWorker::start(1);
        worker
            .submit(PngJob {
                path: dir.path().join("missing-folder").join("a.png"),
                frame: Frame::solid(2, 2, [0; 3]),
            })
            .unwrap();
        // Give the worker time to fail; later submits must then be refused.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut refused = false;
        while std::time::Instant::now() < deadline {
            let result = worker.submit(PngJob {
                path: dir.path().join("b.png"),
                frame: Frame::solid(2, 2, [0; 3]),
            });
            if result.is_err() {
                refused = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(refused, "submit keeps succeeding after a failure");
        assert!(matches!(worker.finish(), Err(StoreError::Png { .. })));
    }
}
