//! The Panorama on disk (spec §16.4): the Shots of a Session folder served
//! to the Stitch as a [`ShotSource`], then `panorama.jpg` with its `GPano` XMP
//! packet, `panorama-preview.jpg` for the viewer, and the `panorama` block
//! of `session.json`. Store errors (folder gone, disk full) stay
//! [`StoreError`]s, distinct from the Stitch's own.

use std::path::{Path, PathBuf};
use std::time::Instant;

use img_parts::jpeg::markers;
use img_parts::jpeg::{Jpeg, JpegSegment};
use panowww_core::ports::Frame;
use panowww_stitch::{ShotRef, ShotSource, StitchOptions, Stitched, UnalignedShots};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::StoreError;
use crate::png;
use crate::session::{SessionRecord, ShotEntry, TIMESTAMP, shot_file_name};
use crate::sink::WallClock;

/// File name of the Panorama inside the Session folder.
pub const FILE: &str = "panorama.jpg";
/// File name of the preview copy, at most 8192 px wide.
pub const PREVIEW_FILE: &str = "panorama-preview.jpg";
/// JPEG quality of the preview copy.
pub const PREVIEW_JPEG_QUALITY: u8 = 90;

/// Namespace that opens an XMP APP1 segment (XMP Part 3 §1.1.3), null
/// terminated.
const XMP_NAMESPACE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

/// The `panorama` block of `session.json`: the files, their size, when
/// and by which build they were stitched, what the Stitch measured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PanoramaEntry {
    /// [`FILE`]
    pub file: String,
    /// [`PREVIEW_FILE`]
    pub preview: String,
    pub width: u32,
    pub height: u32,
    /// Size of `file` on disk. A JSON number: serde writes `u64` as one.
    #[ts(type = "number")]
    pub bytes: u64,
    pub stitched_at: String,
    /// The whole Stitch, files written.
    pub duration_ms: u32,
    pub app_version: String,
    pub field_of_view_deg: f64,
    pub turn_step_deg: f64,
    pub pitch_step_deg: f64,
    pub start_pole_offset_deg: f64,
    pub closing_turn_deg: f64,
    pub drift_deg: f64,
    /// Shots placed from the grid for want of control points, by file name.
    pub unaligned: Vec<String>,
    /// `[twin, duplicate]` Shots of a Step the game swallowed, by file name.
    pub dropped_steps: Vec<[String; 2]>,
    pub anti_ghost: bool,
    /// What became of the `unaligned` Shots. Absent before the options:
    /// placed from the grid.
    #[serde(default)]
    pub unaligned_shots: UnalignedShots,
    /// The JPEG quality `file` was written at. Absent before the options: 100.
    #[serde(default = "full_quality")]
    pub jpeg_quality: u8,
}

fn full_quality() -> u8 {
    StitchOptions::MAX_QUALITY
}

/// The Shots of one Session folder, decoded on demand from their PNG.
#[derive(Debug, Clone)]
pub struct FolderShotSource {
    folder: PathBuf,
    shots: Vec<ShotEntry>,
}

impl FolderShotSource {
    /// The Shots `session.json` lists, Row by Row then column by column.
    ///
    /// # Errors
    ///
    /// The store's: the folder or its `session.json` cannot be read.
    pub fn open(folder: &Path) -> Result<Self, StoreError> {
        let record = SessionRecord::read(folder)?;
        Ok(Self::from_record(folder, &record))
    }

    /// Same, from a record already read.
    pub fn from_record(folder: &Path, record: &SessionRecord) -> Self {
        let mut shots = record.shots.clone();
        shots.sort_by_key(|s| (s.row, s.col));
        Self {
            folder: folder.to_owned(),
            shots,
        }
    }
}

impl ShotSource for FolderShotSource {
    fn count(&self) -> usize {
        self.shots.len()
    }

    fn position(&self, index: usize) -> (u32, u32) {
        let shot = &self.shots[index];
        (shot.row, shot.col)
    }

    fn shot(&self, index: usize) -> Result<Frame, panowww_core::ports::StoreError> {
        Ok(png::read(&self.folder.join(&self.shots[index].file))?)
    }
}

