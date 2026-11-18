//! Pictures saved next to each scan (2026-11-18 agreement).
//!
//! In the patient folder, for the operator / client (Indonesian names, right foot already turned
//! the right way up because the saved scan is):
//!   * "Peta kontak <kiri|kanan>.jpg"   graded paleness: bright green = pressed hard, dim green =
//!                                       pressed lightly, dark = lifted (podoscope look);
//!   * "Indeks lengkung <..>.jpg"        scan, outline, foot axis and the thirds of the arch index;
//!   * "Sudut Clarke <..>.jpg"           scan with the CHECKED Clarke construction (written only
//!                                       after a person confirmed the points).
//! In the "Arsip analisis" subfolder, for the technical team only, frozen as computed on the day:
//!   * "Kecerahan <..>.jpg"              whole foot coloured by brightness (the 17 Nov feasibility look);
//!   * "Kontak ambang <..>.jpg"          the yes/no contact cut-off behind the app's first Clarke guess.

use crate::measure::{ClarkePts, Measurements, MEASURE_DOWN};
use image::{imageops::FilterType, Rgb, RgbImage};
use std::path::{Path, PathBuf};

pub const ARCHIVE_DIR: &str = "Arsip analisis";
const JPG_Q: u8 = 88;

fn side_word(side: &str) -> &'static str {
    if side == "left" { "kiri" } else { "kanan" }
}

pub fn contact_map_name(side: &str) -> String {
    format!("Peta kontak {}.jpg", side_word(side))
}
pub fn arch_index_name(side: &str) -> String {
    format!("Indeks lengkung {}.jpg", side_word(side))
}
pub fn clarke_name(side: &str) -> String {
    format!("Sudut Clarke {}.jpg", side_word(side))
}
pub fn archive_files(side: &str) -> [String; 2] {
    [format!("Kecerahan {}.jpg", side_word(side)), format!("Kontak ambang {}.jpg", side_word(side))]
}

fn save_jpg(img: &RgbImage, path: &Path) -> Result<(), String> {
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, JPG_Q).encode_image(img).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("jpg.tmp");
    std::fs::write(&tmp, &buf).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Scan at measuring size plus the outline and contact masks.
struct Maps {
    img: RgbImage,
    foot: Vec<bool>,
    contact: Vec<bool>,
    gr: Vec<f32>,
}

fn maps(scan: &Path) -> Result<Maps, String> {
    let full = image::open(scan).map_err(|e| format!("cannot read scan: {e}"))?.to_rgb8();
    let img = image::imageops::resize(&full, full.width() / MEASURE_DOWN, full.height() / MEASURE_DOWN, FilterType::Triangle);
    let seg = crate::analyze::segment(&img, MEASURE_DOWN);
    let c = crate::contact::find(&img, &seg.mask, MEASURE_DOWN);
    let gr = crate::contact::green_red(&img, MEASURE_DOWN);
    Ok(Maps { img, foot: seg.mask, contact: c.mask, gr })
}

fn green(v: f32) -> Rgb<u8> {
    let v = v.clamp(0.0, 1.0);
    Rgb([0, (v * 255.0) as u8, (v * v * 110.0) as u8])
}

/// Graded paleness picture: the one shown to people.
fn graded(m: &Maps) -> RgbImage {
    let w = m.img.width();
    let span = crate::contact::GRADE_HI - crate::contact::GRADE_LO;
    RgbImage::from_fn(w, m.img.height(), |x, y| {
        let i = (y * w + x) as usize;
        if m.foot[i] { green((m.gr[i] - crate::contact::GRADE_LO) / span) } else { Rgb([0, 0, 0]) }
    })
}

