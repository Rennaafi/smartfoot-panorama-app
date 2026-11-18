//! Foot measurements from the foot mask.
//!
//! Definitions (Cavanagh & Rodgers arch index, as in the Umardani scanner studies):
//!  * Foot axis: the line from the heel centre H to the second-toe tip J. H is found 15 % of the foot
//!    length in from the heel end; J starts as the most distal point. Both can be corrected by hand.
//!  * Foot length FL: extent of the foot along the axis. Foot width FW: widest cut across the axis.
//!  * Toes are excluded for the arch index: the toeless footprint runs from the heel end to the toe
//!    base. The toe base is where the toes stop being separate (a gap in the cross-section), or, when
//!    the toes touch, 82 % of FL (marked "proportion" in the result).
//!  * The toeless length is cut into thirds: hindfoot A, midfoot B, forefoot C. AI = B / (A + B + C).
//!  * Chippaux-Smirak index = narrowest midfoot width / widest forefoot width x 100.
//!
//! Everything is computed in millimetres from the pixel size in `Scale`. Sessions use
//! `Scale::calibrated()`, measured with steel and plastic rulers on the final case on 13 Nov 2026.

use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Scans are measured on a half-size copy (0.17 mm per pixel at 300 dpi).
pub const MEASURE_DOWN: u32 = 2;
const HEEL_CENTRE_FRAC: f32 = 0.15;
/// Toe base when the toe gaps cannot be found. Was 0.75; on 17 real scans where the gaps WERE found
/// (13 Nov seated R01-R10/B/C, 18 Nov standing L1-L3/R1/R3) the toe base sat at 0.807-0.849 of FL,
/// median 0.82, so 0.75 moved the thirds whenever a scan of the same foot fell back. Through the
/// acrylic the blur often closes the gaps until the last ~6 % of FL (18 Nov R2/R4-R6).
const TOE_BASE_FALLBACK: f32 = 0.82;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Scale {
    /// mm per pixel across the image width (sensor direction) and down the image (scan direction).
    pub x_mm_per_px: f32,
    pub y_mm_per_px: f32,
    pub calibrated: bool,
}

/// Ruler calibration of the LiDE 300 in the final case (10 mm acrylic, scanner on its spacer),
/// 13 Nov 2026. Scan direction: 1 mm tick pitch of a steel and a plastic ruler (mm and inch scales,
/// lengthwise, lights off and on, under white cloth and beside a foot) = 11.83-11.84 px/mm, 300.7 dpi.
/// Sensor direction: inch marks of the steel ruler laid across (1 mm ticks are blurred out
/// there), best band 299.5 dpi, matching the 29 Oct ID-card check (299.3 dpi).
pub const CAL_DPI_SCAN: f32 = 300.7;
pub const CAL_DPI_SENSOR: f32 = 299.5;
pub const CAL_DATE: &str = "2026-11-13";

impl Scale {
    pub fn nominal(dpi: f32) -> Self {
        Scale { x_mm_per_px: 25.4 / dpi, y_mm_per_px: 25.4 / dpi, calibrated: false }
    }
    /// The ruler calibration above (x = sensor direction, y = scan direction).
    pub fn calibrated() -> Self {
        Scale { x_mm_per_px: 25.4 / CAL_DPI_SENSOR, y_mm_per_px: 25.4 / CAL_DPI_SCAN, calibrated: true }
    }
}

/// What is written into every new session file, so each session records the scale it used.
pub fn calibration_record() -> serde_json::Value {
    let s = Scale::calibrated();
    serde_json::json!({
        "date": CAL_DATE,
        "x_mm_per_px": s.x_mm_per_px,
        "y_mm_per_px": s.y_mm_per_px,
        "dpi_sensor_axis": CAL_DPI_SENSOR,
        "dpi_scan_axis": CAL_DPI_SCAN,
        "method": "steel + plastic ruler tick pitch on the acrylic (scan axis), steel ruler inch marks (sensor axis)",
    })
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
    /// Both ends of the widest cut (foot width), full-resolution pixels. Zero in older session files.
    #[serde(default)]
    pub fw_line: [[f32; 2]; 2],
    pub toe_base: [f32; 2],
    /// True when a team member moved heel centre / toe tip by hand.
    pub corrected: bool,
    /// The app's FIRST GUESS of Clarke's angle in degrees, for the operator to check (see
    /// session::PointCheck for the checked value). None = no guess possible / file before v0.2.
    #[serde(default)]
    pub clarke_deg: Option<f32>,
    /// Points behind clarke_deg, full-resolution pixels.
    #[serde(default)]
    pub clarke_pts: Option<ClarkePts>,
    /// Where the guess came from: "kontak" (contact-map construction) or "perkiraan" (rough default).
    #[serde(default)]
    pub clarke_source: Option<String>,
    /// The same measures on the contact map (pressed skin only), on the same axis and thirds.
    #[serde(default)]
    pub contact: Option<ContactMeasure>,
}

