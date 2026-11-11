//! Finds the foot in a scan and draws its outline for the review screen.
//!
//! Method (2026-11-11, tuned on the first scans through the case's 10 mm acrylic plus the 2026-11-03
//! bare-glass reference scans). The sole sits ~10 mm above the scanner's focus, so it is blurred
//! (edge 10-90 % = 5-6 mm across the sensor, 1.5-2 mm along the scan) and often DARKER than the
//! background, while the parts pressed on the acrylic turn pale. Plain colour distance found only
//! the pale rim. What separates skin from the grey background in every scan is its red tint, so:
//!   1. two maps on a 1/4-size copy, lightly smoothed: chroma (R - G) and brightness;
//!   2. background of each map = smooth 2-D cubic surface (room light) + per-column offset (sensor
//!      pattern), fitted only on pixels not yet believed to be skin; repeated 3 times;
//!   3. seeds = clearly red pixels; outline at half their typical redness (= the 50 % point of a
//!      blurred edge, which keeps sizes right: card test +0.6 % / +1.1 %);
//!   4. pale pressed skin (brighter than the background) joins only where it touches red skin, so
//!      glare and smudges alone never make a foot.
//! An empty case, lights on or off, gives no red seeds at all (99th percentile 2-3 vs seeds > 10).

use image::{imageops::FilterType, Rgb, RgbImage};
use std::path::Path;

pub struct Analysis {
    pub preview_png: Vec<u8>,
    pub found: bool,
    pub touches_edge: bool,
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
/// Redness above the background (R - G units) that is surely skin. Empty scans: 99th pct 2-3.
const SEED: f32 = 10.0;
/// Typical redness of the seeds below this = nothing skin-like on the glass. Feet measured 18-21.
const MIN_LEVEL: f32 = 8.0;
/// Brightness above the background that marks a pixel as "maybe an object" while fitting the background.
const BRIGHT_EXCL: f32 = 25.0;
/// Contact with an image edge closer than this counts as touching it (the outermost scan rows are a
/// dark strip about 1 mm wide, so a foot running off the glass stops short of the last pixel).
const EDGE_MARGIN_MM: f32 = 2.0;
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
    /// Redness above the background per pixel, x4 and clipped to 0-255 (for diagnostics/previews).
    pub score: Vec<u8>,
    /// Redness threshold used for the outline (R - G units).
    pub thr: f32,
    /// Typical redness of the seed pixels; 0 when there were no seeds.
    pub level: f32,
    /// Brightness threshold for pale pressed skin joining the outline.
    pub pale_thr: f32,
    pub w: u32,
    pub h: u32,
    /// Area of the objects dropped by "keep the largest blob", as a fraction of the kept one.
    pub extra_frac: f32,
}

