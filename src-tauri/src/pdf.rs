//! The patient's result sheet ("Hasil skrining kaki - <Nama> - <tanggal>.pdf") in the patient folder.
//!
//! Agreed 25 Nov 2026: two A4 pages in plain Bahasa Indonesia for the patient (sent by WhatsApp, no
//! printer on site). Page 1 = arch shape in words (Clarke, only after a person checked the points),
//! INLOW risk class, the client's advice for the worse foot, the operator's notes and a "bukan
//! diagnosis" line; page 2 = scan + contact map per foot. No raw numbers.
//!
//! Typst (the same 0.15.1 as the CLI that made the mockup) runs inside the app: offline, nothing
//! extra on the flash disk. Fonts come from Windows (Segoe UI).
//!
//! Two fingerprints: `content_fingerprint` covers what the patient reads (a change after sending
//! means "kirim ulang"); `build_key` adds the header lines and the app version (a change means the
//! file is made again, without asking for a resend).

use crate::inlow;
use crate::measure;
use crate::session::{self, Patient, Sesi, Settings};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

const TEMPLATE: &str = include_str!("hasil_skrining.typ");
pub const SHEET_PREFIX: &str = "Hasil skrining kaki - ";

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct FootCard {
    pub sisi: String,
    pub arch: String,
    pub arch_ok: bool,
    pub arch_note: String,
    pub risk: String,
    pub risk_fill: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Picture {
    pub sisi: String,
    pub scan: String,
    /// Empty when the contact map is missing.
    pub kontak: String,
}

/// Everything the patient reads on the sheet.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Content {
    pub nama: String,
    pub usia: u32,
    pub kelamin: String,
    pub tanggal: String,
    pub tempat: String,
    pub petugas: String,
    pub feet: Vec<FootCard>,
    pub saran: Vec<String>,
    pub saran_note: String,
    pub catatan: Vec<String>,
    pub gambar: Vec<Picture>,
    /// When each scan was accepted: a rescan changes the pictures, so the sheet counts as changed.
    pub scans_at: Vec<String>,
}

#[derive(Serialize)]
struct SheetData<'a> {
    content: &'a Content,
    penyelenggara: String,
    kontak: String,
    versi: &'static str,
    dibuat: String,
}

const BULAN: [&str; 12] = ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];

/// "2026-11-19" -> "19 November 2026" (short: "19 Nov 2026"); anything else comes back unchanged.
pub fn tanggal(iso: &str, short: bool) -> String {
    let parts: Vec<&str> = iso.get(..10).unwrap_or(iso).split('-').collect();
    match (parts.first().and_then(|y| y.parse::<u32>().ok()), parts.get(1).and_then(|m| m.parse::<usize>().ok()), parts.get(2).and_then(|d| d.parse::<u32>().ok())) {
        (Some(y), Some(m @ 1..=12), Some(d)) => {
            let b = BULAN[m - 1];
            format!("{d} {} {y}", if short { &b[..3] } else { b })
        }
        _ => iso.to_string(),
    }
}

fn side_word(side: &str) -> &'static str {
    if side == "right" { "kanan" } else { "kiri" }
}

fn risk_fill(letter: char) -> &'static str {
    match letter {
        'A' | 'B' => "#d7f0e4",
        'C' => "#fde7c2",
        'D' => "#f9d0b4",
        _ => "#f6c9c9",
    }
}

fn foot_card(p: &Patient, side: &str) -> FootCard {
    let e = p.scans.iter().find(|e| e.side == side);
    let (arch, arch_ok, arch_note) = match e {
        None => ("Belum dipindai".to_string(), false, String::new()),
        Some(e) if e.measurements.is_none() => ("Belum dapat ditentukan".into(), false, "Gambar kaki belum dapat diukur.".into()),
        Some(e) => match session::reported_clarke(e) {
            (Some(deg), "diperiksa") => {
                let w = measure::clarke_label(deg).split(' ').next().unwrap_or("");
                let mut cs = w.chars();
                let cap = cs.next().map(|f| f.to_uppercase().collect::<String>() + cs.as_str()).unwrap_or_default();
                (cap, true, format!("Sudut Clarke {}°", deg.round() as i32))
            }
            _ => ("Belum dapat ditentukan".into(), false, "Titik pada gambar belum diperiksa petugas.".into()),
        },
    };
    let scores = if side == "right" { &p.inlow.kanan } else { &p.inlow.kiri };
    let (risk, fill) = match scores.total() {
        Some(t) => {
            let c = inlow::risk_class(t);
            (format!("{} ({})", c.name, c.letter), risk_fill(c.letter))
        }
        None => ("INLOW belum lengkap".to_string(), "#e6e9ef"),
    };
    FootCard { sisi: side_word(side).into(), arch, arch_ok, arch_note, risk, risk_fill: fill.into() }
}

