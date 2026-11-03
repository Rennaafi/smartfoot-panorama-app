//! Foot measurements from the foot mask.
//!
//! Definitions (Cavanagh & Rodgers arch index, as in the Umardani scanner studies):
//!  * Foot axis: the line from the heel centre H to the second-toe tip J. H is found 15 % of the foot
//!    length in from the heel end; J starts as the most distal point. Both can be corrected by hand.
//!  * Foot length FL: extent of the foot along the axis. Foot width FW: widest cut across the axis.
//!  * Toes are excluded for the arch index: the toeless footprint runs from the heel end to the toe
//!    base. The toe base is where the toes stop being separate (a gap in the cross-section), or, when
//!    the toes touch, 75 % of FL (marked "proportion" in the result).
//!  * The toeless length is cut into thirds: hindfoot A, midfoot B, forefoot C. AI = B / (A + B + C).
//!  * Chippaux-Smirak index = narrowest midfoot width / widest forefoot width x 100.
//!
//! Everything is computed in millimetres from the pixel size in `Scale`. Until the calibration tool
//! exists the scale is the nominal scanner resolution and `calibrated` is false.

use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Scans are measured on a half-size copy (0.17 mm per pixel at 300 dpi).
pub const MEASURE_DOWN: u32 = 2;
const HEEL_CENTRE_FRAC: f32 = 0.15;
const TOE_BASE_FALLBACK: f32 = 0.75;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Scale {
    /// mm per pixel across the image width (sensor direction) and down the image (scan direction).
    pub x_mm_per_px: f32,
    pub y_mm_per_px: f32,
    pub calibrated: bool,
}

impl Scale {
    pub fn nominal(dpi: f32) -> Self {
        Scale { x_mm_per_px: 25.4 / dpi, y_mm_per_px: 25.4 / dpi, calibrated: false }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Measurements {
    pub scale: Scale,
    pub fl_mm: f32,
    pub fw_mm: f32,
    pub area_cm2: f32,
    pub toeless_area_cm2: f32,
    pub hind_cm2: f32, // A
    pub mid_cm2: f32,  // B
    pub fore_cm2: f32, // C
    pub ai: f32,
    pub csi_pct: f32,
    /// "gap" = toe base found from the toe gaps, "proportion" = 75 % of FL because the toes touch.
    pub toe_base_method: String,
    pub toeless_frac: f32,
    /// Points in full-resolution image pixels, for drawing and dragging.
    pub heel_centre: [f32; 2],
    pub toe_tip: [f32; 2],
    pub heel_end: [f32; 2],
    pub toe_base: [f32; 2],
    /// True when a team member moved heel centre / toe tip by hand.
    pub corrected: bool,
}

type P = [f32; 2];
fn dot(a: P, b: P) -> f32 { a[0] * b[0] + a[1] * b[1] }
fn sub(a: P, b: P) -> P { [a[0] - b[0], a[1] - b[1]] }
fn add(a: P, b: P, k: f32) -> P { [a[0] + b[0] * k, a[1] + b[1] * k] }
fn unit(a: P) -> P { let n = dot(a, a).sqrt().max(1e-6); [a[0] / n, a[1] / n] }

/// Mean of a sliding window of `win` bins.
fn smooth(v: &[f32], win: usize) -> Vec<f32> {
    let r = win / 2;
    (0..v.len())
        .map(|i| {
            let (a, b) = (i.saturating_sub(r), (i + r + 1).min(v.len()));
            v[a..b].iter().sum::<f32>() / (b - a) as f32
        })
        .collect()
}

/// `down` = how many full-resolution pixels one mask pixel covers.
/// `pts` = optional hand-placed (heel centre, toe tip) in full-resolution pixels.
pub fn measure_mask(mask: &[bool], w: usize, h: usize, scale: &Scale, down: f32, pts: Option<(P, P)>) -> Option<Measurements> {
    let (fx, fy) = (down * scale.x_mm_per_px, down * scale.y_mm_per_px);
    let mut p: Vec<P> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if mask[y * w + x] {
                p.push([(x as f32 + 0.5) * fx, (y as f32 + 0.5) * fy]);
            }
        }
    }
    if p.len() < 500 {
        return None;
    }
    let px_area = fx * fy; // mm^2