/// Clarke's angle construction: the medial tangent touches the forefoot at `fore` and the heel at
/// `heel`; `arch` is the deepest point of the arch below that tangent. The angle sits at `fore`,
/// between the tangent and the line to `arch`. Full-resolution image pixels.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClarkePts {
    pub fore: [f32; 2],
    pub heel: [f32; 2],
    pub arch: [f32; 2],
}

/// Measures of the contact map (see contact.rs). AI uses the outline's axis, heel end and toe base,
/// so it can be compared directly with the outline AI.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContactMeasure {
    /// green / red threshold used for this scan
    pub thr: f32,
    /// contact area as a share of the outline area
    pub frac: f32,
    pub area_cm2: f32,
    pub toeless_cm2: f32,
    pub hind_cm2: f32,
    pub mid_cm2: f32,
    pub fore_cm2: f32,
    pub ai: f32,
    /// None when the contact is split (no pressed skin across part of the arch) or too small.
    pub clarke_deg: Option<f32>,
    pub clarke_pts: Option<ClarkePts>,
}

/// Clarke's angle on points in the foot frame (t along the axis, s across, mm), over the toeless
/// part t_lo..t_hi. `side` = +1 or -1, the s direction of the medial (big-toe) border.
/// Returns (angle in degrees, [fore, heel, arch] as (t, s), arch depth in mm).
fn clarke(ts: &[f32], ss: &[f32], t_lo: f32, t_hi: f32, side: f32) -> Option<(f32, [P; 3], f32)> {
    let nb = (t_hi - t_lo).floor() as usize;
    if nb < 30 {
        return None;
    }
    // medial edge profile: the most medial point in each 1 mm slice
    let mut e = vec![f32::MIN; nb];
    for (&t, &s) in ts.iter().zip(ss) {
        if t >= t_lo && t < t_hi {
            let k = ((t - t_lo) as usize).min(nb - 1);
            e[k] = e[k].max(side * s);
        }
    }
    let pts: Vec<P> = (0..nb).filter(|&k| e[k] > f32::MIN).map(|k| [k as f32 + 0.5, e[k]]).collect();
    if pts.len() < 10 {
        return None;
    }
    // upper hull of the profile (monotone chain): the tangent lines from outside
    let mut hull: Vec<P> = Vec::new();
    for &p in &pts {
        while hull.len() >= 2 {
            let (a, b) = (hull[hull.len() - 2], hull[hull.len() - 1]);
            if (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]) >= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(p);
    }
    // the hull edge across the middle of the toeless foot = the medial tangent
    let mid = nb as f32 / 2.0;
    let i = hull.windows(2).position(|w| w[0][0] <= mid && w[1][0] >= mid)?;
    let (h1, f1) = (hull[i], hull[i + 1]);
    // it must really rest on heel and forefoot (first and last ~40 %), not on a bump in the arch
    if h1[0] > 0.4 * nb as f32 || f1[0] < 0.6 * nb as f32 {
        return None;
    }
    // deepest point under the tangent; an empty slice means the arch has no contact at all
    let (k0, k1) = (h1[0] as usize, f1[0] as usize);
    let slope = (f1[1] - h1[1]) / (f1[0] - h1[0]);
    // The deepest point is looked for in the middle third only: a ragged pocket of unpressed skin just
    // behind the ball (seen on 18 Nov scan L3) is not the arch.
    let (m0, m1) = (k0.max(nb / 3), k1.min(2 * nb / 3).min(nb - 1));
    let mut best = (0f32, (m0 + m1) / 2);
    for k in k0..=k1.min(nb - 1) {
        if e[k] == f32::MIN {
            return None;
        }
        let depth = (h1[1] + slope * (k as f32 + 0.5 - h1[0])) - e[k];
        if (m0..=m1).contains(&k) && depth > best.0 {
            best = (depth, k);
        }
    }
    let arch = [best.1 as f32 + 0.5, e[best.1]];
    let (u, v) = (sub(h1, f1), sub(arch, f1));
    let cos = dot(u, v) / (dot(u, u).sqrt() * dot(v, v).sqrt()).max(1e-6);
    let deg = cos.clamp(-1.0, 1.0).acos().to_degrees();
    let back = |p: P| [p[0] + t_lo, side * p[1]];
    Some((deg, [back(f1), back(h1), back(arch)], best.0))
}