/// `down` = how many scan pixels one pixel of `img` covers; sizes below were tuned at 4.
pub fn segment(img: &RgbImage, down: u32) -> Seg {
    let k = (4 / down.max(1)).max(1) as usize;
    let (w, h) = img.dimensions();
    let (wu, hu) = (w as usize, h as usize);
    let n = wu * hu;
    let empty = |score: Vec<u8>| Seg { mask: vec![false; n], score, thr: 0.0, level: 0.0, pale_thr: 0.0, w, h, extra_frac: 0.0 };
    if wu < 8 || hu < 8 {
        return empty(vec![0; n]);
    }

    // 1. chroma and brightness maps, smoothed over ~1.7 mm (box 5 px at 1/4 scale)
    let mut rg = vec![0f32; n];
    let mut lum = vec![0f32; n];
    for (i, p) in img.pixels().enumerate() {
        let (r, g, b) = (p[0] as f32, p[1] as f32, p[2] as f32);
        rg[i] = r - g;
        lum[i] = (r + g + b) / 3.0;
    }
    let rg = box_blur(&rg, wu, hu, 2 * k);
    let lum = box_blur(&lum, wu, hu, 2 * k);

    // 2. background = cubic surface + column offsets, fitted on pixels not (yet) taken for an object.
    //    Start by excluding whatever is clearly redder than the image as a whole: a foot can fill
    //    most of a column, and a column median taken over the foot would subtract the foot itself.
    let rg_med = percentile(rg.clone(), 0.5);
    let start: Vec<bool> = (0..n).map(|i| rg[i] - rg_med > SEED / 2.0).collect();
    let mut excl = morph(&start, wu, hu, 10 * k, true);
    let (mut d, mut e) = (vec![0f32; n], vec![0f32; n]);
    for _ in 0..3 {
        let bd = background(&rg, &excl, wu, hu);
        let be = background(&lum, &excl, wu, hu);
        for i in 0..n {
            d[i] = rg[i] - bd[i];
            e[i] = lum[i] - be[i];
        }
        let obj: Vec<bool> = (0..n).map(|i| d[i] > SEED / 2.0 || e[i] > BRIGHT_EXCL).collect();
        excl = morph(&obj, wu, hu, 10 * k, true);
    }
    let score: Vec<u8> = d.iter().map(|&v| (v * 4.0).clamp(0.0, 255.0) as u8).collect();

    // 3. seeds: surely-red skin (at least ~2 cm^2 of it)
    let r2 = 2 * k;
    let red: Vec<bool> = d.iter().map(|&v| v > SEED).collect();
    let seeds = morph(&morph(&red, wu, hu, r2, false), wu, hu, r2, true);
    let seed_vals: Vec<f32> = (0..n).filter(|&i| seeds[i]).map(|i| d[i]).collect();
    let min_seed_px = (200.0 / (down.max(1) as f32 * 25.4 / 300.0).powi(2)) as usize;
    if seed_vals.len() < min_seed_px.max(4) {
        return empty(score);
    }
    let level = percentile(seed_vals, 0.60);
    let thr = (level / 2.0).max(SEED * 0.6);

    // 4. pale pressed skin: brighter than the background, judged against the noise away from the foot
    let near = morph(&seeds, wu, hu, 15 * k, true);
    let far_e: Vec<f32> = (0..n).filter(|&i| !near[i]).map(|i| e[i]).collect();
    let sig = if far_e.len() > 100 {
        let m = percentile(far_e.clone(), 0.5);
        1.4826 * percentile(far_e.iter().map(|v| (v - m).abs()).collect(), 0.5)
    } else {
        0.0
    };
    let near_e: Vec<f32> = (0..n).filter(|&i| near[i]).map(|i| e[i]).collect();
    let pale_thr = (percentile(near_e, 0.95) / 2.0).max(6.0 * sig).max(15.0);

    // 5. outline = allowed pixels (red enough, or pale enough) connected to a seed
    let allowed: Vec<bool> = (0..n).map(|i| d[i] > thr || e[i] > pale_thr).collect();
    let allowed = morph(&morph(&allowed, wu, hu, r2, false), wu, hu, r2, true);
    let mut mask = grow_from(&seeds, &allowed, wu, hu);
    mask = morph(&mask, wu, hu, r2, true);
    mask = morph(&mask, wu, hu, r2, false);
    let before = mask.iter().filter(|&&m| m).count();
    mask = largest_blob(&mask, wu, hu);
    let kept = mask.iter().filter(|&&m| m).count();
    let extra_frac = if kept > 0 { before.saturating_sub(kept) as f32 / kept as f32 } else { 0.0 };
    // only small holes (< 2 cm^2) are filled; a big pocket between toes must stay open
    let max_hole_px = (200.0 / (down.max(1) as f32 * 25.4 / 300.0).powi(2)) as usize;
    mask = fill_holes(&mask, wu, hu, max_hole_px);
    if !real_object(level) {
        mask.iter_mut().for_each(|m| *m = false);
    }
    Seg { mask, score, thr, level, pale_thr, w, h, extra_frac }
}

/// Is the thing on the glass skin-like enough to be a foot? (typical seed redness, R - G units)
fn real_object(level: f32) -> bool {
    level >= MIN_LEVEL
}

fn percentile(mut v: Vec<f32>, q: f32) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let i = ((v.len() - 1) as f32 * q).round() as usize;
    *v.select_nth_unstable_by(i, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).1
}

/// Mean over a (2r+1)^2 box, edges clamped (integral image).
fn box_blur(v: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut ii = vec![0f64; (w + 1) * (h + 1)];
    for y in 0..h {
        let mut row = 0f64;
        for x in 0..w {
            row += v[y * w + x] as f64;
            ii[(y + 1) * (w + 1) + x + 1] = ii[y * (w + 1) + x + 1] + row;
        }
    }
    let mut out = vec![0f32; w * h];
    for y in 0..h {
        let (y0, y1) = (y.saturating_sub(r), (y + r + 1).min(h));
        for x in 0..w {
            let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(w));
            let s = ii[y1 * (w + 1) + x1] - ii[y0 * (w + 1) + x1] - ii[y1 * (w + 1) + x0] + ii[y0 * (w + 1) + x0];
            out[y * w + x] = (s / ((x1 - x0) * (y1 - y0)) as f64) as f32;
        }
    }
    out
}

