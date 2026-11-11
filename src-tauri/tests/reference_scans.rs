//! Regression test on REAL scans from the LiDE 300 (captured 2026-11-03, 300 dpi, driverless route).
//! The scans are too big for the repo (about 34 MB) and live outside it, in D:\SmartFoot_RefScans
//! (override with the SMARTFOOT_REF_SCANS environment variable). If the folder is missing the test
//! prints a note and passes, so a fresh checkout still builds and tests.
//!
//! Why it exists: the synthetic cases in negative_images.rs said "empty glass = no foot" while the
//! real scanner's faint left-to-right fade was once reported as a foot (finding F1).

use smartfoot_app_lib::{analyze, measure};
use std::path::PathBuf;

fn dir() -> PathBuf {
    std::env::var_os("SMARTFOOT_REF_SCANS").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"D:\SmartFoot_RefScans"))
}

#[test]
fn real_scans_foot_or_no_foot() {
    let d = dir();
    if !d.is_dir() {
        println!("reference scans not found at {}, skipping", d.display());
        return;
    }
    // (file, a foot-like object must be found?, measurement must refuse?)
    let cases = [
        ("empty_lidopen_lightON", false, true),
        ("empty_lidopen_lightOFF_a", false, true),
        ("empty_lidopen_lightOFF_b", false, true),
        ("smallobj_lightOFF", false, true),
        ("empty_lidclosed", false, false), // white lid: nothing to find; the measurement is not asserted
        ("hand_lightOFF", true, false),
        ("packet_lightOFF", true, false),
    ];
    let mut failures = vec![];
    for (name, want_found, want_refuse) in cases {
        let p = d.join(format!("{name}.png"));
        if !p.exists() {
            println!("{name}: file missing, skipped");
            continue;
        }
        let a = analyze::analyze(&p).expect("analyze");
        let m = measure::measure_scan(&p, &measure::Scale::nominal(300.0), None);
        println!("{name:<28} found={} area={:.1}% measure={}", a.found, a.area_frac * 100.0, if m.is_ok() { "numbers" } else { "refused" });
        if a.found != want_found {
            failures.push(format!("{name}: found={} (want {want_found})", a.found));
        }
        if !a.found {
            let blue = image::load_from_memory(&a.preview_png).expect("preview").to_rgb8().pixels().filter(|p| p.0 == [31, 95, 191]).count();
            if blue > 0 {
                failures.push(format!("{name}: {blue} outline pixels drawn although no foot was found"));
            }
        }
        if want_refuse && m.is_ok() {
            failures.push(format!("{name}: measurement produced numbers although there is no foot"));
        }
        if want_found && m.is_err() {
            failures.push(format!("{name}: measurement refused a real object"));
        }
    }
    assert!(failures.is_empty(), "real-scan regression failures:\n  {}", failures.join("\n  "));
}

/// Scans through the final case (10 mm acrylic window, LiDE 300 inside), captured 2026-11-11.
/// Live in D:\SmartFoot_CaseScans (override with SMARTFOOT_CASE_SCANS); skipped if missing.
/// Why: the sole sits ~10 mm above the focus, is blurred and often darker than the background, and
/// the first finder outlined only the pale rim, called an empty lit case a foot and a half foot "none".
#[test]
fn case_scans_through_acrylic() {
    let d = std::env::var_os("SMARTFOOT_CASE_SCANS").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"D:\SmartFoot_CaseScans"));
    if !d.is_dir() {
        println!("case scans not found at {}, skipping", d.display());
        return;
    }
    // (file, foot found?, edge warning?, foot length range in mm from the measurement, if found)
    let cases: [(&str, bool, bool, (f32, f32)); 7] = [
        ("empty_lightOFF", false, false, (0.0, 0.0)),
        ("empty_lightON", false, false, (0.0, 0.0)), // was a false foot (dark column band)
        ("foot_lightOFF", true, false, (200.0, 280.0)),
        ("footR_mid_lightOFF", true, true, (200.0, 280.0)), // heel ran off the bottom edge
        ("footR_mid_lightON", true, false, (200.0, 280.0)),
        ("footL_mid_lightOFF", true, false, (200.0, 280.0)),
        ("footR_edge_lightOFF", true, true, (0.0, 0.0)), // half the foot past the left edge (was "no foot")
    ];
    let mut failures = vec![];
    for (name, want_found, want_edge, (fl_lo, fl_hi)) in cases {
        let p = d.join(format!("{name}.png"));
        if !p.exists() {
            println!("{name}: file missing, skipped");
            continue;
        }
        let a = analyze::analyze(&p).expect("analyze");
        let m = measure::measure_scan(&p, &measure::Scale::nominal(300.0), None);
        let fl = m.as_ref().map(|m| m.fl_mm).unwrap_or(0.0);
        println!("{name:<22} found={} edge={} odd={} area={:.1}% FL={fl:.0} mm", a.found, a.touches_edge, a.odd_shape, a.area_frac * 100.0);
        if a.found != want_found {
            failures.push(format!("{name}: found={} (want {want_found})", a.found));
        }
        if a.found && a.touches_edge != want_edge {
            failures.push(format!("{name}: edge={} (want {want_edge})", a.touches_edge));
        }
        if !want_found && m.is_ok() {
            failures.push(format!("{name}: measured a foot that is not there"));
        }
        if want_found && fl_hi > 0.0 && !(fl_lo..=fl_hi).contains(&fl) {
            failures.push(format!("{name}: foot length {fl:.0} mm outside {fl_lo}-{fl_hi}"));
        }
        if want_found && !want_edge && a.odd_shape {
            failures.push(format!("{name}: whole foot flagged as odd shape"));
        }
    }
    assert!(failures.is_empty(), "case-scan regression failures:\n  {}", failures.join("\n  "));
}