/// Brightness picture (17 Nov feasibility test): brightness stretched between the foot's 5th and
/// 99th percentile. It does not tell pressed from lifted skin; archived for comparison only.
fn brightness(m: &Maps) -> RgbImage {
    let w = m.img.width();
    let lum: Vec<f32> = m.img.pixels().map(|p| (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0).collect();
    let mut inside: Vec<f32> = (0..lum.len()).filter(|&i| m.foot[i]).map(|i| lum[i]).collect();
    inside.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |f: f32| inside.get(((inside.len().max(1) - 1) as f32 * f) as usize).copied().unwrap_or(0.0);
    let (lo, hi) = (q(0.05), q(0.99).max(q(0.05) + 1.0));
    RgbImage::from_fn(w, m.img.height(), |x, y| {
        let i = (y * w + x) as usize;
        if m.foot[i] { green((lum[i] - lo) / (hi - lo)) } else { Rgb([0, 0, 0]) }
    })
}

/// Yes/no cut-off: grey outline, green where the contact map says "pressed".
fn cutoff(m: &Maps) -> RgbImage {
    let w = m.img.width();
    RgbImage::from_fn(w, m.img.height(), |x, y| {
        let i = (y * w + x) as usize;
        if m.contact[i] { Rgb([60, 200, 60]) } else if m.foot[i] { Rgb([70, 70, 70]) } else { Rgb([0, 0, 0]) }
    })
}

/// Pictures that depend only on the scan: "Peta kontak" + the two archive pictures.
/// Returns the files written (relative to `dir`).
pub fn write_scan_images(dir: &Path, side: &str, scan: &Path) -> Result<Vec<PathBuf>, String> {
    let m = maps(scan)?;
    let arch = dir.join(ARCHIVE_DIR);
    std::fs::create_dir_all(&arch).map_err(|e| e.to_string())?;
    let [kec, amb] = archive_files(side);
    let out = vec![
        (graded(&m), PathBuf::from(contact_map_name(side))),
        (brightness(&m), Path::new(ARCHIVE_DIR).join(kec)),
        (cutoff(&m), Path::new(ARCHIVE_DIR).join(amb)),
    ];
    for (img, rel) in &out {
        save_jpg(img, &dir.join(rel))?;
    }
    Ok(out.into_iter().map(|(_, r)| r).collect())
}

// ---- drawing ------------------------------------------------------------------------------------

fn dot(img: &mut RgbImage, x: f32, y: f32, r: f32, c: Rgb<u8>) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    let ri = r.ceil() as i32;
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            let (px, py) = (x as i32 + dx, y as i32 + dy);
            if px >= 0 && py >= 0 && px < w && py < h && ((dx * dx + dy * dy) as f32) <= r * r {
                img.put_pixel(px as u32, py as u32, c);
            }
        }
    }
}

fn line(img: &mut RgbImage, a: [f32; 2], b: [f32; 2], width: f32, c: Rgb<u8>) {
    let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
    let n = (len.ceil() as usize).max(1);
    for k in 0..=n {
        let t = k as f32 / n as f32;
        dot(img, a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, width / 2.0, c);
    }
}

/// 5 x 7 pixel glyphs for numbers on the pictures ("38,6°", "0,36").
const GLYPHS: [(char, [u8; 7]); 13] = [
    ('0', [14, 17, 19, 21, 25, 17, 14]),
    ('1', [4, 12, 4, 4, 4, 4, 14]),
    ('2', [14, 17, 1, 2, 4, 8, 31]),
    ('3', [30, 1, 1, 14, 1, 1, 30]),
    ('4', [2, 6, 10, 18, 31, 2, 2]),
    ('5', [31, 16, 30, 1, 1, 17, 14]),
    ('6', [6, 8, 16, 30, 17, 17, 14]),
    ('7', [31, 1, 2, 4, 8, 8, 8]),
    ('8', [14, 17, 17, 14, 17, 17, 14]),
    ('9', [14, 17, 17, 15, 1, 2, 12]),
    (',', [0, 0, 0, 0, 0, 4, 8]),
    ('°', [12, 18, 18, 12, 0, 0, 0]),
    (' ', [0, 0, 0, 0, 0, 0, 0]),
];

fn text(img: &mut RgbImage, x: f32, y: f32, s: &str, scale: f32, c: Rgb<u8>) {
    let mut cx = x;
    for ch in s.chars() {
        if let Some((_, rows)) = GLYPHS.iter().find(|(g, _)| *g == ch) {
            for (ry, bits) in rows.iter().enumerate() {
                for rx in 0..5 {
                    if bits & (16 >> rx) != 0 {
                        for yy in 0..scale as i32 {
                            for xx in 0..scale as i32 {
                                let (px, py) = (cx as i32 + rx * scale as i32 + xx, y as i32 + ry as i32 * scale as i32 + yy);
                                if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                                    img.put_pixel(px as u32, py as u32, c);
                                }
                            }
                        }
                    }
                }
            }
        }
        cx += 6.0 * scale;
    }
}