/// Background estimate: least-squares cubic surface in (x, y) over the non-excluded pixels (every 7th,
/// for speed), plus the per-column median of what is left (the scanner's fixed column pattern).
fn background(v: &[f32], excl: &[bool], w: usize, h: usize) -> Vec<f32> {
    const T: usize = 10; // terms x^i y^j, i + j <= 3
    let terms = |x: f64, y: f64| -> [f64; T] { [1.0, y, y * y, y * y * y, x, x * y, x * y * y, x * x, x * x * y, x * x * x] };
    let nx = |x: usize| x as f64 / w as f64 * 2.0 - 1.0;
    let ny = |y: usize| y as f64 / h as f64 * 2.0 - 1.0;
    let mut ata = [[0f64; T]; T];
    let mut atb = [0f64; T];
    let mut used = 0usize;
    for i in (0..w * h).step_by(7) {
        if excl[i] {
            continue;
        }
        let t = terms(nx(i % w), ny(i / w));
        for a in 0..T {
            atb[a] += t[a] * v[i] as f64;
            for b in 0..T {
                ata[a][b] += t[a] * t[b];
            }
        }
        used += 1;
    }
    let coef = if used > 50 { solve(ata, atb) } else { None }.unwrap_or_else(|| {
        // too few background pixels to fit a surface: fall back to a flat median
        let mut c = [0f64; T];
        c[0] = percentile(v.to_vec(), 0.5) as f64;
        c
    });
    let mut smooth = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let t = terms(nx(x), ny(y));
            smooth[y * w + x] = (0..T).map(|a| coef[a] * t[a]).sum::<f64>() as f32;
        }
    }
    // column offsets, only where enough of the column is background (>= 10 % of its height);
    // other columns borrow the offset of the nearest trusted column
    let off: Vec<Option<f32>> = (0..w)
        .map(|x| {
            let res: Vec<f32> = (0..h).filter(|&y| !excl[y * w + x]).map(|y| v[y * w + x] - smooth[y * w + x]).collect();
            if res.len() * 10 >= h { Some(percentile(res, 0.5)) } else { None }
        })
        .collect();
    let trusted: Vec<usize> = (0..w).filter(|&x| off[x].is_some()).collect();
    for x in 0..w {
        let o = match off[x] {
            Some(o) => o,
            None => trusted.iter().min_by_key(|&&t| t.abs_diff(x)).and_then(|&t| off[t]).unwrap_or(0.0),
        };
        for y in 0..h {
            smooth[y * w + x] += o;
        }
    }
    smooth
}

/// Gaussian elimination with partial pivoting; None if (near) singular.
fn solve<const T: usize>(mut a: [[f64; T]; T], mut b: [f64; T]) -> Option<[f64; T]> {
    for c in 0..T {
        let p = (c..T).max_by(|&i, &j| a[i][c].abs().partial_cmp(&a[j][c].abs()).unwrap_or(std::cmp::Ordering::Equal))?;
        if a[p][c].abs() < 1e-9 {
            return None;
        }
        a.swap(c, p);
        b.swap(c, p);
        for r in c + 1..T {
            let f = a[r][c] / a[c][c];
            for k in c..T {
                a[r][k] -= f * a[c][k];
            }
            b[r] -= f * b[c];
        }
    }
    let mut x = [0f64; T];
    for r in (0..T).rev() {
        x[r] = (b[r] - (r + 1..T).map(|k| a[r][k] * x[k]).sum::<f64>()) / a[r][r];
    }
    Some(x)
}

