//! Contact map: which part of the sole is pressed on the acrylic.
//!
//! Method (2026-11-18, tuned on 9 standing scans + seated R01 through the 10 mm acrylic). Skin that
//! is pressed on the acrylic goes pale (blood pushed out), skin that is not stays pink. Sampled on
//! standing scans: green / red = 0.72-0.76 on the pressed heel, ball and outer border, 0.63-0.65 on
//! the lifted arch. The ratio does not depend on brightness, so room light and scanner shading drop
//! out. Steps, inside the foot outline only:
//!   1. green / red per pixel after light smoothing (~1 mm);
//!   2. threshold by Otsu over the foot's own pixels (came out 0.68-0.70 on every scan), kept
//!      within 0.64-0.74 so an odd scan cannot run away;
//!   3. remove anything narrower than ~5 mm (blur halo along the raised arch edge), specks smaller
//!      than 0.5 % of the foot, and thin pieces that hug the outline (a crescent behind the heel).
//!
//! Limitation, on purpose in the open: this finds skin pressed hard enough to blanch. Skin that
//! only touches the acrylic lightly stays pink and is not counted, so on a lightly loaded foot the
//! map can be smaller than an ink or podoscope footprint. To be checked with wet footprints.

use image::RgbImage;

pub const THR_MIN: f32 = 0.64;
pub const THR_MAX: f32 = 0.74;
/// Pieces narrower than this are blur halo, not contact.
const MIN_WIDTH_MM: f32 = 5.0;
/// A piece with more than this share of its pixels within EDGE_BAND_MM of the outline is a rim.
const RIM_SHARE: f32 = 0.7;
const EDGE_BAND_MM: f32 = 3.0;
/// Pieces smaller than this share of the foot are dropped.
const MIN_PIECE: f32 = 0.005;

pub struct ContactMap {
    pub mask: Vec<bool>,
    /// green / red threshold that was used
    pub thr: f32,
    /// pressed pixels as a share of the foot outline
    pub frac: f32,
}

/// `img` and `foot` are the same size; `down` = scan pixels per `img` pixel (300 dpi scan).
pub fn find(img: &RgbImage, foot: &[bool], down: u32) -> ContactMap {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let n = w * h;
    let mm_px = down.max(1) as f32 * 25.4 / 300.0;
    let px = |mm: f32| ((mm / mm_px).round() as usize).max(1);
    let foot_n = foot.iter().filter(|&&f| f).count();
    let empty = ContactMap { mask: vec![false; n], thr: 0.0, frac: 0.0 };
    if foot_n < 100 || foot.len() != n {
        return empty;
    }

    // 1. green / red, from separately smoothed channels
    let gr = green_red(img, down);

    // 2. Otsu threshold over the foot pixels
    let vals: Vec<f32> = (0..n).filter(|&i| foot[i]).map(|i| gr[i]).collect();
    let thr = otsu(&vals, 0.4, 1.0).clamp(THR_MIN, THR_MAX);
    let raw: Vec<bool> = (0..n).map(|i| foot[i] && gr[i] > thr).collect();

    // 3. clean: open (drops < ~5 mm wide), small close, then judge the pieces
    let ro = px(MIN_WIDTH_MM / 2.0);
    let m = crate::analyze::morph(&raw, w, h, ro, false);
    let m = crate::analyze::morph(&m, w, h, ro, true);
    let rc = px(1.0);
    let m = crate::analyze::morph(&crate::analyze::morph(&m, w, h, rc, true), w, h, rc, false);
    let m: Vec<bool> = (0..n).map(|i| m[i] && foot[i]).collect();
    let inner = crate::analyze::morph(foot, w, h, px(EDGE_BAND_MM), false);
    let mut mask = vec![false; n];
    for piece in pieces(&m, w, h) {
        let near_edge = piece.iter().filter(|&&p| !inner[p]).count();
        if (piece.len() as f32) < MIN_PIECE * foot_n as f32 || near_edge as f32 > RIM_SHARE * piece.len() as f32 {
            continue;
        }
        for p in piece {
            mask[p] = true;
        }
    }
    let frac = mask.iter().filter(|&&b| b).count() as f32 / foot_n as f32;
    ContactMap { mask, thr, frac }
}

/// Green / red per pixel, each channel smoothed over ~1 mm first (`down` = scan pixels per pixel).
/// Pale (pressed) skin is high, pink (lifted) skin low; also drives the graded "Peta kontak" picture.
pub fn green_red(img: &RgbImage, down: u32) -> Vec<f32> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let n = w * h;
    let rb = ((0.5 / (down.max(1) as f32 * 25.4 / 300.0)).round() as usize).max(1);
    let (mut r, mut g) = (vec![0f32; n], vec![0f32; n]);
    for (i, p) in img.pixels().enumerate() {
        r[i] = p[0] as f32;
        g[i] = p[1] as f32;
    }
    let (r, g) = (crate::analyze::box_blur(&r, w, h, rb), crate::analyze::box_blur(&g, w, h, rb));
    (0..n).map(|i| g[i] / r[i].max(1.0)).collect()
}