    // principal axis of the mask
    let n = p.len() as f32;
    let c = [p.iter().map(|q| q[0]).sum::<f32>() / n, p.iter().map(|q| q[1]).sum::<f32>() / n];
    let (mut sxx, mut syy, mut sxy) = (0f32, 0f32, 0f32);
    for q in &p {
        let d = sub(*q, c);
        sxx += d[0] * d[0];
        syy += d[1] * d[1];
        sxy += d[0] * d[1];
    }
    let ang = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    let mut u = [ang.cos(), ang.sin()];

    // which end are the toes? the widest part (the ball) lies nearer the toes than the heel
    let t0: Vec<f32> = p.iter().map(|q| dot(sub(*q, c), u)).collect();
    let (tmin, tmax) = t0.iter().fold((f32::MAX, f32::MIN), |a, &t| (a.0.min(t), a.1.max(t)));
    let nb = ((tmax - tmin) / 2.0).ceil() as usize + 1;
    let mut cnt = vec![0f32; nb];
    for &t in &t0 {
        cnt[(((t - tmin) / 2.0) as usize).min(nb - 1)] += 1.0;
    }
    let sm = smooth(&cnt, 5);
    let widest = sm.iter().enumerate().fold((0, 0.0), |a, (i, &v)| if v > a.1 { (i, v) } else { a }).0;
    if (widest as f32 + 0.5) / (nb as f32) < 0.5 {
        u = [-u[0], -u[1]]; // toes were at the low end: flip
    }
    let t1: Vec<f32> = p.iter().map(|q| dot(sub(*q, c), u)).collect();
    let (tlo, thi) = t1.iter().fold((f32::MAX, f32::MIN), |a, &t| (a.0.min(t), a.1.max(t)));
    let fl0 = thi - tlo;
    let v = [-u[1], u[0]];

    // heel centre H and toe tip J (automatic unless given)
    let (hc, tip) = match pts {
        Some((h, j)) => ([h[0] * scale.x_mm_per_px, h[1] * scale.y_mm_per_px], [j[0] * scale.x_mm_per_px, j[1] * scale.y_mm_per_px]),
        None => {
            let th = tlo + HEEL_CENTRE_FRAC * fl0;
            let band: Vec<f32> = p.iter().zip(&t1).filter(|(_, t)| (**t - th).abs() < 1.5).map(|(q, _)| dot(sub(*q, c), v)).collect();
            let sh = band.iter().sum::<f32>() / band.len().max(1) as f32;
            let tipband: Vec<f32> = p.iter().zip(&t1).filter(|(_, t)| **t > thi - 0.8).map(|(q, _)| dot(sub(*q, c), v)).collect();
            let st = tipband.iter().sum::<f32>() / tipband.len().max(1) as f32;
            (add(add(c, u, th), v, sh), add(add(c, u, thi), v, st))
        }
    };

    // measurement frame along the foot axis, origin at H
    let a = unit(sub(tip, hc));
    let b = [-a[1], a[0]];
    let (mut ts, mut ss) = (Vec::with_capacity(p.len()), Vec::with_capacity(p.len()));
    for q in &p {
        let d = sub(*q, hc);
        ts.push(dot(d, a));
        ss.push(dot(d, b));
    }
    let (t_min, t_max) = ts.iter().fold((f32::MAX, f32::MIN), |m, &t| (m.0.min(t), m.1.max(t)));
    let fl = t_max - t_min;
    if fl < 50.0 {
        return None;
    }

    // 1 mm slices across the axis: width and the separate runs in each
    let nbin = fl.ceil() as usize + 1;
    let mut slices: Vec<Vec<f32>> = vec![Vec::new(); nbin];
    for (i, &t) in ts.iter().enumerate() {
        slices[((t - t_min) as usize).min(nbin - 1)].push(ss[i]);
    }
    let mut width = vec![0f32; nbin];
    let mut runs = vec![1usize; nbin];
    for (k, sl) in slices.iter_mut().enumerate() {
        if sl.is_empty() {
            continue;
        }
        sl.sort_by(|x, y| x.partial_cmp(y).unwrap());
        width[k] = sl[sl.len() - 1] - sl[0];
        runs[k] = 1 + sl.windows(2).filter(|w| w[1] - w[0] > 1.0).count();
    }