/// Pixels of `allowed` connected (4-neighbour) to at least one seed pixel.
fn grow_from(seeds: &[bool], allowed: &[bool], w: usize, h: usize) -> Vec<bool> {
    let mut on = vec![false; w * h];
    let mut stack: Vec<usize> = (0..w * h).filter(|&i| seeds[i] && allowed[i]).collect();
    for &i in &stack {
        on[i] = true;
    }
    while let Some(p) = stack.pop() {
        let (x, y) = (p % w, p / w);
        let mut push = |q: usize| {
            if !on[q] && allowed[q] {
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

pub fn analyze_image(img: &RgbImage) -> Analysis {
    let Seg { mask, level, w, h, extra_frac, .. } = segment(img, DOWN);
    let (wu, hu) = (w as usize, h as usize);

    let count = mask.iter().filter(|&&m| m).count();
    let area_frac = count as f32 / (wu * hu) as f32;
    let found = real_object(level) && area_frac > 0.02 && area_frac < 0.85;

    // 5. edge contact: ANY contact warns. Through the case's acrylic the raised ankle is blurred grey
    //    and not outlined, so contact at an edge means the foot itself runs off the scan area (the
    //    old "narrow contact = ankle" exemption silenced heels at the bottom edge).
    let px_per_mm = 300.0 / 25.4 / DOWN as f32;
    let m = ((EDGE_MARGIN_MM * px_per_mm).ceil() as usize).max(1);
    let mut cnt = [0u32; 4];
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0u32, 0u32);
    for y in 0..hu {
        for x in 0..wu {
            if mask[y * wu + x] {
                x0 = x0.min(x as u32);
                y0 = y0.min(y as u32);
                x1 = x1.max(x as u32);
                y1 = y1.max(y as u32);
                for (i, hit) in [(0, x < m), (1, x + m >= wu), (2, y < m), (3, y + m >= hu)] {
                    if hit {
                        cnt[i] += 1;
                    }
                }
            }
        }
    }
    let touches_edge = cnt.iter().any(|&c| c >= 5);
    // clearly skin-coloured, well above the "anything there?" bar
    let clear = found && level >= 2.0 * MIN_LEVEL;

    // shape sanity: principal axes of the mask. A foot is about 2.2-3.2 times longer than wide (by
    // spread) and 150-330 mm long; a ruler, card or speck is far outside that. Lower bar 120 mm: an ID
    // card through the acrylic (red-tinted photo, so "found") measured ~100 mm by this spread.
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
    let odd_shape = found && (major_mm < 120.0 || major_mm > 420.0 || elong < 1.6 || elong > 5.0);
    let extra_objects = found && extra_frac > 0.25;

    // 6. preview with the outline drawn in blue
    let mut prev = img.clone();
    // no foot found = no outline (F2): stray squiggles over an empty glass only confuse the nurse
    for y in 1..hu - 1 {
        for x in 1..wu - 1 {
            if found && mask[y * wu + x] && !(mask[y * wu + x - 1] && mask[y * wu + x + 1] && mask[(y - 1) * wu + x] && mask[(y + 1) * wu + x]) {
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
        clear,
        area_frac,
        odd_shape,
        extra_objects,
        bbox: if count > 0 { Some([x0, y0, x1, y1]) } else { None },
        preview_w: w,
        preview_h: h,
    }
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
        assert!(a.touches_edge);
    }

    #[test]
    fn any_edge_contact_warns() {
        // a heel about 100 px (34 mm) wide running off the bottom edge: through the case this is the foot
        // itself leaving the scan area, not an ankle, so it must warn (was exempted as a "limb" before)
        let a = analyze_image(&scene(700, 900, |x, y| ((x - 350.0) / 150.0).powi(2) + ((y - 450.0) / 300.0).powi(2) < 1.0 || ((300.0..400.0).contains(&x) && y > 450.0)));
        assert!(a.found && a.touches_edge, "found={} edge={}", a.found, a.touches_edge);
    }

    #[test]
    fn grey_object_is_not_a_foot() {
        // a big neutral-grey object (card, packet seen edge-on, shadow): not skin-coloured, so no foot
        let a = analyze_image(&RgbImage::from_fn(700, 900, |x, y| {
            let n = ((x * 7 + y * 13) % 13) as u8;
            if ((x as f32 - 350.0) / 150.0).powi(2) + ((y as f32 - 450.0) / 300.0).powi(2) < 1.0 { Rgb([40 + n, 40 + n, 44 + n]) } else { Rgb([104 + n, 104 + n, 110 + n]) }
        }));
        assert!(!a.found, "area={}", a.area_frac);
    }

    #[test]
    fn blank_scan_finds_nothing() {
        let img = RgbImage::from_pixel(160, 220, Rgb([120, 120, 125]));
        assert!(!analyze_image(&img).found);
    }
}
