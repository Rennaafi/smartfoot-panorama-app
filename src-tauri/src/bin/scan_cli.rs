//! Hardware test tool: `cargo run --bin scan_cli -- list` or
//! `cargo run --bin scan_cli -- scan "Canon LiDE 300 HTTP" 300 out.png`

use smartfoot_app_lib::scanner;
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("list") => match scanner::list_scanners() {
            Ok(v) if v.is_empty() => println!("no scanners found"),
            Ok(v) => v.iter().enumerate().for_each(|(i, d)| println!("[{}] {d}", i + 1)),
            Err(e) => eprintln!("error: {e}"),
        },
        Some("scan") if a.len() == 4 => {
            let dpi: u32 = a[2].parse().expect("dpi must be a number");
            match scanner::scan(&a[1], dpi, Path::new(&a[3])) {
                Ok(r) => println!("saved {} ({}x{} px, {:.1} s)", r.path.display(), r.width_px, r.height_px, r.seconds),
                Err(e) => eprintln!("error: {e}"),
            }
        }
        _ => eprintln!("usage: scan_cli list | scan <device> <dpi> <out.png>"),
    }
}