/// Writes `panorama.jpg`, `panorama-preview.jpg` and the `panorama` block
/// into the Session folder, each file whole or not at all, replacing the
/// files and the block of an earlier Stitch. `started` is when the Stitch
/// began: the entry's duration runs from there to the last byte written.
///
/// # Errors
///
/// The store's: `session.json` unreadable, a file that cannot be encoded,
/// written or renamed.
pub fn write_panorama(
    folder: &Path,
    stitched: &Stitched,
    jpeg_quality: u8,
    app_version: &str,
    started: Instant,
    clock: &dyn WallClock,
) -> Result<PanoramaEntry, StoreError> {
    let mut record = SessionRecord::read(folder)?;
    let report = &stitched.report;
    let path = folder.join(FILE);
    let jpeg_quality = jpeg_quality.clamp(StitchOptions::MIN_QUALITY, StitchOptions::MAX_QUALITY);
    let jpeg = encode_jpeg(&path, &stitched.panorama, jpeg_quality)?;
    let jpeg = with_xmp(
        &path,
        jpeg,
        &gpano_packet(&record, report, jpeg_quality, app_version),
    )?;
    write_atomic(&path, &jpeg)?;
    let preview_path = folder.join(PREVIEW_FILE);
    let preview = encode_jpeg(&preview_path, &stitched.preview, PREVIEW_JPEG_QUALITY)?;
    write_atomic(&preview_path, &preview)?;
    let file_name = |r: &ShotRef| shot_file_name(r.row, r.col);
    let entry = PanoramaEntry {
        file: FILE.to_owned(),
        preview: PREVIEW_FILE.to_owned(),
        width: report.width,
        height: report.height,
        bytes: u64::try_from(jpeg.len()).unwrap_or(u64::MAX),
        stitched_at: clock.now().strftime(TIMESTAMP).to_string(),
        duration_ms: u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX),
        app_version: app_version.to_owned(),
        field_of_view_deg: report.field_of_view_deg,
        turn_step_deg: report.turn_step_deg,
        pitch_step_deg: report.pitch_step_deg,
        start_pole_offset_deg: report.start_pole_offset_deg,
        closing_turn_deg: report.closing_turn_deg,
        drift_deg: report.drift_deg,
        unaligned: report.unaligned.iter().map(file_name).collect(),
        dropped_steps: report
            .dropped_steps
            .iter()
            .map(|[twin, duplicate]| [file_name(twin), file_name(duplicate)])
            .collect(),
        anti_ghost: report.anti_ghost,
        unaligned_shots: report.unaligned_shots,
        jpeg_quality,
    };
    record.version = crate::session::CURRENT_VERSION;
    record.panorama = Some(entry.clone());
    record.write(folder)?;
    Ok(entry)
}

/// The record's Panorama block when its file is still in the folder.
pub fn existing(folder: &Path, record: &SessionRecord) -> Option<PanoramaEntry> {
    record
        .panorama
        .clone()
        .filter(|entry| folder.join(&entry.file).is_file())
}

/// Whether a Stitch would replace an earlier Panorama: *Stitch again* asks
/// only then.
pub fn exists(folder: &Path) -> bool {
    folder.join(FILE).is_file()
}

/// `frame` as JPEG bytes at `quality`.
fn encode_jpeg(path: &Path, frame: &Frame, quality: u8) -> Result<Vec<u8>, StoreError> {
    let jpeg_error = |message: String| StoreError::Jpeg {
        path: path.to_owned(),
        message,
    };
    if !frame.is_well_formed() || frame.width == 0 || frame.height == 0 {
        return Err(jpeg_error(format!(
            "frame of {}×{} has {} bytes",
            frame.width,
            frame.height,
            frame.rgb.len()
        )));
    }
    let mut bytes = Vec::with_capacity(frame.rgb.len() / 4);
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, quality)
        .encode(
            &frame.rgb,
            frame.width,
            frame.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| jpeg_error(e.to_string()))?;
    Ok(bytes)
}

/// `jpeg` with `packet` in an XMP APP1 segment right after the JFIF APP0
/// (before the first SOF, as XMP Part 3 requires).
fn with_xmp(path: &Path, jpeg: Vec<u8>, packet: &str) -> Result<Vec<u8>, StoreError> {
    let mut parsed = Jpeg::from_bytes(jpeg.into()).map_err(|e| StoreError::Jpeg {
        path: path.to_owned(),
        message: e.to_string(),
    })?;
    let mut contents = Vec::with_capacity(XMP_NAMESPACE.len() + packet.len());
    contents.extend_from_slice(XMP_NAMESPACE);
    contents.extend_from_slice(packet.as_bytes());
    let segment = JpegSegment::new_with_contents(markers::APP1, contents.into());
    let segments = parsed.segments_mut();
    let at = segments
        .iter()
        .position(|s| s.marker() == markers::APP0)
        .map_or(0, |i| i + 1);
    segments.insert(at, segment);
    Ok(parsed.encoder().bytes().to_vec())
}