/// Clarke's angle in words, the client's table (17 Nov 2026): below 31 flat, 31-41 normal, above 41
/// cavus. The 31 degree flatfoot cut-off agrees with Pita-Fernandez et al. 2015, J Epidemiol 25(2)
/// (31.5 for age 40-64, 30.5 for 65+); the 41 degree upper limit is the client's.
pub fn clarke_label(deg: f32) -> &'static str {
    if deg < 31.0 { "datar (flat foot)" } else if deg <= 41.0 { "normal" } else { "tinggi (cavus)" }
}

/// Arch index in words, Cavanagh & Rodgers 1987 (standing footprints): 0.21 or less high arch,
/// 0.26 or more flat, normal in between.
pub fn ai_label(ai: f32) -> &'static str {
    if ai <= 0.21 { "tinggi (cavus)" } else if ai < 0.26 { "normal" } else { "datar (flat foot)" }
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
    measure_masks(mask, None, w, h, scale, down, pts)
}

/// As `measure_mask`, plus the contact map's measures when `contact` (same grid) is given.
pub fn measure_masks(mask: &[bool], contact: Option<(&[bool], f32)>, w: usize, h: usize, scale: &Scale, down: f32, pts: Option<(P, P)>) -> Option<Measurements> {
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
    // where the foot is widest, for drawing the width arrow (slices are sorted above)
    let kw = ws.iter().enumerate().fold((0, f32::MIN), |m, (i, &v)| if v > m.1 { (i, v) } else { m }).0;
    let fw_line = match slices.get(kw) {
        Some(sl) if !sl.is_empty() => {
            let tw = t_min + kw as f32 + 0.5;
            [back(add(add(hc, a, tw), b, sl[0])), back(add(add(hc, a, tw), b, sl[sl.len() - 1]))]
        }
        _ => [[0.0; 2]; 2],
    };

    // Clarke's angle on the outline. The medial border is the side with the deeper arch.
    let to_img = |q: P| back(add(add(hc, a, q[0]), b, q[1]));
    let pts3 = |p: [P; 3]| ClarkePts { fore: to_img(p[0]), heel: to_img(p[1]), arch: to_img(p[2]) };
    let sides: Vec<(f32, Option<(f32, [P; 3], f32)>)> = [1.0f32, -1.0].iter().map(|&sd| (sd, clarke(&ts, &ss, t_min, t_base, sd))).collect();
    let medial = sides.iter().filter_map(|(sd, r)| r.map(|r| (*sd, r))).fold(None, |m: Option<(f32, (f32, [P; 3], f32))>, x| match m {
        Some(y) if y.1 .2 >= x.1 .2 => Some(y),
        _ => Some(x),
    });
    let side = medial.map(|m| m.0).unwrap_or(1.0);

    // the contact map on the same frame and thirds
    let contact = contact.map(|(cm, thr)| {
        let (mut ct, mut cs) = (Vec::new(), Vec::new());
        for y in 0..h {
            for x in 0..w {
                if cm[y * w + x] {
                    let d = sub([(x as f32 + 0.5) * fx, (y as f32 + 0.5) * fy], hc);
                    ct.push(dot(d, a));
                    cs.push(dot(d, b));
                }
            }
        }
        let (mut ka, mut kb, mut kc) = (0f32, 0f32, 0f32);
        for &t in &ct {
            if t < t_min + lt / 3.0 { ka += 1.0 } else if t < t_min + 2.0 * lt / 3.0 { kb += 1.0 } else if t < t_base { kc += 1.0 }
        }
        let (ha, hb, hcn) = (ka * to_cm2, kb * to_cm2, kc * to_cm2);
        let cl = clarke(&ct, &cs, t_min, t_base, side);
        ContactMeasure {
            thr,
            frac: ct.len() as f32 / call.max(1.0),
            area_cm2: ct.len() as f32 * to_cm2,
            toeless_cm2: ha + hb + hcn,
            hind_cm2: ha,
            mid_cm2: hb,
            fore_cm2: hcn,
            ai: if ha + hb + hcn > 0.0 { hb / (ha + hb + hcn) } else { 0.0 },
            clarke_deg: cl.map(|c| c.0),
            clarke_pts: cl.map(|c| pts3(c.1)),
        }
    });

    // The app's first guess for the operator (2026-11-18 agreement): the contact-map construction
    // when there is one; otherwise a rough default on the outline's medial tangent with the arch point
    // in the middle of the toeless foot, halfway between the tangent and the foot axis. The outline's
    // own Clarke (5-9 degrees on every real foot, the blurred outline includes the lifted arch) is
    // never reported.
    let guess: Option<(f32, ClarkePts, &str)> = match contact.as_ref().and_then(|c| c.clarke_pts.clone()) {
        Some(p) => Some((clarke_from_points(&p, scale), p, "kontak")),
        None => medial.map(|(_, (_, p3, _))| {
            let (fore, heel) = (p3[0], p3[1]);
            let tm = t_min + lt / 2.0;
            let s_tan = heel[1] + (fore[1] - heel[1]) * (tm - heel[0]) / (fore[0] - heel[0]).max(1e-3);
            let p = pts3([fore, heel, [tm, s_tan / 2.0]]);
            (clarke_from_points(&p, scale), p, "perkiraan")
        }),
    };

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
        fw_line,
        corrected: pts.is_some(),
        clarke_deg: guess.as_ref().map(|g| g.0),
        clarke_pts: guess.as_ref().map(|g| g.1.clone()),
        clarke_source: guess.map(|g| g.2.to_string()),
        contact,
    })
}

