//! Finds the foot in a scan and draws its outline for the review screen.
//!
//! Method (provisional, tuned on the 2026-11 hand scans): work on a 1/4-size copy, estimate the
//! background colour per column from the top and bottom strips (room-light streaks run along
//! columns, so a per-column estimate follows them), score every pixel by its colour distance
//! from that background, split with Otsu's threshold, clean the mask, keep the largest blob.

use image::{imageops::FilterType, Rgb, RgbImage};
use std::path::Path;

pub struct Analysis {
    pub preview_png: Vec<u8>,
    pub found: bool,
    pub touches_edge: bool,
    /// A single narrow stretch touches one edge: an ankle or wrist coming in, not a cut-off foot.
    pub limb_at_edge: bool,
    pub clear: bool,
    pub area_frac: f32,
    /// The thing found is not foot-shaped (too small/large or too elongated/round: a ruler, a card,
    /// a hand held sideways...). A warning only: the operator may still accept. Thresholds are
    /// provisional until real feet have been scanned on the final casing.
    pub odd_shape: bool,
    /// A second large object lies on the glass (e.g. both feet): only the largest one is outlined.
    pub extra_objects: bool,
    pub bbox: Option<[u32; 4]>, // x0,y0,x1,y1 in preview pixels
    pub preview_w: u32,
    pub preview_h: u32,
}

const DOWN: u32 = 4;
/// Widest stretch on one edge still treated as a limb entering (ankle/wrist), in mm.
const LIMB_MAX_MM: f32 = 80.0;
/// Pixels above this fraction of the Otsu threshold count as foot when connected to a strong pixel.
const WEAK_FRAC: f32 = 0.55;
const STRIP: u32 = 20; // rows used for the background estimate (preview pixels)
/// Smallest scan (full-resolution pixels per side) worth analysing; anything smaller is a broken scan.
pub const MIN_SCAN_PX: u32 = 160;

pub fn too_small(w: u32, h: u32) -> Result<(), String> {
    if w < MIN_SCAN_PX || h < MIN_SCAN_PX {
        Err(format!("BAD_IMAGE: the scan is only {w}x{h} pixels, too small to be a real scan"))
    } else {
        Ok(())
    }
}

pub fn analyze(path: &Path) -> Result<Analysis, String> {
    let full = image::open(path).map_err(|e| format!("cannot read scan: {e}"))?.to_rgb8();
    too_small(full.width(), full.height())?;
    let img = image::imageops::resize(&full, full.width() / DOWN, full.height() / DOWN, FilterType::Triangle);
    Ok(analyze_image(&img))
}

/// Result of finding the foot: the cleaned mask plus the numbers behind it.
pub struct Seg {
    pub mask: Vec<bool>,
    pub score: Vec<u8>,
    pub thr: u8,
    pub w: u32,
    pub h: u32,
    /// Area of the objects dropped by "keep the largest blob", as a fraction of the kept one.
    pub extra_frac: f32,
}

