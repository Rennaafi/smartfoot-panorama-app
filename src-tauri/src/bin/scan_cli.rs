//! Hardware test tool: `cargo run --bin scan_cli -- list` or
//! `cargo run --bin scan_cli -- scan "Canon LiDE 300 HTTP" 300 out.png`

use smartfoot_app_lib::{analyze, backup, measure, scanner};
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("list") => match scanner::list_scanners() {
            Ok(v) if v.is_empty() => println!("no scanners found"),
            Ok(v) => v.iter().enumerate().for_each(|(i, d)| println!("[{}] {d}", i + 1)),
            Err(e) => eprintln!("error: {e}"),
        },
        Some("status") => match scanner::status(None) {
            Ok(s) => println!("{}", serde_json::to_string_pretty(&s).unwrap()),
            Err(e) => eprintln!("error: {e}"),
        },
        Some("test") => match scanner::status(None).and_then(|s| s.chosen.ok_or("NOT_FOUND: no scanner".to_string())) {
            Ok(dev) => match scanner::test_scan(&dev, &std::env::temp_dir()) {
                Ok(secs) => println!("test scan OK on {dev}: {secs:.1} s"),
                Err(e) => eprintln!("test scan FAILED on {dev}: {e}"),
            },
            Err(e) => eprintln!("error: {e}"),
        },
        Some("backup") if a.len() == 3 => match backup::sync_all(Path::new(&a[1]), Path::new(&a[2])) {
            Ok(n) => println!("backup OK: {n} file(s) copied"),
            Err(e) => eprintln!("backup FAILED: {e}"),
        },
        Some("scan") if a.len() == 4 => {
            let dpi: u32 = a[2].parse().expect("dpi must be a number");
            match scanner::scan(&a[1], dpi, Path::new(&a[3])) {
                Ok(r) => println!("saved {} ({}x{} px, {:.1} s)", r.path.display(), r.width_px, r.height_px, r.seconds),
                Err(e) => eprintln!("error: {e}"),
            }
        }
        Some("analyze") if a.len() == 3 => match analyze::analyze(Path::new(&a[1])) {
            Ok(r) => {
                std::fs::write(&a[2], &r.preview_png).expect("write preview");
                println!("found={} touches_edge={} clear={} odd_shape={} area={:.1}% bbox={:?} -> {}", r.found, r.touches_edge, r.clear, r.odd_shape, r.area_frac * 100.0, r.bbox, a[2]);
            }
            Err(e) => eprintln!("error: {e}"),
        },
        // Developer diagnostics: the numbers behind the foot/no-foot decision for one or more scans.
        Some("diag") if a.len() >= 2 => {
            // level = typical redness of the seeds (R-G above background), thr = outline threshold,
            // pale = brightness threshold for pale pressed skin
            println!("{:<30} {:>6} {:>5} {:>5} {:>6} {:>6} {:>5} {:>5} {:>5}", "scan", "level", "thr", "pale", "area%", "extra", "found", "odd", "edge");
            for p in &a[1..] {
                let full = image::open(p).expect("open").to_rgb8();
                let img = image::imageops::resize(&full, full.width() / 4, full.height() / 4, image::imageops::FilterType::Triangle);
                let s = analyze::segment(&img, 4);
                let r = analyze::analyze_image(&img);
                let name = Path::new(p).file_stem().unwrap().to_string_lossy();
                println!(
                    "{:<30} {:>6.1} {:>5.1} {:>5.1} {:>6.1} {:>6.2} {:>5} {:>5} {:>5}",
                    name, s.level, s.thr, s.pale_thr, r.area_frac * 100.0, s.extra_frac, r.found, r.odd_shape, r.touches_edge
                );
            }
        }
        Some("measure") if a.len() == 2 => match measure::measure_scan(Path::new(&a[1]), &measure::Scale::calibrated(), None) {
            Ok(m) => println!("{}", serde_json::to_string_pretty(&m).unwrap()),
            Err(e) => eprintln!("error: {e}"),
        },
        // Developer diagnostics: outline (grey 128) and contact (white) masks at measuring size.
        Some("masks") if a.len() == 3 => {
            let full = image::open(&a[1]).expect("open").to_rgb8();
            let d = measure::MEASURE_DOWN;
            let img = image::imageops::resize(&full, full.width() / d, full.height() / d, image::imageops::FilterType::Triangle);
            let s = analyze::segment(&img, d);
            let c = smartfoot_app_lib::contact::find(&img, &s.mask, d);
            let out = image::GrayImage::from_fn(s.w, s.h, |x, y| {
                let i = (y * s.w + x) as usize;
                image::Luma([if c.mask[i] { 255 } else if s.mask[i] { 128 } else { 0 }])
            });
            out.save(&a[2]).expect("write masks");
            println!("thr={:.3} contact={:.1}% of foot -> {}", c.thr, c.frac * 100.0, a[2]);
        }
        // Team tool: the pictures the app saves for one scan, written into a folder (for checking).
        Some("pictures") if a.len() == 4 && (a[3] == "left" || a[3] == "right") => {
            let (scan, out) = (Path::new(&a[1]), Path::new(&a[2]));
            std::fs::create_dir_all(out).expect("create output folder");
            match smartfoot_app_lib::figures::write_scan_images(out, &a[3], scan).and_then(|mut f| {
                let m = measure::measure_scan(scan, &measure::Scale::calibrated(), None)?;
                f.push(smartfoot_app_lib::figures::write_arch_index_image(out, &a[3], scan, &m)?);
                Ok(f)
            }) {
                Ok(f) => f.iter().for_each(|p| println!("{}", out.join(p).display())),
                Err(e) => eprintln!("error: {e}"),
            }
        }
        // Team tool: one comparison sheet for a whole session folder (+ a .txt index of the rows).
        Some("sheet") if a.len() == 3 => match smartfoot_app_lib::figures::session_sheet(Path::new(&a[1]), Path::new(&a[2])) {
            Ok(n) => println!("{n} row(s) -> {}", a[2]),
            Err(e) => eprintln!("error: {e}"),
        },
        _ => eprintln!("usage: scan_cli list | status | test | backup <data_dir> <backup_dir> | scan <device> <dpi> <out.png> | analyze <scan.png> <preview.png> | measure <scan.png> | masks <scan.png> <masks.png> | pictures <scan.png> <out_dir> <left|right> | sheet <session_dir> <out.jpg>"),
    }
}