/// Which foot the advice follows, in words.
fn saran_note(p: &Patient) -> String {
    let t = |side: &str| if side == "right" { p.inlow.kanan.total() } else { p.inlow.kiri.total() };
    let class = |side: &str| t(side).map(inlow::risk_class);
    match (class("right"), class("left")) {
        (None, None) => "Saran belum dapat diberikan karena pemeriksaan INLOW belum lengkap.".into(),
        (Some(c), None) => format!("Berdasarkan kaki kanan ({}); INLOW kaki kiri belum lengkap.", c.name.to_lowercase()),
        (None, Some(c)) => format!("Berdasarkan kaki kiri ({}); INLOW kaki kanan belum lengkap.", c.name.to_lowercase()),
        (Some(r), Some(l)) if r.letter == l.letter => format!("Untuk kedua kaki ({}).", r.name.to_lowercase()),
        (Some(r), Some(l)) => {
            let (side, c) = if r.letter > l.letter { ("kanan", r) } else { ("kiri", l) };
            format!("Mengikuti kaki dengan risiko lebih tinggi (kaki {side}, {}).", c.name.to_lowercase())
        }
    }
}

/// The sheet's content for a patient (pictures named, not yet made).
pub fn content(p: &Patient, sesi: Option<&Sesi>) -> Content {
    let mut catatan = vec![];
    for (label, n) in [("Kaki kanan: ", &p.notes.kanan), ("Kaki kiri: ", &p.notes.kiri), ("", &p.notes.umum)] {
        if !n.trim().is_empty() {
            catatan.push(format!("{label}{}", n.trim()));
        }
    }
    let gambar = ["right", "left"]
        .iter()
        .filter(|s| p.scans.iter().any(|e| e.side == **s))
        .map(|s| Picture { sisi: side_word(s).into(), scan: format!("img/scan_{s}.jpg"), kontak: format!("img/kontak_{s}.jpg") })
        .collect();
    Content {
        nama: p.nama.clone(),
        usia: p.usia,
        kelamin: p.kelamin.clone(),
        tanggal: tanggal(sesi.map(|s| s.date.as_str()).unwrap_or(p.created.as_str()), false),
        tempat: sesi.map(|s| s.place.clone()).filter(|s| !s.is_empty()).unwrap_or_else(|| "-".into()),
        petugas: p.operator.clone(),
        feet: vec![foot_card(p, "right"), foot_card(p, "left")],
        saran: crate::export::saran(p),
        saran_note: saran_note(p),
        catatan,
        gambar,
        scans_at: p.scans.iter().map(|e| format!("{}:{}", e.side, e.accepted_at)).collect(),
    }
}

/// FNV-1a, 64 bit: stable across builds (std's hasher is not promised to be).
fn fnv(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

pub fn content_fingerprint(c: &Content) -> String {
    fnv(serde_json::to_string(c).unwrap_or_default().as_bytes())
}

pub fn build_key(c: &Content, s: &Settings) -> String {
    fnv(format!("{}|{}|{}|{}", content_fingerprint(c), s.penyelenggara, s.kontak, env!("CARGO_PKG_VERSION")).as_bytes())
}

pub fn sesi_of(patient_dir: &Path) -> Option<Sesi> {
    patient_dir.parent().and_then(Path::parent).and_then(|d| session::read_sesi(d).ok())
}

/// Fingerprint of what the patient would read now (for the sending status).
pub fn fingerprint_of(dir: &Path, p: &Patient) -> String {
    content_fingerprint(&content(p, sesi_of(dir).as_ref()))
}

pub fn sheet_name(p: &Patient, sesi: Option<&Sesi>) -> String {
    let date = tanggal(sesi.map(|s| s.date.as_str()).unwrap_or(p.created.as_str()), true);
    format!("{SHEET_PREFIX}{} - {date}.pdf", session::folder_safe(&p.nama))
}

/// The sheet file in the folder, if one was made (the name follows the patient's name and date).
pub fn find_sheet(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|f| {
        let n = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        n.starts_with(SHEET_PREFIX) && n.ends_with(".pdf")
    })
}