/// `down` = how many scan pixels one pixel of `img` covers; sizes below were tuned at 4.
pub fn segment(img: &RgbImage, down: u32) -> Seg {
    let k = (4 / down.max(1)).max(1);
    let (w, h) = img.dimensions();
    let (wu, hu) = (w as usize, h as usize);

    // 1. per-column background colour: median of the top strip and of the bottom strip; where the
    //    foot reaches one end, take whichever strip is closer to the overall background colour
    let med = |rows: std::ops::Range<u32>, x: usize| -> [f32; 3] {
        let mut ch: [Vec<u8>; 3] = [vec![], vec![], vec![]];
        for y in rows {
            let p = img.get_pixel(x as u32, y);
            for c in 0..3 {
                ch[c].push(p[c]);
            }
        }
        let mut m = [0f32; 3];
        for c in 0..3 {
            ch[c].sort_unstable();
            m[c] = ch[c][ch[c].len() / 2] as f32;
        }
        m
    };
    let top: Vec<[f32; 3]> = (0..wu).map(|x| med(0..(STRIP * k).min(h), x)).collect();
    let bot: Vec<[f32; 3]> = (0..wu).map(|x| med(h.saturating_sub(STRIP * k)..h, x)).collect();
    let mut all: [Vec<f32>; 3] = [vec![], vec![], vec![]];
    for x in 0..wu {
        for c in 0..3 {
            all[c].push(top[x][c]);
            all[c].push(bot[x][c]);
        }
    }
    let mut g = [0f32; 3];
    for c in 0..3 {
        all[c].sort_by(|a, b| a.partial_cmp(b).unwrap());
        g[c] = all[c][all[c].len() / 2];
    }
    let dist = |a: &[f32; 3]| (a[0] - g[0]).powi(2) + (a[1] - g[1]).powi(2) + (a[2] - g[2]).powi(2);
    let bg: Vec<[f32; 3]> = (0..wu).map(|x| if dist(&top[x]) <= dist(&bot[x]) { top[x] } else { bot[x] }).collect();
    let bg: Vec<[f32; 3]> = (0..wu)
        .map(|x| {
            let (a, b) = (x.saturating_sub(6 * k as usize), (x + 6 * k as usize).min(wu - 1));
            let mut s = [0f32; 3];
            for i in a..=b {
                for c in 0..3 {
                    s[c] += bg[i][c];
                }
            }
            let n = (b - a + 1) as f32;
            [s[0] / n, s[1] / n, s[2] / n]
        })
        .collect();

    // 2. colour-distance score per pixel
    let mut score = vec![0u8; wu * hu];
    for y in 0..hu {
        for x in 0..wu {
            let p = img.get_pixel(x as u32, y as u32);
            let b = bg[x];
            let d = ((p[0] as f32 - b[0]).powi(2) + (p[1] as f32 - b[1]).powi(2) + (p[2] as f32 - b[2]).powi(2)).sqrt();
            score[y * wu + x] = d.min(255.0) as u8;
        }
    }

    // 3. Otsu threshold
    let thr = otsu(&score);
    // hysteresis: strong pixels (above Otsu) grow into weaker neighbours, so soft edges such as a raised
    // thumb or the lifted arch stay attached without picking up lone background noise
    let mut mask = hysteresis(&score, wu, hu, thr, (thr as f32 * WEAK_FRAC) as u8);

    // 4. clean up: open (remove specks), close (fill gaps), largest blob, fill holes
    let r2 = 2 * k as usize;
    // closing radius ~0.6 mm: seals pinholes but not the real gaps between toes
    let r3 = ((0.6 / (down.max(1) as f32 * 25.4 / 300.0)).round() as usize).max(1);
    mask = morph(&mask, wu, hu, r2, false);
    mask = morph(&mask, wu, hu, r2, true);
    mask = morph(&mask, wu, hu, r3, true);
    mask = morph(&mask, wu, hu, r3, false);
    let before = mask.iter().filter(|&&m| m).count();
    mask = largest_blob(&mask, wu, hu);
    let kept = mask.iter().filter(|&&m| m).count();
    let extra_frac = if kept > 0 { before.saturating_sub(kept) as f32 / kept as f32 } else { 0.0 };
    // only small holes (< 2 cm^2) are filled; a big pocket between toes must stay open
    let max_hole_px = (200.0 / (down.max(1) as f32 * 25.4 / 300.0).powi(2)) as usize;
    mask = fill_holes(&mask, wu, hu, max_hole_px);
    Seg { mask, score, thr, w, h, extra_frac }
}

