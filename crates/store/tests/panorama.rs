//! A synthetic Session written as PNG files into a temporary folder, stitched
//! through `FolderShotSource`, and the Panorama the store leaves behind:
//! the two JPEGs, the `GPano` packet, the `panorama` block, the zip.

#![allow(
    clippy::too_many_lines,
    reason = "one Stitch of a synthetic Session, every file it leaves checked in order"
)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use panowww_core::game::{Game, NewGameDefaults, StartPole};
use panowww_core::ports::{CancelFlag, DisplayId, Zone};
use panowww_stitch::synthetic::{MemorySource, Perturbation, session};
use panowww_stitch::{Grid, Limits, ShotSource, stitch};
use panowww_store::panorama::{self, FolderShotSource, PanoramaEntry, write_panorama};
use panowww_store::session::{
    CURRENT_VERSION, SessionFormat, SessionRecord, SessionStatus, ShotEntry, shot_file_name,
};
use panowww_store::sessions::{SessionSummary, list_all, zip_session};
use panowww_store::sink::WallClock;
use panowww_store::{StoreError, png};

struct FrozenClock(jiff::Zoned);

impl WallClock for FrozenClock {
    fn now(&self) -> jiff::Zoned {
        self.0.clone()
    }
}

fn clock() -> FrozenClock {
    FrozenClock("2026-11-14T16:00:00+01:00[+01:00]".parse().unwrap())
}

fn grid() -> Grid {
    Grid {
        rows: 5,
        shots_per_row: 8,
        start_pole: StartPole::Nadir,
    }
}

/// Writes the synthetic Session into `<root>/<game>/<timestamp>` as the
/// sink would have: one PNG per Shot and a completed `session.json`.
fn write_session(root: &Path, source: &MemorySource) -> PathBuf {
    let folder = root.join("Synthetic").join("2026-11-14_15-00-00");
    std::fs::create_dir_all(&folder).unwrap();
    let grid = grid();
    let mut game = Game::new("Synthetic", NewGameDefaults::default());
    game.rows = grid.rows;
    game.shots_per_row = grid.shots_per_row;
    game.start_pole = grid.start_pole;
    let mut shots = Vec::new();
    for i in 0..source.count() {
        let (row, col) = source.position(i);
        let file = shot_file_name(row, col);
        png::write(&folder.join(&file), &source.shot(i).unwrap()).unwrap();
        shots.push(ShotEntry {
            file,
            row,
            col,
            taken_at: format!("2026-11-14T15:00:{i:02}+01:00"),
            capture_ms: 10,
        });
    }
    let record = SessionRecord {
        format: SessionFormat::PanowwwSession,
        version: CURRENT_VERSION,
        id: uuid::Uuid::new_v4(),
        app_version: "0.0.1-test".to_owned(),
        started_at: "2026-11-14T15:00:00+01:00".to_owned(),
        ended_at: Some("2026-11-14T15:01:00+01:00".to_owned()),
        status: SessionStatus::Completed,
        error: None,
        game,
        zone: Zone {
            display_id: DisplayId("main".to_owned()),
            x: 0,
            y: 0,
            w: 320,
            h: 160,
            display_w: 2560,
            display_h: 1440,
            scale: 1.0,
        },
        shots_expected: u32::try_from(shots.len()).unwrap(),
        shots_taken: u32::try_from(shots.len()).unwrap(),
        shots,
        panorama: None,
    };
    record.write(&folder).unwrap();
    folder
}

