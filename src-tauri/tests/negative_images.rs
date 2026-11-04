//! Negative tests for the image pipeline: what happens when the scan is NOT a clean foot.
//! Rule (docs/NEGATIVE_TESTS.md): never panic, never report a confident foot where there is none.
//! Every case runs even if an earlier one panicked; the table is printed with `--nocapture`.

use image::{Rgb, RgbImage};
use smartfoot_app_lib::{analyze, measure};
use std::panic::{catch_unwind, AssertUnwindSafe};

const W: u32 = 1700; // 200 dpi A4: big enough to behave like a real scan, quick enough for a test
const H: u32 = 2338;

fn bg(px: u32, py: u32) -> Rgb<u8> {
    let noise = ((px * 7 + py * 13) % 13) as i32 - 6;
    let g = (110 + noise) as u8;
    Rgb([g, g, g.saturating_add(6)])
}

fn skin(px: u32, py: u32) -> Rgb<u8> {
    let noise = ((px * 7 + py * 13) % 13) as i32 - 6;
    Rgb([(215 + noise) as u8, (170 + noise) as u8, (150 + noise) as u8])
}

/// Background with filled ellipses (cx, cy, rx, ry) in pixels.
fn with_blobs(blobs: &[(f32, f32, f32, f32)]) -> RgbImage {
    RgbImage::from_fn(W, H, |x, y| {
        let hit = blobs.iter().any(|&(cx, cy, rx, ry)| ((x as f32 - cx) / rx).powi(2) + ((y as f32 - cy) / ry).powi(2) <= 1.0);
        if hit { skin(x, y) } else { bg(x, y) }
    })
}

struct Case {
    name: &'static str,
    img: RgbImage,
    /// A foot should be reported (found) or must NOT be.
    expect_found: Option<bool>,
}

fn cases() -> Vec<Case> {
    let foot = (850.0, 1200.0, 330.0, 800.0);
    let mut v = vec![
        Case { name: "baseline: one clean foot", img: with_blobs(&[foot]), expect_found: Some(true) },
        Case { name: "empty glass (lid open, grey)", img: RgbImage::from_fn(W, H, bg), expect_found: Some(false) },
        Case { name: "all white (lid closed, clipped)", img: RgbImage::from_pixel(W, H, Rgb([255, 255, 255])), expect_found: Some(false) },
        Case { name: "all black", img: RgbImage::from_pixel(W, H, Rgb([0, 0, 0])), expect_found: Some(false) },
        Case {
            name: "light falloff gradient, no object",
            img: RgbImage::from_fn(W, H, |x, y| { let g = (60 + (x * 120 / W) as i32 + ((y * 7 + x * 3) % 9) as i32) as u8; Rgb([g, g, g]) }),
            expect_found: Some(false),
        },
        // F1, measured on the real LiDE 300 on 2026-11-03: empty glass has a stable left-to-right fade
        // (about 17-24 grey levels) plus a slow top-to-bottom drift. Two harsher variants than reality.
        Case {
            name: "empty glass, real-like fade 24 levels",
            img: RgbImage::from_fn(W, H, |x, y| { let g = (95 + 24 - (x * 24 / W) as i32 + (y * 8 / H) as i32 + ((y * 7 + x * 3) % 5) as i32) as u8; Rgb([g, g, g.saturating_add(4)]) }),
            expect_found: Some(false),
        },
        Case {
            name: "empty glass, harsh fade 40 levels",
            img: RgbImage::from_fn(W, H, |x, y| { let g = (90 + 40 - (x * 40 / W) as i32 + (y * 12 / H) as i32 + ((y * 7 + x * 3) % 7) as i32) as u8; Rgb([g, g, g.saturating_add(5)]) }),
            expect_found: Some(false),
        },
        Case {
            name: "pure random noise",
            img: { let mut s = 12345u32; RgbImage::from_fn(W, H, |_, _| { s = s.wrapping_mul(1664525).wrapping_add(1013904223); let b = (s >> 24) as u8; Rgb([b, b.wrapping_add(31), b.wrapping_mul(3)]) }) },
            expect_found: None,
        },
        Case { name: "dust speck 5 mm", img: with_blobs(&[(900.0, 1100.0, 20.0, 20.0)]), expect_found: Some(false) },
        Case { name: "foot cut off at the left edge", img: with_blobs(&[(150.0, 1200.0, 330.0, 800.0)]), expect_found: Some(true) },
        Case { name: "forefoot cut off at the bottom edge", img: with_blobs(&[(850.0, 2250.0, 480.0, 800.0)]), expect_found: Some(true) },
        Case { name: "two feet side by side", img: with_blobs(&[(450.0, 1200.0, 300.0, 780.0), (1250.0, 1200.0, 300.0, 780.0)]), expect_found: None },
        Case { name: "thin ruler 15 x 300 mm", img: with_blobs(&[(850.0, 1200.0, 60.0, 1180.0)]), expect_found: None },
        Case { name: "credit card 85 x 54 mm", img: with_blobs(&[(850.0, 1200.0, 330.0, 210.0)]), expect_found: None },
    ];
    // shape plausibility: wrong objects must carry a warning, a normal foot must not
    v.push(Case { name: "ankle-like 60 mm entering bottom", img: with_blobs(&[(850.0, 1250.0, 330.0, 800.0), (850.0, 2338.0, 240.0, 600.0)]), expect_found: Some(true) });
    // very small / degenerate images, as a failing scanner or a corrupt file could deliver
    for (w, h) in [(1u32, 1u32), (3, 3), (7, 5), (15, 15), (40, 40), (4, 4000), (4000, 4)] {
        v.push(Case { name: Box::leak(format!("tiny image {w}x{h}").into_boxed_str()), img: RgbImage::from_fn(w, h, |x, y| if (x + y) % 3 == 0 { skin(x, y) } else { bg(x, y) }), expect_found: None });
    }
    v
}