// ------------------------------------------------------------------------------------------------
// Pictures: cropped to the foot (from the contact map, which is black outside the foot), small JPEG

fn foot_box(contact: &image::GrayImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = contact.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in contact.enumerate_pixels() {
        if p[0] > 40 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let m = 40;
    Some((x0.saturating_sub(m), y0.saturating_sub(m), (x1 + m).min(w - 1), (y1 + m).min(h - 1)))
}

fn jpeg(img: &image::RgbImage) -> Result<Vec<u8>, String> {
    let max_h = 1000;
    let img = if img.height() > max_h {
        let w = (img.width() as f32 * max_h as f32 / img.height() as f32).round().max(1.0) as u32;
        image::imageops::resize(img, w, max_h, image::imageops::FilterType::Triangle)
    } else {
        img.clone()
    };
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 82).encode_image(&img).map_err(|e| e.to_string())?;
    Ok(out)
}

/// (scan, contact) JPEG bytes for one foot; contact None if its picture is missing.
fn pictures(dir: &Path, p: &Patient, side: &str) -> Result<(Vec<u8>, Option<Vec<u8>>), String> {
    let e = p.scans.iter().find(|e| e.side == side).ok_or("scan not found")?;
    let scan = image::open(dir.join(&e.file)).map_err(|er| format!("cannot read scan: {er}"))?.to_rgb8();
    let contact = image::open(dir.join(crate::figures::contact_map_name(side))).ok().map(|c| c.to_rgb8());
    let Some(contact) = contact else { return Ok((jpeg(&scan)?, None)) };
    let gray = image::DynamicImage::ImageRgb8(contact.clone()).to_luma8();
    let Some((x0, y0, x1, y1)) = foot_box(&gray) else { return Ok((jpeg(&scan)?, Some(jpeg(&contact)?))) };
    let crop_c = image::imageops::crop_imm(&contact, x0, y0, x1 - x0, y1 - y0).to_image();
    // the contact map is a smaller copy of the scan: scale the box
    let (sx, sy) = (scan.width() as f32 / contact.width() as f32, scan.height() as f32 / contact.height() as f32);
    let (a, b) = ((x0 as f32 * sx) as u32, (y0 as f32 * sy) as u32);
    let (w, h) = ((((x1 - x0) as f32) * sx) as u32, (((y1 - y0) as f32) * sy) as u32);
    let crop_s = image::imageops::crop_imm(&scan, a, b, w.min(scan.width() - a), h.min(scan.height() - b)).to_image();
    Ok((jpeg(&crop_s)?, Some(jpeg(&crop_c)?)))
}

// ------------------------------------------------------------------------------------------------
// Typst

struct Fonts {
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

fn fonts() -> Result<&'static Fonts, String> {
    static F: OnceLock<Result<Fonts, String>> = OnceLock::new();
    F.get_or_init(|| {
        let dir = PathBuf::from(std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into())).join("Fonts");
        let mut fonts = vec![];
        for f in ["segoeui.ttf", "segoeuib.ttf", "segoeuii.ttf", "segoeuiz.ttf", "seguisym.ttf"] {
            if let Ok(b) = std::fs::read(dir.join(f)) {
                fonts.extend(Font::iter(Bytes::new(b)));
            }
        }
        if fonts.is_empty() {
            return Err("FONT: Segoe UI was not found in the Windows fonts folder".into());
        }
        Ok(Fonts { book: LazyHash::new(FontBook::from_fonts(&fonts)), fonts })
    })
    .as_ref()
    .map_err(Clone::clone)
}

fn file_id(path: &str) -> FileId {
    RootedPath::new(VirtualRoot::Project, VirtualPath::new(path).expect("fixed path")).intern()
}

struct SheetWorld {
    library: LazyHash<Library>,
    fonts: &'static Fonts,
    main: Source,
    files: HashMap<FileId, Bytes>,
}

impl World for SheetWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.fonts.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotFound(PathBuf::from(format!("{id:?}"))))
        }
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.get(&id).cloned().ok_or_else(|| FileError::NotFound(PathBuf::from(format!("{id:?}"))))
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        None
    }
}