#[test]
fn a_session_folder_is_stitched_and_its_panorama_written_with_its_block() {
    let root = tempfile::tempdir().unwrap();
    let grid = grid();
    let (memory, _truth) = session(320, 160, 90.0, &grid, &Perturbation::none(&grid));
    let folder = write_session(root.path(), &memory);

    // The folder serves the same Shots as the memory it was written from.
    let source = FolderShotSource::open(&folder).unwrap();
    assert_eq!(source.count(), memory.count());
    for i in 0..source.count() {
        assert_eq!(source.position(i), memory.position(i));
    }
    assert_eq!(source.shot(7).unwrap(), memory.shot(7).unwrap());

    let started = Instant::now();
    let stitched = stitch(
        &source,
        &grid,
        &Limits::default(),
        &|_| {},
        &CancelFlag::new(),
    )
    .unwrap();
    let entry = write_panorama(&folder, &stitched, "1.1.0-test", started, &clock()).unwrap();

    // The files.
    let jpeg = std::fs::read(folder.join("panorama.jpg")).unwrap();
    let decoded = image::load_from_memory(&jpeg).unwrap();
    assert_eq!(
        (decoded.width(), decoded.height()),
        (stitched.panorama.width, stitched.panorama.height)
    );
    assert_eq!(decoded.width(), decoded.height() * 2, "2:1");
    let preview = std::fs::read(folder.join("panorama-preview.jpg")).unwrap();
    let decoded = image::load_from_memory(&preview).unwrap();
    assert_eq!(
        (decoded.width(), decoded.height()),
        (stitched.preview.width, stitched.preview.height)
    );
    assert!(
        panorama::read_xmp(&preview).is_none(),
        "the preview carries no packet"
    );
    let names: Vec<String> = std::fs::read_dir(&folder)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| {
            std::path::Path::new(n)
                .extension()
                .is_some_and(|e| e == "tmp")
        })
        .collect();
    assert!(names.is_empty(), "temp files left: {names:?}");

    // The GPano packet.
    let xmp = panorama::read_xmp(&jpeg).expect("an XMP APP1 segment");
    assert!(xmp.starts_with("<?xpacket begin="), "{xmp}");
    assert!(xmp.trim_end().ends_with("<?xpacket end=\"w\"?>"), "{xmp}");
    for expected in [
        "<GPano:ProjectionType>equirectangular</GPano:ProjectionType>".to_owned(),
        "<GPano:UsePanoramaViewer>True</GPano:UsePanoramaViewer>".to_owned(),
        format!(
            "<GPano:FullPanoWidthPixels>{}</GPano:FullPanoWidthPixels>",
            stitched.panorama.width
        ),
        format!(
            "<GPano:FullPanoHeightPixels>{}</GPano:FullPanoHeightPixels>",
            stitched.panorama.height
        ),
        format!(
            "<GPano:CroppedAreaImageWidthPixels>{}</GPano:CroppedAreaImageWidthPixels>",
            stitched.panorama.width
        ),
        "<GPano:CroppedAreaLeftPixels>0</GPano:CroppedAreaLeftPixels>".to_owned(),
        "<GPano:StitchingSoftware>Panowww 1.1.0-test</GPano:StitchingSoftware>".to_owned(),
        "<GPano:SourcePhotosCount>40</GPano:SourcePhotosCount>".to_owned(),
        "<GPano:FirstPhotoDate>2026-11-14T15:00:00+01:00</GPano:FirstPhotoDate>".to_owned(),
        "<GPano:LastPhotoDate>2026-11-14T15:00:39+01:00</GPano:LastPhotoDate>".to_owned(),
    ] {
        assert!(xmp.contains(&expected), "missing {expected} in\n{xmp}");
    }
    assert!(!xmp.contains("PoseHeadingDegrees"));

    // The block, in the entry and on disk.
    assert_eq!(entry.file, "panorama.jpg");
    assert_eq!(entry.preview, "panorama-preview.jpg");
    assert_eq!(
        (entry.width, entry.height),
        (stitched.panorama.width, stitched.panorama.height)
    );
    assert_eq!(entry.bytes, u64::try_from(jpeg.len()).unwrap());
    assert_eq!(entry.stitched_at, "2026-11-14T16:00:00+01:00");
    assert_eq!(entry.app_version, "1.1.0-test");
    assert!(entry.duration_ms > 0);
    // Accuracy is the stitch crate's business (its tests hold 90.0 on
    // larger Shots); here the value only has to be the measured one.
    assert!(
        (80.0..100.0).contains(&entry.field_of_view_deg),
        "{}",
        entry.field_of_view_deg
    );
    assert_eq!(entry.field_of_view_deg, stitched.report.field_of_view_deg);
    assert_eq!(entry.unaligned, Vec::<String>::new());
    assert_eq!(entry.dropped_steps, Vec::<[String; 2]>::new());
    assert!(entry.anti_ghost);
    let record = SessionRecord::read(&folder).unwrap();
    assert_eq!(record.version, CURRENT_VERSION);
    assert_eq!(record.panorama.as_ref(), Some(&entry));
    assert_eq!(record.status, SessionStatus::Completed);
    let text = std::fs::read_to_string(folder.join("session.json")).unwrap();
    assert!(text.contains(r#""panorama": {"#), "{text}");
    assert!(
        text.contains(r#""stitchedAt": "2026-11-14T16:00:00+01:00""#),
        "{text}"
    );
    assert!(text.contains(r#""antiGhost": true"#), "{text}");

    // The summary carries it, the zip holds it.
    let listed = list_all(root.path()).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].panorama.as_ref(), Some(&entry));
    let zip = zip_session(&folder).unwrap();
    let archive = zip::ZipArchive::new(std::fs::File::open(&zip).unwrap()).unwrap();
    let mut names: Vec<String> = archive
        .file_names()
        .map(|n| n.unwrap().into_owned())
        .collect();
    names.sort();
    assert!(names.contains(&"panorama.jpg".to_owned()), "{names:?}");
    assert!(
        names.contains(&"panorama-preview.jpg".to_owned()),
        "{names:?}"
    );
    assert!(names.contains(&"session.json".to_owned()));
    assert_eq!(names.len(), 43, "40 Shots, the record, the two JPEGs");

    // A second Stitch replaces the files and the block.
    let later = FrozenClock("2026-11-14T17:00:00+01:00[+01:00]".parse().unwrap());
    let again = write_panorama(&folder, &stitched, "1.1.0-test", Instant::now(), &later).unwrap();
    assert_eq!(again.stitched_at, "2026-11-14T17:00:00+01:00");
    assert_eq!(
        SessionRecord::read(&folder)
            .unwrap()
            .panorama
            .unwrap()
            .stitched_at,
        "2026-11-14T17:00:00+01:00"
    );

    // Gone from disk by hand: the summary stops offering it.
    std::fs::remove_file(folder.join("panorama.jpg")).unwrap();
    assert!(!panorama::exists(&folder));
    let summary = SessionSummary::read(folder.clone()).unwrap();
    assert!(summary.record.panorama.is_some());
    assert_eq!(summary.panorama, None);
}

#[test]
fn a_record_without_the_block_still_loads_and_an_old_version_too() {
    let root = tempfile::tempdir().unwrap();
    let grid = grid();
    let (memory, _) = session(64, 32, 90.0, &grid, &Perturbation::none(&grid));
    let folder = write_session(root.path(), &memory);
    let record = SessionRecord::read(&folder).unwrap();
    assert_eq!(record.panorama, None);
    assert_eq!(panorama::existing(&folder, &record), None);

    // A version 1 file from before the Stitch.
    let text = std::fs::read_to_string(folder.join("session.json")).unwrap();
    let old = text.replacen(r#""version": 2"#, r#""version": 1"#, 1);
    assert_ne!(old, text);
    std::fs::write(folder.join("session.json"), old).unwrap();
    let record = SessionRecord::read(&folder).unwrap();
    assert_eq!(record.version, 1);
    assert_eq!(record.panorama, None);

    // The block alone round-trips through serde with every field.
    let entry = PanoramaEntry {
        file: "panorama.jpg".to_owned(),
        preview: "panorama-preview.jpg".to_owned(),
        width: 5184,
        height: 2592,
        bytes: 12_345_678,
        stitched_at: "2026-11-14T16:00:00+01:00".to_owned(),
        duration_ms: 5200,
        app_version: "1.1.0".to_owned(),
        field_of_view_deg: 90.0,
        turn_step_deg: 25.2,
        pitch_step_deg: 25.2,
        start_pole_offset_deg: 19.0,
        closing_turn_deg: 0.4,
        drift_deg: 0.3,
        unaligned: vec!["r07_c03.png".to_owned()],
        dropped_steps: vec![["r04_c07.png".to_owned(), "r04_c08.png".to_owned()]],
        anti_ghost: true,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(
        json.contains(r#""droppedSteps":[["r04_c07.png","r04_c08.png"]]"#),
        "{json}"
    );
    assert_eq!(serde_json::from_str::<PanoramaEntry>(&json).unwrap(), entry);
}

#[test]
fn a_missing_folder_is_the_store_error_not_the_stitch_error() {
    let root = tempfile::tempdir().unwrap();
    let gone = root.path().join("nowhere");
    let error = FolderShotSource::open(&gone).unwrap_err();
    assert!(
        matches!(error, StoreError::Io { action: "read", .. }),
        "{error}"
    );
    assert!(error.to_string().contains("session.json"), "{error}");
}

#[test]
fn a_missing_shot_surfaces_through_the_source_as_the_store_message() {
    let root = tempfile::tempdir().unwrap();
    let grid = grid();
    let (memory, _) = session(64, 32, 90.0, &grid, &Perturbation::none(&grid));
    let folder = write_session(root.path(), &memory);
    std::fs::remove_file(folder.join("r02_c03.png")).unwrap();
    let source = FolderShotSource::open(&folder).unwrap();
    let index = (0..source.count())
        .find(|&i| source.position(i) == (2, 3))
        .unwrap();
    let error = source.shot(index).unwrap_err();
    assert!(error.to_string().contains("r02_c03.png"), "{error}");
    // And the Stitch names the Shot.
    let error = stitch(
        &source,
        &grid,
        &Limits::default(),
        &|_| {},
        &CancelFlag::new(),
    )
    .unwrap_err();
    assert!(
        matches!(
            error,
            panowww_stitch::StitchError::Shot { row: 2, col: 3, .. }
        ),
        "{error:?}"
    );
}