pub fn analyze_image(img: &RgbImage) -> Analysis {
    let Seg { mask, score, thr, w, h, extra_frac } = segment(img, DOWN);
    let (wu, hu) = (w as usize, h as usize);

    let count = mask.iter().filter(|&&m| m).count();
    let area_frac = count as f32 / (wu * hu) as f32;
    let found = thr >= 8 && area_frac > 0.02 && area_frac < 0.85;

    // 5. edge contact and separation quality
    // contact with each edge: pixel count and the stretch it covers (left, right, top, bottom)
    let mut cnt = [0u32; 4];
    let mut lo = [u32::MAX; 4];
    let mut hi = [0u32; 4];
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0u32, 0u32);
    for y in 0..hu {
        for x in 0..wu {
            if mask[y * wu + x] {
                x0 = x0.min(x as u32);
                y0 = y0.min(y as u32);
                x1 = x1.max(x as u32);
                y1 = y1.max(y as u32);
                for (i, hit, along) in [(0, x <= 1, y), (1, x + 2 >= wu, y), (2, y <= 1, x), (3, y + 2 >= hu, x)] {
                    if hit {
                        cnt[i] += 1;
                        lo[i] = lo[i].min(along as u32);
                        hi[i] = hi[i].max(along as u32);
                    }
                }
            }
        }
    }
    let px_per_mm = 300.0 / 25.4 / DOWN as f32;
    let touched: Vec<usize> = (0..4).filter(|&i| cnt[i] >= 5).collect();
    let narrow_single = touched.len() == 1 && ((hi[touched[0]] - lo[touched[0]]) as f32 / px_per_mm) <= LIMB_MAX_MM;
    let touches_edge = !touched.is_empty() && !narrow_single;
    let limb_at_edge = narrow_single;
    let (mut fs, mut fn_, mut bs, mut bn) = (0f64, 0f64, 0f64, 0f64);
    for i in 0..wu * hu {
        if mask[i] {
            fs += score[i] as f64;
            fn_ += 1.0;
        } else {
            bs += score[i] as f64;
            bn += 1.0;
        }
    }
    let (fm, bm) = (fs / fn_.max(1.0), bs / bn.max(1.0));
    let clear = found && fm >= 1.8 * thr as f64 && bm <= 0.6 * thr as f64;

    // shape sanity: principal axes of the mask. A foot is about 2.2-3.2 times longer than wide (by
    // spread) and 150-330 mm long; a ruler, card or speck is far outside that.
    let (mut sx, mut sy, mut sxx, mut syy, mut sxy) = (0f64, 0f64, 0f64, 0f64, 0f64);
    for y in 0..hu {
        for x in 0..wu {
            if mask[y * wu + x] {
                let (xf, yf) = (x as f64, y as f64);
                sx += xf; sy += yf; sxx += xf * xf; syy += yf * yf; sxy += xf * yf;
            }
        }
    }
    let n = (count as f64).max(1.0);
    let (mx, my) = (sx / n, sy / n);
    let (cxx, cyy, cxy) = (sxx / n - mx * mx, syy / n - my * my, sxy / n - mx * my);
    let (tr, det) = (cxx + cyy, cxx * cyy - cxy * cxy);
    let disc = ((tr * tr / 4.0) - det).max(0.0).sqrt();
    let (l1, l2) = (tr / 2.0 + disc, (tr / 2.0 - disc).max(1e-9));
    let major_mm = 4.0 * l1.sqrt() / px_per_mm as f64;
    let elong = (l1 / l2).sqrt();
    let odd_shape = found && (major_mm < 90.0 || major_mm > 420.0 || elong < 1.6 || elong > 5.0);
    let extra_objects = found && extra_frac > 0.25;

    // 6. preview with the outline drawn in blue
    let mut prev = img.clone();
    for y in 1..hu - 1 {
        for x in 1..wu - 1 {
            if mask[y * wu + x] && !(mask[y * wu + x - 1] && mask[y * wu + x + 1] && mask[(y - 1) * wu + x] && mask[(y + 1) * wu + x]) {
                for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
                    let (px, py) = ((x as i32 + dx) as u32, (y as i32 + dy) as u32);
                    if px < w && py < h {
                        prev.put_pixel(px, py, Rgb([31, 95, 191]));
                    }
                }
            }
        }
    }
    let mut png = Vec::new();
    prev.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).expect("png encode");

    Analysis {
        preview_png: png,
        found,
        touches_edge,
        limb_at_edge,
        clear,
        area_frac,
        odd_shape,
        extra_objects,
        bbox: if count > 0 { Some([x0, y0, x1, y1]) } else { None },
        preview_w: w,
        preview_h: h,
    }
}

fn hysteresis(score: &[u8], w: usize, h: usize, strong: u8, weak: u8) -> Vec<bool> {
    let mut on = vec![false; w * h];
    let mut stack: Vec<usize> = (0..w * h).filter(|&i| score[i] > strong).collect();
    for &i in &stack {
        on[i] = true;
    }
    while let Some(p) = stack.pop() {
        let (x, y) = (p % w, p / w);
        let mut push = |q: usize| {
            if !on[q] && score[q] > weak {
                on[q] = true;
                stack.push(q);
            }
        };
        if x > 0 { push(p - 1); }
        if x + 1 < w { push(p + 1); }
        if y > 0 { push(p - w); }
        if y + 1 < h { push(p + w); }
    }
    on
}