/// Typeset the sheet. `files` = data.json and the pictures, by their path in the template.
fn typeset(files: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>, String> {
    let world = SheetWorld {
        library: LazyHash::new(Library::default()),
        fonts: fonts()?,
        main: Source::new(file_id("main.typ"), TEMPLATE.to_string()),
        files: files.into_iter().map(|(n, b)| (file_id(&n), Bytes::new(b))).collect(),
    };
    let doc = typst::compile::<typst_layout::PagedDocument>(&world)
        .output
        .map_err(|errs| format!("SHEET: {}", errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ")))?;
    typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default()).map_err(|errs| format!("SHEET: {}", errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ")))
}

/// Make the sheet now and record its build key. Returns the file.
pub fn write_sheet(dir: &Path, s: &Settings) -> Result<PathBuf, String> {
    let p = session::read_patient(dir)?;
    let sesi = sesi_of(dir);
    let c = content(&p, sesi.as_ref());
    let mut files = vec![];
    for side in ["right", "left"] {
        if !p.scans.iter().any(|e| e.side == side) {
            continue;
        }
        let (scan, contact) = pictures(dir, &p, side)?;
        files.push((format!("img/scan_{side}.jpg"), scan));
        if let Some(k) = contact {
            files.push((format!("img/kontak_{side}.jpg"), k));
        }
    }
    // a missing contact map: tell the template to leave that picture out
    let mut c2 = c.clone();
    for g in &mut c2.gambar {
        if !files.iter().any(|(n, _)| *n == g.kontak) {
            g.kontak = String::new();
        }
    }
    let data = SheetData { content: &c2, penyelenggara: s.penyelenggara.clone(), kontak: s.kontak.clone(), versi: env!("CARGO_PKG_VERSION"), dibuat: tanggal(&session::today(), false) };
    files.push(("data.json".into(), serde_json::to_vec(&data).map_err(|e| e.to_string())?));
    let bytes = typeset(files)?;
    let out = dir.join(sheet_name(&p, sesi.as_ref()));
    let tmp = dir.join("_sheet.tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("cannot save the result sheet: {e}"))?;
    // the name follows the patient's name: an older sheet under another name goes
    if let Some(old) = find_sheet(dir).filter(|f| *f != out) {
        let _ = std::fs::remove_file(old);
    }
    std::fs::rename(&tmp, &out).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("cannot save the result sheet (is it open in a PDF viewer?): {e}")
    })?;
    session::set_pdf_key(dir, Some(build_key(&c, s)))?;
    Ok(out)
}

/// Make the sheet if the patient is finished and the file is missing or out of date. Problems come
/// back as a message: the sheet must never block saving a patient.
pub fn ensure_sheet(dir: &Path, s: &Settings) -> Option<String> {
    let p = session::read_patient(dir).ok()?;
    if !p.completed || p.cancelled {
        return None;
    }
    let key = build_key(&content(&p, sesi_of(dir).as_ref()), s);
    if p.pdf_key.as_deref() == Some(key.as_str()) && find_sheet(dir).is_some() {
        return None;
    }
    write_sheet(dir, s).err().map(|e| if e.starts_with("SHEET") || e.starts_with("FONT") { e } else { format!("SHEET: {e}") })
}

/// The WhatsApp message for a patient, from the Settings text (or the default).
pub fn wa_message(p: &Patient, sesi: Option<&Sesi>, s: &Settings) -> String {
    let t = if s.wa_pesan.trim().is_empty() { session::DEFAULT_WA_PESAN } else { s.wa_pesan.as_str() };
    let sapaan = if p.kelamin == "Perempuan" { "Ibu" } else { "Bapak" };
    t.replace("{sapaan}", sapaan)
        .replace("{nama}", &p.nama)
        .replace("{tanggal}", &tanggal(sesi.map(|s| s.date.as_str()).unwrap_or(p.created.as_str()), false))
        .replace("{tempat}", sesi.map(|s| s.place.as_str()).filter(|s| !s.is_empty()).unwrap_or("-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_in_indonesian() {
        assert_eq!(tanggal("2026-11-19", false), "19 November 2026");
        assert_eq!(tanggal("2026-08-05T10:00:00+07:00", true), "5 Agu 2026");
        assert_eq!(tanggal("kemarin", false), "kemarin");
    }

    #[test]
    fn fingerprint_is_stable_and_follows_content() {
        assert_eq!(fnv(b"abc"), "e71fa2190541574b");
        assert_ne!(fnv(b"abc"), fnv(b"abd"));
    }
}