/// Green / red range of the graded picture: at or below GRADE_LO = dark (lifted), at or above
/// GRADE_HI = full green (pressed hard). Sampled 18 Nov: lifted arch 0.63-0.65, pressed 0.72-0.76.
pub const GRADE_LO: f32 = 0.62;
pub const GRADE_HI: f32 = 0.75;

/// Otsu's threshold of `v` on a 256-bin histogram over [lo, hi].
fn otsu(v: &[f32], lo: f32, hi: f32) -> f32 {
    let mut hist = [0f64; 256];
    for &x in v {
        hist[(((x - lo) / (hi - lo) * 255.0).clamp(0.0, 255.0)) as usize] += 1.0;
    }
    let total: f64 = hist.iter().sum();
    let sum_all: f64 = hist.iter().enumerate().map(|(i, &c)| i as f64 * c).sum();
    let (mut wb, mut sb, mut best, mut best_i) = (0f64, 0f64, -1f64, 127usize);
    for (i, &c) in hist.iter().enumerate() {
        wb += c;
        sb += i as f64 * c;
        let wf = total - wb;
        if wb == 0.0 || wf == 0.0 {
            continue;
        }
        let (mb, mf) = (sb / wb, (sum_all - sb) / wf);
        let between = wb * wf * (mb - mf).powi(2);
        if between > best {
            best = between;
            best_i = i;
        }
    }
    lo + (best_i as f32 + 0.5) / 255.0 * (hi - lo)
}

/// 4-connected pieces of a mask, as lists of pixel indices.
fn pieces(m: &[bool], w: usize, h: usize) -> Vec<Vec<usize>> {
    let mut seen = vec![false; w * h];
    let mut out = Vec::new();
    let mut stack = Vec::new();
    for s in 0..w * h {
        if !m[s] || seen[s] {
            continue;
        }
        seen[s] = true;
        stack.push(s);
        let mut piece = Vec::new();
        while let Some(p) = stack.pop() {
            piece.push(p);
            let (x, y) = (p % w, p / w);
            let mut push = |q: usize| {
                if m[q] && !seen[q] {
                    seen[q] = true;
                    stack.push(q);
                }
            };
            if x > 0 { push(p - 1); }
            if x + 1 < w { push(p + 1); }
            if y > 0 { push(p - w); }
            if y + 1 < h { push(p + w); }
        }
        out.push(piece);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    /// 120 x 300 px "foot" (an ellipse) at 0.5 mm/px (down = 6): pale heel and ball, pink arch,
    /// plus a 2 mm pale rim along one edge that must be dropped.
    fn synthetic() -> (RgbImage, Vec<bool>) {
        let (w, h) = (160u32, 340u32);
        let mut foot = vec![false; (w * h) as usize];
        let img = RgbImage::from_fn(w, h, |x, y| {
            let (dx, dy) = ((x as f32 - 80.0) / 60.0, (y as f32 - 170.0) / 150.0);
            let inside = dx * dx + dy * dy <= 1.0;
            foot[(y * w + x) as usize] = inside;
            if !inside {
                return Rgb([40, 38, 44]);
            }
            let pressed = y < 110 || y > 240; // ball and heel
            let rim = x < 26 && (130..210).contains(&y); // thin pale strip on the raised edge
            if pressed || rim { Rgb([116, 86, 75]) } else { Rgb([108, 69, 70]) }
        });
        (img, foot)
    }

    #[test]
    fn finds_pressed_parts_and_drops_the_rim() {
        let (img, foot) = synthetic();
        let c = find(&img, &foot, 6);
        assert!((THR_MIN..=THR_MAX).contains(&c.thr), "thr {}", c.thr);
        let at = |x: usize, y: usize| c.mask[y * 160 + x];
        assert!(at(80, 60) && at(80, 290), "ball and heel are contact");
        assert!(!at(80, 170), "arch is not");
        assert!(!at(24, 170), "thin rim is dropped");
        assert!(c.frac > 0.3 && c.frac < 0.8, "frac {}", c.frac);
    }

    #[test]
    fn empty_foot_gives_empty_map() {
        let img = RgbImage::from_pixel(50, 50, Rgb([40, 40, 40]));
        let c = find(&img, &vec![false; 2500], 2);
        assert_eq!(c.frac, 0.0);
        assert!(c.mask.iter().all(|&b| !b));
    }
}