    // toe base: the lowest slice (beyond 60 % of FL) from which at least 3 slices in a row show separate toes
    let lo = (0.60 * fl) as usize;
    let gap_at = (lo..nbin.saturating_sub(2)).find(|&k| runs[k] >= 2 && runs[k + 1] >= 2 && runs[k + 2] >= 2);
    let (base_frac, method) = match gap_at {
        Some(k) if (0.65..=0.88).contains(&(k as f32 / fl)) => (k as f32 / fl, "gap"),
        _ => (TOE_BASE_FALLBACK, "proportion"),
    };
    let t_base = t_min + base_frac * fl;
    let lt = t_base - t_min;

    // thirds of the toeless footprint
    let (mut ca, mut cb, mut cc, mut call) = (0f32, 0f32, 0f32, 0f32);
    for &t in &ts {
        call += 1.0;
        if t < t_min + lt / 3.0 {
            ca += 1.0
        } else if t < t_min + 2.0 * lt / 3.0 {
            cb += 1.0
        } else if t < t_base {
            cc += 1.0
        }
    }
    let to_cm2 = px_area / 100.0;
    let (ha, hb, hcn) = (ca * to_cm2, cb * to_cm2, cc * to_cm2);
    let ai = hb / (ha + hb + hcn).max(1e-6);

    // widths: foot width = widest slice; CSI = narrowest midfoot / widest forefoot
    let ws = smooth(&width, 5);
    let fw = ws.iter().cloned().fold(0.0, f32::max);
    let slice_range = |from: f32, to: f32| (((from - t_min) as usize).min(nbin - 1))..(((to - t_min).ceil() as usize).min(nbin));
    let mid_min = ws[slice_range(t_min + lt / 3.0, t_min + 2.0 * lt / 3.0)].iter().cloned().fold(f32::MAX, f32::min);
    let fore_max = ws[slice_range(t_min + 2.0 * lt / 3.0, t_base)].iter().cloned().fold(0.0, f32::max);
    let csi = if fore_max > 0.0 && mid_min < f32::MAX { 100.0 * mid_min / fore_max } else { 0.0 };

    let back = |q: P| [q[0] / scale.x_mm_per_px, q[1] / scale.y_mm_per_px];
    Some(Measurements {
        scale: scale.clone(),
        fl_mm: fl,
        fw_mm: fw,
        area_cm2: call * to_cm2,
        toeless_area_cm2: ha + hb + hcn,
        hind_cm2: ha,
        mid_cm2: hb,
        fore_cm2: hcn,
        ai,
        csi_pct: csi,
        toe_base_method: method.to_string(),
        toeless_frac: base_frac,
        heel_centre: back(hc),
        toe_tip: back(tip),
        heel_end: back(add(hc, a, t_min)),
        toe_base: back(add(hc, a, t_base)),
        corrected: pts.is_some(),
    })
}