/// Clarke's angle from three points (full-resolution pixels): the angle at `fore` between the line
/// to `heel` (the medial tangent) and the line to `arch`. Distances in mm, so the two pixel sizes count.
pub fn clarke_from_points(p: &ClarkePts, scale: &Scale) -> f32 {
    let mm = |q: [f32; 2]| [q[0] * scale.x_mm_per_px, q[1] * scale.y_mm_per_px];
    let (f, h, a) = (mm(p.fore), mm(p.heel), mm(p.arch));
    let (u, v) = (sub(h, f), sub(a, f));
    let cos = dot(u, v) / (dot(u, u).sqrt() * dot(v, v).sqrt()).max(1e-6);
    cos.clamp(-1.0, 1.0).acos().to_degrees()
}

/// Measure a scan file. `pts` = hand-placed (heel centre, toe tip) in full-resolution pixels.
pub fn measure_scan(path: &Path, scale: &Scale, pts: Option<(P, P)>) -> Result<Measurements, String> {
    let full = image::open(path).map_err(|e| format!("cannot read scan: {e}"))?.to_rgb8();
    crate::analyze::too_small(full.width(), full.height())?;
    let img = image::imageops::resize(&full, full.width() / MEASURE_DOWN, full.height() / MEASURE_DOWN, FilterType::Triangle);
    let seg = crate::analyze::segment(&img, MEASURE_DOWN);
    let cm = crate::contact::find(&img, &seg.mask, MEASURE_DOWN);
    let contact = if cm.frac > 0.0 { Some((cm.mask.as_slice(), cm.thr)) } else { None };
    measure_masks(&seg.mask, contact, seg.w as usize, seg.h as usize, scale, MEASURE_DOWN as f32, pts)
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
    fn width_line_is_as_long_as_the_foot_width() {
        // the review screen draws the width arrow from fw_line, so it must match fw_mm
        let sc = Scale { x_mm_per_px: 0.25, y_mm_per_px: 0.25, calibrated: false };
        for deg in [0.0, 30.0, 100.0] {
            let (mask, w, h, _) = make(deg, false);
            let m = measure_mask(&mask, w, h, &sc, 1.0, None).unwrap();
            let [p, q] = m.fw_line;
            let len = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt() * 0.25;
            assert!((len - m.fw_mm).abs() < 0.04 * m.fw_mm, "rot {deg}: line {len} vs fw {}", m.fw_mm);
        }
    }

    /// Medial edge made of straight pieces: tangent rests on t = 50 (heel) and t = 150 (forefoot) at
    /// s = 40, arch dips to s = 20 at t = 100, so Clarke = atan(20 / 50) = 21.8 degrees.
    fn clarke_points(gap: bool) -> (Vec<f32>, Vec<f32>) {
        let e = |t: f32| if t < 50.0 { 30.0 + t / 5.0 } else if t < 100.0 { 40.0 - (t - 50.0) * 0.4 } else if t < 150.0 { 20.0 + (t - 100.0) * 0.4 } else { 40.0 - (t - 150.0) / 5.0 };
        let (mut ts, mut ss) = (vec![], vec![]);
        let mut t = 0.0f32;
        while t < 200.0 {
            if !(gap && (90.0..110.0).contains(&t)) {
                let mut s = -30.0f32;
                while s <= e(t) {
                    ts.push(t);
                    ss.push(s);
                    s += 0.5;
                }
            }
            t += 0.25;
        }
        (ts, ss)
    }

    #[test]
    fn clarke_angle_of_a_known_shape() {
        let (ts, ss) = clarke_points(false);
        let (deg, p, depth) = clarke(&ts, &ss, 0.0, 200.0, 1.0).expect("clarke");
        assert!((deg - 21.8).abs() < 1.0, "angle {deg}");
        assert!((depth - 20.0).abs() < 1.0, "depth {depth}");
        assert!((p[0][0] - 150.0).abs() < 2.0 && (p[1][0] - 50.0).abs() < 2.0 && (p[2][0] - 100.0).abs() < 2.0, "{p:?}");
        // mirrored foot, other side: the same angle
        let neg: Vec<f32> = ss.iter().map(|s| -s).collect();
        let (deg2, _, _) = clarke(&ts, &neg, 0.0, 200.0, -1.0).expect("mirrored");
        assert!((deg2 - deg).abs() < 0.01);
    }

    #[test]
    fn split_contact_has_no_clarke_angle() {
        let (ts, ss) = clarke_points(true);
        assert!(clarke(&ts, &ss, 0.0, 200.0, 1.0).is_none());
    }

    #[test]
    fn contact_ai_uses_the_outline_thirds() {
        // contact = the whole outline: same AI; contact = outline minus the middle third: AI 0
        let sc = Scale { x_mm_per_px: 0.25, y_mm_per_px: 0.25, calibrated: false };
        let (mask, w, h, _) = make(0.0, false);
        let m = measure_masks(&mask, Some((&mask, 0.7)), w, h, &sc, 1.0, None).unwrap();
        let c = m.contact.as_ref().unwrap();
        assert!((c.ai - m.ai).abs() < 1e-4 && (c.frac - 1.0).abs() < 1e-4);
        let (y0, y1) = (m.heel_end[1].min(m.toe_base[1]), m.heel_end[1].max(m.toe_base[1]));
        let (a, b) = (y0 + (y1 - y0) / 3.0, y0 + 2.0 * (y1 - y0) / 3.0);
        let cut: Vec<bool> = mask.iter().enumerate().map(|(i, &v)| v && !((a + 4.0..b - 4.0).contains(&((i / w) as f32)))).collect();
        let m2 = measure_masks(&mask, Some((&cut, 0.7)), w, h, &sc, 1.0, None).unwrap();
        assert!(m2.contact.unwrap().ai < 0.03);
    }

    #[test]
    fn calibrated_scale_is_the_ruler_result() {
        let s = Scale::calibrated();
        assert!(s.calibrated);
        assert!((25.4 / s.y_mm_per_px - 300.7).abs() < 0.01 && (25.4 / s.x_mm_per_px - 299.5).abs() < 0.01);
        assert_eq!(calibration_record()["date"], "2026-11-13");
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
