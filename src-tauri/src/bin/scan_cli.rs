//! Hardware test tool: `cargo run --bin scan_cli -- list` or
//! `cargo run --bin scan_cli -- scan "Canon LiDE 300 HTTP" 300 out.png`

use smartfoot_app_lib::{analyze, measure, scanner};
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
                println!("found={} touches_edge={} limb_at_edge={} clear={} area={:.1}% bbox={:?} -> {}", r.found, r.touches_edge, r.limb_at_edge, r.clear, r.area_frac * 100.0, r.bbox, a[2]);
            }
            Err(e) => eprintln!("error: {e}"),
        },
        Some("measure") if a.len() == 2 => match measure::measure_scan(Path::new(&a[1]), &measure::Scale::nominal(300.0), None) {
            Ok(m) => println!("{}", serde_json::to_string_pretty(&m).unwrap()),
            Err(e) => eprintln!("error: {e}"),
        },
        _ => eprintln!("usage: scan_cli list | status | test | scan <device> <dpi> <out.png> | analyze <scan.png> <preview.png> | measure <scan.png>"),
    }
}