fn otsu(score: &[u8]) -> u8 {
    let mut hist = [0u64; 256];
    for &s in score {
        hist[s as usize] += 1;
    }
    let total = score.len() as f64;
    let sum_all: f64 = hist.iter().enumerate().map(|(i, &c)| i as f64 * c as f64).sum();
    let (mut sum_b, mut w_b, mut best, mut thr) = (0f64, 0f64, 0f64, 0u8);
    for t in 0..256 {
        w_b += hist[t] as f64;
        if w_b == 0.0 {
            continue;
        }
        let w_f = total - w_b;
        if w_f == 0.0 {
            break;
        }
        sum_b += t as f64 * hist[t] as f64;
        let (m_b, m_f) = (sum_b / w_b, (sum_all - sum_b) / w_f);
        let between = w_b * w_f * (m_b - m_f).powi(2);
        if between > best {
            best = between;
            thr = t as u8;
        }
    }
    thr
}

/// Square dilation (`dilate = true`) or erosion by radius `r`, using running sums.
fn morph(m: &[bool], w: usize, h: usize, r: usize, dilate: bool) -> Vec<bool> {
    let src: Vec<u8> = m.iter().map(|&b| if b != dilate { 0 } else { 1 }).collect(); // erosion works on the inverse
    let mut tmp = vec![0u32; w * h];
    for y in 0..h {
        let mut pre = vec![0u32; w + 1];
        for x in 0..w {
            pre[x + 1] = pre[x] + src[y * w + x] as u32;
        }
        for x in 0..w {
            let (a, b) = (x.saturating_sub(r), (x + r + 1).min(w));
            tmp[y * w + x] = pre[b] - pre[a];
        }
    }
    let mut out = vec![false; w * h];
    for x in 0..w {
        let mut pre = vec![0u32; h + 1];
        for y in 0..h {
            pre[y + 1] = pre[y] + tmp[y * w + x];
        }
        for y in 0..h {
            let (a, b) = (y.saturating_sub(r), (y + r + 1).min(h));
            let any = pre[b] - pre[a] > 0;
            out[y * w + x] = if dilate { any } else { !any };
        }
    }
    out
}

fn largest_blob(m: &[bool], w: usize, h: usize) -> Vec<bool> {
    let mut label = vec![0u32; w * h];
    let (mut best_id, mut best_n, mut id) = (0u32, 0usize, 0u32);
    let mut stack = Vec::new();
    for s in 0..w * h {
        if !m[s] || label[s] != 0 {
            continue;
        }
        id += 1;
        let mut n = 0;
        label[s] = id;
        stack.push(s);
        while let Some(p) = stack.pop() {
            n += 1;
            let (x, y) = (p % w, p / w);
            let mut push = |q: usize| {
                if m[q] && label[q] == 0 {
                    label[q] = id;
                    stack.push(q);
                }
            };
            if x > 0 { push(p - 1); }
            if x + 1 < w { push(p + 1); }
            if y > 0 { push(p - w); }
            if y + 1 < h { push(p + w); }
        }
        if n > best_n {
            best_n = n;
            best_id = id;
        }
    }
    label.iter().map(|&l| l == best_id && best_id != 0).collect()
}