#[test]
fn bad_images_never_panic_and_never_invent_a_foot() {
    let dir = std::env::temp_dir().join(format!("sf_neg_img_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut failures: Vec<String> = vec![];
    println!("\n{:<38} {:<28} {:<28} verdict", "case", "analyze", "measure");
    for (i, c) in cases().into_iter().enumerate() {
        let path = dir.join(format!("c{i}.png"));
        c.img.save(&path).unwrap();
        let a = catch_unwind(AssertUnwindSafe(|| analyze::analyze(&path)));
        let m = catch_unwind(AssertUnwindSafe(|| measure::measure_scan(&path, &measure::Scale::nominal(200.0), None)));
        let a_txt = match &a {
            Err(_) => "PANIC".to_string(),
            Ok(Err(e)) => format!("Err({})", e.chars().take(18).collect::<String>()),
            Ok(Ok(x)) => format!("found={} edge={} {:.1}%", x.found, x.touches_edge, x.area_frac * 100.0),
        };
        let m_txt = match &m {
            Err(_) => "PANIC".to_string(),
            Ok(Err(e)) => format!("Err({})", e.chars().take(18).collect::<String>()),
            Ok(Ok(x)) => format!("FL {:.0} FW {:.0} AI {:.2}", x.fl_mm, x.fw_mm, x.ai),
        };
        let mut verdict = "ok".to_string();
        let flags = match &a { Ok(Ok(x)) => (x.odd_shape, x.extra_objects), _ => (false, false) };
        let want_odd = matches!(c.name, "thin ruler 15 x 300 mm" | "credit card 85 x 54 mm");
        let want_two = c.name == "two feet side by side";
        let odd_matters = !c.name.contains("cut off"); // a cut-off blob may look odd too; the edge warning is the main one
        if matches!(&a, Ok(Ok(_))) && ((odd_matters && flags.0 != want_odd) || flags.1 != want_two) {
            verdict = format!("FAIL: odd_shape={} (want {want_odd}) two_objects={} (want {want_two})", flags.0, flags.1);
        }
        if a.is_err() || m.is_err() {
            verdict = "FAIL: panic".into();
        } else if !verdict.starts_with("FAIL") {
          if let (Some(want), Ok(Ok(x))) = (c.expect_found, &a) {
            if x.found != want {
                verdict = format!("FAIL: expected found={want}");
            }
          }
        }
        // the review screen must also agree with the measurement: no foot means nothing to measure
        if let (Ok(Ok(x)), Ok(Ok(_))) = (&a, &m) {
            if !x.found && c.expect_found == Some(false) {
                verdict = "FAIL: measured a foot that was not found".into();
            }
        }
        println!("{:<38} {:<28} {:<28} {verdict}", c.name, a_txt, m_txt);
        if verdict.starts_with("FAIL") {
            failures.push(format!("{}: {verdict}", c.name));
        }
        // a cut-off foot must raise the edge warning
        if c.name.contains("cut off") {
            if let Ok(Ok(x)) = &a {
                if !x.touches_edge {
                    failures.push(format!("{}: edge warning missing", c.name));
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(failures.is_empty(), "negative image cases failed:\n  {}", failures.join("\n  "));
}

#[test]
fn files_that_are_not_scans_give_a_clean_error() {
    let dir = std::env::temp_dir().join(format!("sf_neg_file_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // truncated PNG
    let good = dir.join("good.png");
    with_blobs(&[(850.0, 1200.0, 330.0, 800.0)]).save(&good).unwrap();
    let bytes = std::fs::read(&good).unwrap();
    let trunc = dir.join("truncated.png");
    std::fs::write(&trunc, &bytes[..bytes.len() / 3]).unwrap();
    let text = dir.join("text.png");
    std::fs::write(&text, b"this is not an image").unwrap();
    let empty = dir.join("empty.png");
    std::fs::write(&empty, b"").unwrap();
    for (name, p) in [("truncated", &trunc), ("text", &text), ("empty", &empty), ("missing", &dir.join("nope.png"))] {
        let a = catch_unwind(AssertUnwindSafe(|| analyze::analyze(p)));
        let m = catch_unwind(AssertUnwindSafe(|| measure::measure_scan(p, &measure::Scale::nominal(200.0), None)));
        assert!(matches!(a, Ok(Err(_))), "{name}: analyze should return Err, not panic or succeed");
        assert!(matches!(m, Ok(Err(_))), "{name}: measure should return Err, not panic or succeed");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
