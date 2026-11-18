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

/// Standing scans of one person's feet on the platform, black cloth, 2026-11-18 (D:\SmartFoot_CaseScans\2026-11-18).
/// L1-L3 = left foot, R1-R6 = right foot with the person turned around (heel at the top of the image).
/// L1 had less weight on the foot; R5 leaned towards the scanner foot. What must hold:
///  * outline numbers repeat (FL within 4 mm per foot, outline AI within 0.03);
///  * the contact map repeats when the foot carries its share (L2/L3: contact AI within 0.03);
///  * Clarke's angle from contact is found and stable where the contact runs along the outer border
///    (L2/L3, within 5 degrees), and None where the pressed skin is split (right foot);
///  * the big-toe side (where the Clarke tangent rests on heel and ball) is on the image-left for all.
#[test]
fn standing_scans_repeat() {
    let base = std::env::var_os("SMARTFOOT_CASE_SCANS").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"D:\SmartFoot_CaseScans"));
    let d = base.join("2026-11-18");
    if !d.is_dir() {
        println!("standing scans not found at {}, skipping", d.display());
        return;
    }
    let get = |n: &str| {
        let p = d.join(format!("stand_{n}.png"));
        if !p.exists() {
            return None;
        }
        Some(measure::measure_scan(&p, &measure::Scale::calibrated(), None).expect(n))
    };
    let all: Vec<(&str, measure::Measurements)> = ["L1", "L2", "L3", "R1", "R2", "R3", "R4", "R5", "R6"].iter().filter_map(|n| get(n).map(|m| (*n, m))).collect();
    if all.len() < 9 {
        println!("some standing scans missing, skipping");
        return;
    }
    let mut failures = vec![];
    let spread = |f: &dyn Fn(&measure::Measurements) -> f32, names: &[&str]| {
        let v: Vec<f32> = all.iter().filter(|(n, _)| names.contains(n)).map(|(_, m)| f(m)).collect();
        v.iter().cloned().fold(f32::MIN, f32::max) - v.iter().cloned().fold(f32::MAX, f32::min)
    };
    for (n, m) in &all {
        let c = m.contact.as_ref().expect("contact measured");
        println!("{n}: FL {:.1} AI {:.3} contact AI {:.3} ({:.0} % of foot) Clarke {:?} toe base {}", m.fl_mm, m.ai, c.ai, c.frac * 100.0, c.clarke_deg, m.toe_base_method);
        match &m.clarke_pts {
            Some(p) if p.heel[0] < m.heel_centre[0] && p.fore[0] < m.toe_base[0] => {}
            _ => failures.push(format!("{n}: big-toe side not on the image-left")),
        }
    }
    let feet: [(&[&str], &str); 2] = [(&["L2", "L3"], "left"), (&["R1", "R2", "R3", "R4", "R5", "R6"], "right")];
    for (names, foot) in feet {
        if spread(&|m| m.fl_mm, names) > 4.0 {
            failures.push(format!("{foot}: FL spread {:.1} mm", spread(&|m| m.fl_mm, names)));
        }
        if spread(&|m| m.ai, names) > 0.03 {
            failures.push(format!("{foot}: outline AI spread {:.3}", spread(&|m| m.ai, names)));
        }
    }
    let l23 = ["L2", "L3"];
    if spread(&|m| m.contact.as_ref().unwrap().ai, &l23) > 0.03 {
        failures.push("left: contact AI of L2/L3 differ by more than 0.03".into());
    }
    let cl: Vec<Option<f32>> = all.iter().filter(|(n, _)| l23.contains(n)).map(|(_, m)| m.contact.as_ref().unwrap().clarke_deg).collect();
    match (cl[0], cl[1]) {
        (Some(a), Some(b)) if (a - b).abs() <= 5.0 && (25.0..65.0).contains(&a) => {}
        other => failures.push(format!("left: contact Clarke L2/L3 = {other:?}")),
    }
    for (n, m) in all.iter().filter(|(n, _)| n.starts_with('R')) {
        if m.contact.as_ref().unwrap().clarke_deg.is_some() {
            failures.push(format!("{n}: Clarke measured on a split contact map"));
        }
    }
    assert!(failures.is_empty(), "standing-scan failures:\n  {}", failures.join("\n  "));
}
