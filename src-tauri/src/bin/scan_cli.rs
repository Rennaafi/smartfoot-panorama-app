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
        // Test tool: a screening session built from scans already on disk, through the same session code
        // the app uses (no scanner needed). Never point it at a real data folder.
        Some("seed") if a.len() == 3 => match seed(Path::new(&a[1]), Path::new(&a[2])) {
            Ok(n) => println!("seeded {n} patient(s) into {}", a[1]),
            Err(e) => eprintln!("seed FAILED: {e}"),
        },
        _ => eprintln!("usage: scan_cli list | status | test | backup <data_dir> <backup_dir> | scan <device> <dpi> <out.png> | analyze <scan.png> <preview.png> | measure <scan.png> | masks <scan.png> <masks.png> | pictures <scan.png> <out_dir> <left|right> | sheet <session_dir> <out.jpg> | seed <empty_data_dir> <list.txt>"),
    }
}

/// `list.txt`: first line `place|operator`, then one patient per line:
/// `nama|usia|Laki-laki or Perempuan|berdiri or duduk|left scan|right scan or -|inlow: none, half or full|done or open`.
/// A right scan of a standing patient is turned 180° as the app does (the patient turned around).
fn seed(dd: &Path, list: &Path) -> Result<usize, String> {
    use smartfoot_app_lib::{export, figures, inlow, session};
    if dd.exists() && std::fs::read_dir(dd).map_err(|e| e.to_string())?.next().is_some() {
        return Err(format!("{} is not empty", dd.display()));
    }
    std::fs::create_dir_all(dd).map_err(|e| e.to_string())?;
    let text = std::fs::read_to_string(list).map_err(|e| e.to_string())?;
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#'));
    let head: Vec<&str> = lines.next().ok_or("empty list")?.split('|').collect();
    let (sd, _) = session::start_sesi(dd, head[0], head.get(1).copied().unwrap_or("Uji"), "TST")?;
    let mut n = 0;
    for (i, line) in lines.enumerate() {
        let f: Vec<&str> = line.split('|').map(str::trim).collect();
        if f.len() != 8 {
            return Err(format!("line {}: expected 8 fields", i + 2));
        }
        let form = session::PatientForm { nama: f[0].into(), usia: f[1].parse().ok(), kelamin: f[2].into(), td: String::new(), bb: None, posisi: f[3].into() };
        let dir = session::create_patient(&sd, &form, "seed (scan_cli)")?;
        for (side, src) in [("left", f[4]), ("right", f[5])] {
            if src == "-" {
                continue;
            }
            let pend = session::pending_file(&dir, side);
            std::fs::copy(src, &pend).map_err(|e| format!("{src}: {e}"))?;
            let turn = side == "right" && f[3] == "berdiri";
            if turn {
                smartfoot_app_lib::rotate_saved_scan(&pend)?;
            }
            let (w, h) = image::image_dimensions(&pend).map_err(|e| e.to_string())?;
            session::accept(&dir, side, 0.0, w, h, turn, vec![])?;
            let raw = dir.join(session::raw_scan_name(side));
            let m = measure::measure_scan(&raw, &measure::Scale::calibrated(), None).ok();
            figures::write_scan_images(&dir, side, &raw)?;
            if let Some(m) = &m {
                figures::write_arch_index_image(&dir, side, &raw, m)?;
            }
            session::set_measurements(&dir, side, m)?;
        }
        let fill = match f[6] { "full" => inlow::ITEMS.len(), "half" => inlow::ITEMS.len() / 2, _ => 0 };
        let mut inl = inlow::Inlow::default();
        for (k, it) in inlow::ITEMS.iter().take(fill).enumerate() {
            inl.kanan.items.insert(it.key.into(), it.options[(k + i) % it.options.len()].0);
            inl.kiri.items.insert(it.key.into(), it.options[0].0);
        }
        session::set_inlow(&dir, inl)?;
        if f[7] == "done" {
            session::complete(&dir)?;
        }
        n += 1;
    }
    for e in export::rebuild(dd, Some(&sd)) {
        eprintln!("warning: {e}");
    }
    Ok(n)
}