fn fill_holes(m: &[bool], w: usize, h: usize, max_px: usize) -> Vec<bool> {
    let mut outside = vec![false; w * h];
    let mut stack = Vec::new();
    for x in 0..w {
        for y in [0, h - 1] {
            if !m[y * w + x] && !outside[y * w + x] { outside[y * w + x] = true; stack.push(y * w + x); }
        }
    }
    for y in 0..h {
        for x in [0, w - 1] {
            if !m[y * w + x] && !outside[y * w + x] { outside[y * w + x] = true; stack.push(y * w + x); }
        }
    }
    while let Some(p) = stack.pop() {
        let (x, y) = (p % w, p / w);
        let mut push = |q: usize| {
            if !m[q] && !outside[q] {
                outside[q] = true;
                stack.push(q);
            }
        };
        if x > 0 { push(p - 1); }
        if x + 1 < w { push(p + 1); }
        if y > 0 { push(p - w); }
        if y + 1 < h { push(p + w); }
    }
    // holes = background not reachable from the border; fill only the small ones
    let mut out = m.to_vec();
    let mut seen = vec![false; w * h];
    for s in 0..w * h {
        if m[s] || outside[s] || seen[s] {
            continue;
        }
        let mut comp = vec![s];
        seen[s] = true;
        let mut i = 0;
        while i < comp.len() {
            let p = comp[i];
            i += 1;
            let (x, y) = (p % w, p / w);
            let mut push = |q: usize| {
                if !m[q] && !outside[q] && !seen[q] {
                    seen[q] = true;
                    comp.push(q);
                }
            };
            if x > 0 { push(p - 1); }
            if x + 1 < w { push(p + 1); }
            if y > 0 { push(p - w); }
            if y + 1 < h { push(p + w); }
        }
        if comp.len() <= max_px {
            for &q in &comp {
                out[q] = true;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Grey background with vertical streaks and a bright ellipse in the middle.
    fn synthetic(touch_edge: bool) -> RgbImage {
        let (w, h) = (160u32, 220u32);
        RgbImage::from_fn(w, h, |x, y| {
            let streak = if x < 12 { 40 } else { 0 };
            let bg = 110 + streak;
            let (cx, cy) = (if touch_edge { 80.0 } else { 80.0 }, if touch_edge { 20.0 } else { 110.0 });
            let d = ((x as f32 - cx) / 30.0).powi(2) + ((y as f32 - cy) / 70.0).powi(2);
            if d < 1.0 { Rgb([215, 170, 150]) } else { Rgb([bg as u8, bg as u8, (bg + 6) as u8]) }
        })
    }

    #[test]
    fn finds_ellipse_despite_streaks() {
        let a = analyze_image(&synthetic(false));
        assert!(a.found && !a.touches_edge && a.clear, "{:?} {} {}", a.bbox, a.found, a.area_frac);
        let [x0, y0, x1, y1] = a.bbox.unwrap();
        assert!((48..=52).contains(&x0) && (108..=112).contains(&x1) && (38..=42).contains(&y0) && (178..=182).contains(&y1), "{:?}", a.bbox);
    }

    /// Sensor-like background (small deterministic noise) with skin-coloured pixels where `fg` is true.
    fn scene(w: u32, h: u32, fg: impl Fn(f32, f32) -> bool) -> RgbImage {
        RgbImage::from_fn(w, h, |x, y| {
            if fg(x as f32, y as f32) {
                Rgb([215, 170, 150])
            } else {
                let n = ((x * 7 + y * 13) % 13) as i32 - 6;
                Rgb([(110 + n) as u8, (110 + n) as u8, (116 + n) as u8])
            }
        })
    }

    #[test]
    fn flags_edge_contact() {
        // a wide shape (about 350 px = 120 mm) cut off by the top edge is a cut-off foot
        let a = analyze_image(&scene(700, 900, |x, y| ((x - 350.0) / 180.0).powi(2) + ((y - 60.0) / 300.0).powi(2) < 1.0));
        assert!(a.touches_edge && !a.limb_at_edge);
    }

    #[test]
    fn narrow_limb_entering_is_not_a_cutoff() {
        // a forearm about 100 px (34 mm) wide running off the right edge
        let a = analyze_image(&scene(700, 900, |x, y| {
            ((x - 300.0) / 150.0).powi(2) + ((y - 450.0) / 200.0).powi(2) < 1.0 || (x > 300.0 && (400.0..500.0).contains(&y))
        }));
        assert!(a.found && a.limb_at_edge && !a.touches_edge, "found={} limb={} edge={}", a.found, a.limb_at_edge, a.touches_edge);
    }

    #[test]
    fn blank_scan_finds_nothing() {
        let img = RgbImage::from_pixel(160, 220, Rgb([120, 120, 125]));
        assert!(!analyze_image(&img).found);
    }
}