/// The `GPano` XMP packet (spec §16.4): PTGui's field set plus what Panowww
/// knows about the Shots; no `PoseHeadingDegrees`.
fn gpano_packet(
    record: &SessionRecord,
    report: &panowww_stitch::StitchReport,
    jpeg_quality: u8,
    app_version: &str,
) -> String {
    let (w, h) = (report.width, report.height);
    let first = record.shots.iter().map(|s| s.taken_at.as_str()).min();
    let last = record.shots.iter().map(|s| s.taken_at.as_str()).max();
    let dates = match (first, last) {
        (Some(first), Some(last)) => format!(
            "   <GPano:FirstPhotoDate>{first}</GPano:FirstPhotoDate>\n   <GPano:LastPhotoDate>{last}</GPano:LastPhotoDate>\n"
        ),
        _ => String::new(),
    };
    let software = xml_escape(&format!("Panowww {app_version}"));
    let count = record.shots.len();
    let anti_ghost = if report.anti_ghost { "True" } else { "False" };
    let unaligned = match report.unaligned_shots {
        UnalignedShots::PlaceFromGrid => "placeFromGrid",
        UnalignedShots::LeaveOut => "leaveOut",
    };
    format!(
        "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n\
         \x20<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n\
         \x20 <rdf:Description rdf:about=\"\" xmlns:GPano=\"http://ns.google.com/photos/1.0/panorama/\">\n\
         \x20  <GPano:ProjectionType>equirectangular</GPano:ProjectionType>\n\
         \x20  <GPano:UsePanoramaViewer>True</GPano:UsePanoramaViewer>\n\
         \x20  <GPano:CroppedAreaLeftPixels>0</GPano:CroppedAreaLeftPixels>\n\
         \x20  <GPano:CroppedAreaTopPixels>0</GPano:CroppedAreaTopPixels>\n\
         \x20  <GPano:CroppedAreaImageWidthPixels>{w}</GPano:CroppedAreaImageWidthPixels>\n\
         \x20  <GPano:CroppedAreaImageHeightPixels>{h}</GPano:CroppedAreaImageHeightPixels>\n\
         \x20  <GPano:FullPanoWidthPixels>{w}</GPano:FullPanoWidthPixels>\n\
         \x20  <GPano:FullPanoHeightPixels>{h}</GPano:FullPanoHeightPixels>\n\
         \x20  <GPano:InitialViewHeadingDegrees>0</GPano:InitialViewHeadingDegrees>\n\
         \x20  <GPano:InitialViewPitchDegrees>0</GPano:InitialViewPitchDegrees>\n\
         \x20  <GPano:StitchingSoftware>{software}</GPano:StitchingSoftware>\n\
         \x20  <GPano:SourcePhotosCount>{count}</GPano:SourcePhotosCount>\n\
         {dates}\
         \x20 </rdf:Description>\n\
         \x20 <rdf:Description rdf:about=\"\" xmlns:Panowww=\"https://github.com/Respawwwn/panowww/ns/stitch/1.0/\">\n\
         \x20  <Panowww:AntiGhost>{anti_ghost}</Panowww:AntiGhost>\n\
         \x20  <Panowww:UnalignedShots>{unaligned}</Panowww:UnalignedShots>\n\
         \x20  <Panowww:JpegQuality>{jpeg_quality}</Panowww:JpegQuality>\n\
         \x20 </rdf:Description>\n\
         \x20</rdf:RDF>\n\
         </x:xmpmeta>\n\
         <?xpacket end=\"w\"?>"
    )
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Writes `bytes` to `path` through a temporary file and a rename.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let temp = path.with_file_name(format!(".{file_name}.tmp"));
    std::fs::write(&temp, bytes).map_err(|e| StoreError::io("write", &temp, e))?;
    std::fs::rename(&temp, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        StoreError::io("rename", path, e)
    })
}

/// The XMP packet of a JPEG, if it carries one: tests and tools read the
/// `GPano` block back with it.
pub fn read_xmp(jpeg: &[u8]) -> Option<String> {
    let parsed = Jpeg::from_bytes(jpeg.to_vec().into()).ok()?;
    parsed.segments_by_marker(markers::APP1).find_map(|s| {
        let contents = s.contents();
        let packet = contents.strip_prefix(XMP_NAMESPACE)?;
        String::from_utf8(packet.to_vec()).ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_xmp_segment_sits_after_app0_and_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.jpg");
        let frame = Frame::solid(64, 32, [10, 200, 30]);
        let plain = encode_jpeg(&path, &frame, 90).unwrap();
        let tagged = with_xmp(&path, plain.clone(), "<x>packet</x>").unwrap();
        assert!(tagged.len() > plain.len() + XMP_NAMESPACE.len());
        let parsed = Jpeg::from_bytes(tagged.clone().into()).unwrap();
        let markers: Vec<u8> = parsed.segments().iter().map(JpegSegment::marker).collect();
        assert_eq!(markers[0], markers::APP0, "JFIF first");
        assert_eq!(markers[1], markers::APP1, "XMP right after");
        assert_eq!(read_xmp(&tagged).as_deref(), Some("<x>packet</x>"));
        assert_eq!(read_xmp(&plain), None);
        let decoded = image::load_from_memory(&tagged).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (64, 32));
    }

    #[test]
    fn a_malformed_frame_is_refused_as_a_jpeg_error() {
        let frame = Frame {
            width: 4,
            height: 4,
            rgb: vec![0; 5],
        };
        assert!(matches!(
            encode_jpeg(Path::new("p.jpg"), &frame, 100),
            Err(StoreError::Jpeg { .. })
        ));
    }

    #[test]
    fn software_names_are_escaped_in_the_packet() {
        assert_eq!(
            xml_escape("a <b> & \"c\""),
            "a &lt;b&gt; &amp; &quot;c&quot;"
        );
    }
}