/// Measure a scan file. `pts` = hand-placed (heel centre, toe tip) in full-resolution pixels.
pub fn measure_scan(path: &Path, scale: &Scale, pts: Option<(P, P)>) -> Result<Measurements, String> {
    let full = image::open(path).map_err(|e| format!("cannot read scan: {e}"))?.to_rgb8();
    crate::analyze::too_small(full.width(), full.height())?;
    let img = image::imageops::resize(&full, full.width() / MEASURE_DOWN, full.height() / MEASURE_DOWN, FilterType::Triangle);
    let seg = crate::analyze::segment(&img, MEASURE_DOWN);
    measure_mask(&seg.mask, seg.w as usize, seg.h as usize, scale, MEASURE_DOWN as f32, pts)
        .ok_or_else(|| "NO_FOOT: no foot found to measure".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic right foot in foot coordinates (mm): axis x = 0, toes at y = 0, heel end at y = 250,
    /// toe base at y = 60, so the toeless length is 190 mm. Five separate toes unless `mitten`.
    fn inside(x: f32, y: f32, mitten: bool) -> bool {
        let tbl = [(60.0, -45.0, 42.0), (80.0, -48.0, 45.0), (110.0, -42.0, 42.0), (140.0, -22.0, 40.0), (170.0, -10.0, 38.0), (200.0, -14.0, 34.0), (225.0, -25.0, 33.0), (245.0, -20.0, 25.0), (250.0, -10.0, 12.0)];
        if (60.0..=250.0).contains(&y) {
            let i = tbl.windows(2).position(|w| y >= w[0].0 && y <= w[1].0).unwrap();
            let (a, b) = (tbl[i], tbl[i + 1]);
            let f = (y - a.0) / (b.0 - a.0);
            if x >= a.1 + f * (b.1 - a.1) && x <= a.2 + f * (b.2 - a.2) {
                return true;
            }
        }
        // (cx, cy, rx, ry): big toe, 2nd (longest, on the axis), 3rd, 4th, 5th
        let toes = [(-33.0, 33.0, 14.0, 30.0), (-10.0, 26.0, 8.0, 26.0), (8.0, 28.0, 8.0, 24.0), (24.0, 34.0, 7.0, 21.0), (36.0, 40.0, 6.0, 18.0)];
        // touching toes: one rounded block fills the gaps between them
        if mitten && y <= 60.0 && ((x + 2.0) / 46.0).powi(2) + ((y - 62.0) / 62.0).powi(2) <= 1.0 {
            return true;
        }
        toes.iter().any(|&(cx, cy, rx, ry)| ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 1.0)
    }

    struct Truth { fl: f32, fw: f32, ai: f32, area_cm2: f32 }

    /// Mask of the foot rotated by `deg` on a 0.25 mm canvas, plus the answers worked out directly
    /// from the un-rotated foot (rows are the axis direction, so no axis estimate is involved).
    fn make(deg: f32, mitten: bool) -> (Vec<bool>, usize, usize, Truth) {
        let (res, size) = (0.25f32, 1700usize); // 425 mm square canvas
        let (s, c) = deg.to_radians().sin_cos();
        let mut mask = vec![false; size * size];
        for py in 0..size {
            for px in 0..size {
                let (dx, dy) = ((px as f32 - size as f32 / 2.0) * res, (py as f32 - size as f32 / 2.0) * res);
                let (x, y) = (c * dx + s * dy, -s * dx + c * dy + 125.0); // inverse rotation, foot centred on canvas
                mask[py * size + px] = inside(x, y, mitten);
            }
        }
        // truth from the un-rotated foot at the same resolution
        let (mut fl_lo, mut fl_hi, mut fw, mut area) = (f32::MAX, f32::MIN, 0f32, 0f32);
        let (mut a_, mut b_, mut c_) = (0f32, 0f32, 0f32);
        let mut yy = -10.0f32;
        while yy < 260.0 {
            let mut row = 0f32;
            let mut xx = -60.0f32;
            while xx < 60.0 {
                if inside(xx, yy, mitten) {
                    row += res;
                    area += res * res;
                    fl_lo = fl_lo.min(yy);
                    fl_hi = fl_hi.max(yy);
                    let (lt, base) = (190.0, 60.0);
                    if yy >= base {
                        let d = 250.0 - yy;
                        if d < lt / 3.0 { a_ += res * res } else if d < 2.0 * lt / 3.0 { b_ += res * res } else { c_ += res * res }
                    }
                }
                xx += res;
            }
            fw = fw.max(row);
            yy += res;
        }
        (mask, size, size, Truth { fl: fl_hi - fl_lo, fw, ai: b_ / (a_ + b_ + c_), area_cm2: area / 100.0 })
    }

    fn check(m: &Measurements, t: &Truth, ai_tol: f32, label: &str) {
        assert!((m.fl_mm - t.fl).abs() < 2.0, "{label}: FL {} vs {}", m.fl_mm, t.fl);
        assert!((m.fw_mm - t.fw).abs() < 3.0, "{label}: FW {} vs {}", m.fw_mm, t.fw);
        assert!((m.ai - t.ai).abs() < ai_tol, "{label}: AI {} vs {}", m.ai, t.ai);
        assert!((m.area_cm2 - t.area_cm2).abs() / t.area_cm2 < 0.02, "{label}: area {} vs {}", m.area_cm2, t.area_cm2);
    }

    #[test]
    fn matches_known_foot_at_any_rotation() {
        let sc = Scale { x_mm_per_px: 0.25, y_mm_per_px: 0.25, calibrated: false };
        for deg in [0.0, 30.0, 100.0, 200.0, 275.0] {
            let (mask, w, h, truth) = make(deg, false);
            let m = measure_mask(&mask, w, h, &sc, 1.0, None).expect("foot");
            check(&m, &truth, 0.03, &format!("rot {deg}"));
            assert_eq!(m.toe_base_method, "gap", "rot {deg}");
            assert!(!m.corrected);
        }
    }

    #[test]
    fn touching_toes_use_the_proportion_fallback() {
        let sc = Scale { x_mm_per_px: 0.25, y_mm_per_px: 0.25, calibrated: false };
        let (mask, w, h, truth) = make(40.0, true);
        let m = measure_mask(&mask, w, h, &sc, 1.0, None).expect("foot");
        assert_eq!(m.toe_base_method, "proportion");
        check(&m, &truth, 0.04, "mitten");
    }

    #[test]
    fn hand_placed_points_are_used_and_flagged() {
        let sc = Scale { x_mm_per_px: 0.25, y_mm_per_px: 0.25, calibrated: false };
        let (mask, w, h, truth) = make(0.0, false);
        let auto = measure_mask(&mask, w, h, &sc, 1.0, None).unwrap();
        // pull the toe tip a few mm sideways and the heel centre a few mm the other way
        let hc = [auto.heel_centre[0] - 12.0, auto.heel_centre[1]];
        let tp = [auto.toe_tip[0] + 12.0, auto.toe_tip[1]];
        let m = measure_mask(&mask, w, h, &sc, 1.0, Some((hc, tp))).unwrap();
        assert!(m.corrected);
        assert_eq!(m.heel_centre, hc);
        check(&m, &truth, 0.05, "hand points");
    }

    #[test]
    fn scale_changes_millimetres_not_shape() {
        let (mask, w, h, _) = make(10.0, false);
        let a = measure_mask(&mask, w, h, &Scale { x_mm_per_px: 0.25, y_mm_per_px: 0.25, calibrated: false }, 1.0, None).unwrap();
        let b = measure_mask(&mask, w, h, &Scale { x_mm_per_px: 0.255, y_mm_per_px: 0.25, calibrated: true }, 1.0, None).unwrap();
        assert!((a.ai - b.ai).abs() < 0.02);
        assert!(b.scale.calibrated && b.area_cm2 > a.area_cm2 * 1.015);
    }

    #[test]
    fn whole_pipeline_on_a_synthetic_scan_file() {
        use image::{Rgb, RgbImage};
        let dpi = 200.0f32;
        let res = 25.4 / dpi;
        let (w, h) = (1700u32, 2338u32);
        let (s, c) = 20.0f32.to_radians().sin_cos();
        let img = RgbImage::from_fn(w, h, |px, py| {
            let (dx, dy) = ((px as f32 - w as f32 / 2.0) * res, (py as f32 - h as f32 / 2.0) * res);
            let (x, y) = (c * dx + s * dy, -s * dx + c * dy + 125.0);
            let noise = ((px * 7 + py * 13) % 13) as i32 - 6;
            if inside(x, y, false) {
                Rgb([(215 + noise) as u8, (170 + noise) as u8, (150 + noise) as u8])
            } else {
                let streak = if px < 60 { 35 } else if px > w - 50 { -20 } else { 0 };
                let g = (110 + streak + noise) as u8;
                Rgb([g, g, g.saturating_add(6)])
            }
        });
        let path = std::env::temp_dir().join(format!("sf_pipeline_{}.png", std::process::id()));
        img.save(&path).unwrap();
        let (_, _, _, truth) = make(0.0, false);
        let m = measure_scan(&path, &Scale::nominal(dpi), None).expect("measurement");
        let _ = std::fs::remove_file(&path);
        check(&m, &truth, 0.03, "whole pipeline");
        assert_eq!(m.toe_base_method, "gap");
    }
}
