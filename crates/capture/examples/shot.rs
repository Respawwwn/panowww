//! The hands-on check of issue 10: lists the displays, captures a zone of
//! one of them into `shot.png`, and runs the Display preview for three
//! seconds to count its frames.
//!
//! ```sh
//! cargo run -p panowww-capture --example shot -- check
//! cargo run -p panowww-capture --example shot -- list
//! cargo run -p panowww-capture --example shot -- 0 100 100 800 450   # display index, x y w h in points
//! ```
//!
//! Open the resulting `shot.png`: no Panowww window, no cursor, native
//! pixels. Record the timings in `docs/manual-test.md`.

#[cfg(target_os = "macos")]
#[allow(clippy::too_many_lines, reason = "one linear hands-on script")]
fn main() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use panowww_capture::macos::{SckGrabber, list_displays, permission};
    use panowww_core::ports::{Grabber, Rect};

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") => {
            println!("screen recording granted: {}", permission::check());
            if !permission::check() {
                println!("requesting: {}", permission::request());
                println!("opening System Settings: {:?}", permission::open_settings());
                println!("relaunch after granting.");
            }
            return;
        }
        Some("list") | None => {
            match list_displays() {
                Ok(displays) => {
                    for (i, d) in displays.iter().enumerate() {
                        println!(
                            "{i}: {} · {}×{} pt · scale {} · origin ({}, {})",
                            d.info.id.0,
                            d.info.width,
                            d.info.height,
                            d.info.scale,
                            d.bounds.origin.x,
                            d.bounds.origin.y
                        );
                    }
                }
                Err(error) => eprintln!("{error}"),
            }
            if args.is_empty() {
                eprintln!("usage: shot check | list | <display index> <x> <y> <w> <h>");
            }
            return;
        }
        Some(_) => {}
    }

    let numbers: Vec<i64> = args.iter().filter_map(|a| a.parse().ok()).collect();
    let [index, left, top, width, height] = numbers[..] else {
        eprintln!("usage: shot <display index> <x> <y> <w> <h>");
        std::process::exit(2);
    };
    let displays = list_displays().expect("displays");
    let display = displays
        .get(usize::try_from(index).unwrap_or(usize::MAX))
        .expect("display index");
    let zone = Rect {
        x: i32::try_from(left).unwrap_or(0),
        y: i32::try_from(top).unwrap_or(0),
        w: u32::try_from(width).unwrap_or(1),
        h: u32::try_from(height).unwrap_or(1),
    };

    let frames = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&frames);
    let started = Instant::now();
    let mut stream = SckGrabber
        .start(
            display.info.id.clone(),
            zone,
            Box::new(move |frame| {
                let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
                println!(
                    "preview frame {n} at {} ms: {} bytes, display {}×{}",
                    started.elapsed().as_millis(),
                    frame.jpeg.0.len(),
                    frame.display_w,
                    frame.display_h
                );
            }),
        )
        .expect("stream opens (Screen Recording granted?)");

    let t0 = Instant::now();
    let frame = stream.shot(Instant::now()).expect("shot");
    println!(
        "shot: {}×{} px in {} ms",
        frame.width,
        frame.height,
        t0.elapsed().as_millis()
    );
    let t1 = Instant::now();
    let second = stream.shot(Instant::now()).expect("second shot");
    println!("second shot: {} ms", t1.elapsed().as_millis());
    drop(second);

    let file = std::fs::File::create("shot.png").expect("shot.png");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let t2 = Instant::now();
    let mut writer = encoder.write_header().expect("png header");
    writer.write_image_data(&frame.rgb).expect("png data");
    writer.finish().expect("png finish");
    println!("shot.png written in {} ms", t2.elapsed().as_millis());

    stream.set_preview(true);
    std::thread::sleep(Duration::from_secs(3));
    let before_stop = frames.load(Ordering::SeqCst);
    stream.set_preview(false);
    let at_stop = frames.load(Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(800));
    let after = frames.load(Ordering::SeqCst);
    println!(
        "preview: {before_stop} frames in 3 s (expect about 6), {at_stop} at stop, {after} after stop"
    );
    stream.stop();
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("shot is the macOS check of issue 10; Windows arrives with issue 24.");
}