fn outline(img: &mut RgbImage, foot: &[bool], c: Rgb<u8>) {
    let (w, h) = (img.width() as usize, img.height() as usize);
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            if foot[i] && !(foot[i - 1] && foot[i + 1] && foot[i - w] && foot[i + w]) {
                dot(img, x as f32, y as f32, 2.0, c);
            }
        }
    }
}

fn comma(v: f32, d: usize) -> String {
    format!("{v:.d$}").replace('.', ",")
}

/// "Indeks lengkung": scan, outline, axis and the four lines of the thirds (heel end, 1/3, 2/3,
/// toe base), with the AI value in the corner.
pub fn write_arch_index_image(dir: &Path, side: &str, scan: &Path, m: &Measurements) -> Result<PathBuf, String> {
    let mp = maps(scan)?;
    let d = MEASURE_DOWN as f32;
    let p = |q: [f32; 2]| [q[0] / d, q[1] / d];
    let mut img = mp.img.clone();
    outline(&mut img, &mp.foot, Rgb([60, 130, 235]));
    let (he, tb) = (p(m.heel_end), p(m.toe_base));
    line(&mut img, he, p(m.toe_tip), 2.0, Rgb([230, 230, 230]));
    let ax = [tb[0] - he[0], tb[1] - he[1]];
    let len = (ax[0] * ax[0] + ax[1] * ax[1]).sqrt().max(1.0);
    let pr = [-ax[1] / len * 400.0, ax[0] / len * 400.0];
    for f in [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0] {
        let c = [he[0] + ax[0] * f, he[1] + ax[1] * f];
        line(&mut img, [c[0] - pr[0], c[1] - pr[1]], [c[0] + pr[0], c[1] + pr[1]], 3.0, Rgb([255, 190, 0]));
    }
    text(&mut img, 30.0, 30.0, &comma(m.ai, 2), 8.0, Rgb([255, 255, 255]));
    let rel = PathBuf::from(arch_index_name(side));
    save_jpg(&img, &dir.join(&rel))?;
    Ok(rel)
}

/// "Sudut Clarke": scan with the graded contact picture blended in, the checked construction and
/// the angle. Only called for a confirmed check.
pub fn write_clarke_image(dir: &Path, side: &str, scan: &Path, pts: &ClarkePts, deg: f32) -> Result<PathBuf, String> {
    let mp = maps(scan)?;
    let g = graded(&mp);
    let mut img = RgbImage::from_fn(mp.img.width(), mp.img.height(), |x, y| {
        let (a, b) = (mp.img.get_pixel(x, y), g.get_pixel(x, y));
        Rgb([((a[0] as u16 + b[0] as u16) / 2) as u8, ((a[1] as u16 + b[1] as u16) / 2) as u8, ((a[2] as u16 + b[2] as u16) / 2) as u8])
    });
    let d = MEASURE_DOWN as f32;
    let p = |q: [f32; 2]| [q[0] / d, q[1] / d];
    let red = Rgb([235, 60, 60]);
    line(&mut img, p(pts.fore), p(pts.heel), 4.0, red);
    line(&mut img, p(pts.fore), p(pts.arch), 4.0, red);
    for q in [pts.fore, pts.heel, pts.arch] {
        let q = p(q);
        dot(&mut img, q[0], q[1], 9.0, Rgb([255, 255, 255]));
        dot(&mut img, q[0], q[1], 6.0, red);
    }
    text(&mut img, 30.0, 30.0, &format!("{}°", comma(deg, 1)), 8.0, Rgb([255, 255, 255]));
    let rel = PathBuf::from(clarke_name(side));
    save_jpg(&img, &dir.join(&rel))?;
    Ok(rel)
}

