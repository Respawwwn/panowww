//! Stitches a Session folder from the terminal, for working on the Stitch
//! with real Sessions: the stitch crate's logs on stderr, the report on
//! stdout, the Panorama written where `--out` says (never into the Session
//! folder). Run with `cargo run --release -p panowww-store --example
//! stitch_folder -- <Session folder> [--out file.jpg]`; `RUST_LOG=debug`
//! shows every pass.

use std::path::PathBuf;
use std::time::Instant;

use panowww_core::ports::CancelFlag;
use panowww_stitch::{Grid, Limits, StitchOptions, StitchProgress, stitch};
use panowww_store::panorama::FolderShotSource;
use panowww_store::session::SessionRecord;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    let mut args = std::env::args().skip(1);
    let mut folder: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next().map(PathBuf::from),
            _ => folder = Some(PathBuf::from(arg)),
        }
    }
    let Some(folder) = folder else {
        eprintln!("usage: stitch_folder <Session folder> [--out file.jpg]");
        std::process::exit(2);
    };
    let out = out.unwrap_or_else(|| std::env::temp_dir().join("stitch_folder.jpg"));
    let record = SessionRecord::read(&folder).expect("session.json");
    let source = FolderShotSource::from_record(&folder, &record);
    let grid = Grid::from_game(&record.game);
    let started = Instant::now();
    let progress = |p: StitchProgress| {
        if p.stage_fraction == 0.0 || p.stage_fraction >= 1.0 {
            eprintln!(
                "{:>6.1} s  {:?} {:.0} %",
                started.elapsed().as_secs_f64(),
                p.stage,
                p.overall * 100.0
            );
        }
    };
    match stitch(
        &source,
        &grid,
        &Limits::default(),
        &StitchOptions::default(),
        &progress,
        &CancelFlag::new(),
    ) {
        Ok(stitched) => {
            let r = &stitched.report;
            println!(
                "{} × {} in {:.2} s · hfov {:.1}° · turn {:.2}° · pitch {:.2}° · start pole {:+.1}° · closing {:+.1}° · drift {:.2}° · residual {:.2} px",
                r.width,
                r.height,
                started.elapsed().as_secs_f64(),
                r.field_of_view_deg,
                r.turn_step_deg,
                r.pitch_step_deg,
                r.start_pole_offset_deg,
                r.closing_turn_deg,
                r.drift_deg,
                r.residual_px
            );
            println!(
                "unaligned {:?} · dropped {:?} · stages {:?}",
                r.unaligned, r.dropped_steps, r.durations
            );
            let file = std::fs::File::create(&out).expect("output file");
            image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), 90)
                .encode(
                    &stitched.panorama.rgb,
                    stitched.panorama.width,
                    stitched.panorama.height,
                    image::ExtendedColorType::Rgb8,
                )
                .expect("JPEG");
            println!("written {}", out.display());
        }
        Err(error) => {
            println!("failed: {error}");
            std::process::exit(1);
        }
    }
}
