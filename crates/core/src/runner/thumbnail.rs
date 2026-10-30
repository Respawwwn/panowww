//! The live thumbnail sent with every `Progress::Shot`: the Shot reduced to a
//! small JPEG in memory, never written to the Session folder (spec §9).

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{PixelType, Resizer};
use image::ExtendedColorType;
use image::codecs::jpeg::JpegEncoder;

use crate::constants::{THUMBNAIL_JPEG_QUALITY, THUMBNAIL_LONG_SIDE};
use crate::ports::{Frame, Jpeg};

/// `frame` reduced to [`THUMBNAIL_LONG_SIDE`] on its longer side and encoded
/// as JPEG. `None` when the frame is malformed or empty; a missing thumbnail
/// never fails a run.
pub fn thumbnail(frame: &Frame) -> Option<Jpeg> {
    if !frame.is_well_formed() || frame.width == 0 || frame.height == 0 {
        return None;
    }
    let (width, height) = fit(frame.width, frame.height, THUMBNAIL_LONG_SIDE);
    let source = ImageRef::new(frame.width, frame.height, &frame.rgb, PixelType::U8x3).ok()?;
    let mut reduced = Image::new(width, height, PixelType::U8x3);
    Resizer::new().resize(&source, &mut reduced, None).ok()?;
    encode(reduced.buffer(), width, height)
}

fn encode(rgb: &[u8], width: u32, height: u32) -> Option<Jpeg> {
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, THUMBNAIL_JPEG_QUALITY)
        .encode(rgb, width, height, ExtendedColorType::Rgb8)
        .ok()?;
    Some(Jpeg(bytes))
}

/// `width × height` scaled so the longer side is at most `long_side`, never
/// enlarged, each side at least 1.
pub(crate) fn fit(width: u32, height: u32, long_side: u32) -> (u32, u32) {
    let longest = width.max(height);
    if longest <= long_side {
        return (width, height);
    }
    let scale = |side: u32| {
        let scaled = u64::from(side) * u64::from(long_side) / u64::from(longest);
        u32::try_from(scaled).unwrap_or(long_side).max(1)
    };
    (scale(width), scale(height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_the_ratio_and_never_enlarges() {
        assert_eq!(fit(1920, 1080, 192), (192, 108));
        assert_eq!(fit(1080, 1920, 192), (108, 192));
        assert_eq!(fit(100, 50, 192), (100, 50));
        assert_eq!(fit(192, 192, 192), (192, 192));
        assert_eq!(fit(10_000, 1, 192), (192, 1));
    }

    #[test]
    fn thumbnail_is_a_small_jpeg() {
        let frame = Frame::solid(1920, 1080, [200, 30, 30]);
        let jpeg = thumbnail(&frame).unwrap();
        assert_eq!(&jpeg.0[..2], &[0xFF, 0xD8], "JPEG magic");
        assert!(jpeg.0.len() < 10_000, "{} bytes", jpeg.0.len());
        let decoded = image::load_from_memory(&jpeg.0).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (192, 108));
    }

    #[test]
    fn small_frames_are_encoded_as_is() {
        let frame = Frame::solid(40, 30, [0, 0, 0]);
        let decoded = image::load_from_memory(&thumbnail(&frame).unwrap().0).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (40, 30));
    }

    #[test]
    fn malformed_and_empty_frames_give_no_thumbnail() {
        assert_eq!(thumbnail(&Frame::solid(0, 10, [0; 3])), None);
        assert_eq!(
            thumbnail(&Frame {
                width: 4,
                height: 4,
                rgb: vec![0; 10]
            }),
            None
        );
    }
}