/// Team tool: one comparison sheet for a whole session. One row per scanned foot: scan, brightness
/// (archive), graded contact map, cut-off (archive); pictures as saved on the day. Also writes a
/// text index naming each row. Returns the number of rows.
pub fn session_sheet(sesi_dir: &Path, out: &Path) -> Result<usize, String> {
    const W: u32 = 260;
    let mut rows: Vec<RgbImage> = vec![];
    let mut index = String::new();
    for (pd, p) in crate::session::list_patients(sesi_dir) {
        for e in &p.scans {
            let [kec, amb] = archive_files(&e.side);
            let files = [pd.join(&e.file), pd.join(ARCHIVE_DIR).join(kec), pd.join(contact_map_name(&e.side)), pd.join(ARCHIVE_DIR).join(amb)];
            let mut tiles = vec![];
            for f in &files {
                let t = match image::open(f) {
                    Ok(im) => {
                        let im = im.to_rgb8();
                        let h = (im.height() as f32 * W as f32 / im.width() as f32) as u32;
                        image::imageops::resize(&im, W, h, FilterType::Triangle)
                    }
                    Err(_) => RgbImage::from_pixel(W, (W as f32 * 1.375) as u32, Rgb([40, 0, 0])),
                };
                tiles.push(t);
            }
            let h = tiles.iter().map(|t| t.height()).max().unwrap_or(1);
            let mut row = RgbImage::new(W * 4 + 30, h + 8);
            for (k, t) in tiles.iter().enumerate() {
                image::imageops::replace(&mut row, t, (k as u32 * (W + 10)) as i64, 4);
            }
            rows.push(row);
            index.push_str(&format!("{}\t{}\t{}\t{}\n", rows.len(), pd.file_name().unwrap_or_default().to_string_lossy(), side_word(&e.side), e.images_version.as_deref().unwrap_or("-")));
        }
    }
    if rows.is_empty() {
        return Err("no scans in this session".into());
    }
    let total: u32 = rows.iter().map(|r| r.height()).sum();
    let mut sheet = RgbImage::new(W * 4 + 30, total);
    let mut y = 0;
    for r in &rows {
        image::imageops::replace(&mut sheet, r, 0, y as i64);
        y += r.height();
    }
    save_jpg(&sheet, out)?;
    std::fs::write(out.with_extension("txt"), format!("row\tpatient\tfoot\timages_version\ncolumns: scan | brightness (archive) | graded contact map | cut-off (archive)\n{index}")).map_err(|e| e.to_string())?;
    Ok(rows.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_cover_the_numbers_we_print() {
        for s in ["38,6°", "0,36"] {
            assert!(s.chars().all(|c| GLYPHS.iter().any(|(g, _)| *g == c)), "{s}");
        }
    }

    #[test]
    fn pictures_are_written_for_a_synthetic_scan() {
        // pale heel and ball, pink arch, on a dark background; 1700 x 2400 px = 300 dpi-ish size
        let (w, h) = (1700u32, 2400u32);
        let img = RgbImage::from_fn(w, h, |x, y| {
            let (dx, dy) = ((x as f32 - 850.0) / 420.0, (y as f32 - 1200.0) / 1050.0);
            if dx * dx + dy * dy > 1.0 {
                return Rgb([38, 36, 42]);
            }
            if y < 750 || y > 1650 { Rgb([116, 86, 75]) } else { Rgb([108, 69, 70]) }
        });
        let d = std::env::temp_dir().join(format!("sf_fig_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let scan = d.join("scan.png");
        img.save(&scan).unwrap();
        let files = write_scan_images(&d, "left", &scan).unwrap();
        assert_eq!(files.len(), 3);
        for f in &files {
            assert!(d.join(f).exists(), "{}", f.display());
        }
        assert!(d.join("Peta kontak kiri.jpg").exists() && d.join("Arsip analisis").join("Kontak ambang kiri.jpg").exists());
        // the graded picture is brighter on the pressed heel than on the lifted arch
        let g = image::open(d.join("Peta kontak kiri.jpg")).unwrap().to_rgb8();
        let at = |x: u32, y: u32| g.get_pixel(x / MEASURE_DOWN, y / MEASURE_DOWN)[1];
        assert!(at(850, 2000) > at(850, 1200) + 80, "heel {} arch {}", at(850, 2000), at(850, 1200));
        let pts = ClarkePts { fore: [500.0, 700.0], heel: [520.0, 1900.0], arch: [700.0, 1300.0] };
        write_clarke_image(&d, "left", &scan, &pts, 38.6).unwrap();
        assert!(d.join("Sudut Clarke kiri.jpg").exists());
        let _ = std::fs::remove_dir_all(&d);
    }
}
