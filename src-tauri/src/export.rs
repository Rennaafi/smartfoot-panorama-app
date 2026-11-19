//! Excel files in the client's layout ("isian" sheet of "New ISi SKOR PENGKAJIAN KAKI (2).xlsx"):
//! No, Pasien, Usia, Kelamin, TD, BB, then the 12 INLOW items as KANAN/KIRI column pairs, totals,
//! risk class, Clarke's angle, arch index and the conclusion. Our extra columns (date, place,
//! operator, posture, foot length/width, notes, internal ID, folder) follow after the client's.
//!
//! Both files are rebuilt from the patient files every time, so they can never disagree with them:
//! "Rekap sesi <...>.xlsx" in each session folder and "Rekap semua sesi.xlsx" in the data folder.

use crate::inlow::{self, FootScores, ITEMS};
use crate::measure::{clarke_label, Measurements};
use crate::session::{self, Patient, Sesi};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};
use std::path::{Path, PathBuf};

fn comma(v: f32, decimals: usize) -> String {
    format!("{v:.decimals$}").replace('.', ",")
}

fn meas<'a>(p: &'a Patient, side: &str) -> Option<&'a Measurements> {
    entry(p, side).and_then(|e| e.measurements.as_ref())
}

fn entry<'a>(p: &'a Patient, side: &str) -> Option<&'a session::ScanEntry> {
    p.scans.iter().find(|e| e.side == side)
}

fn foot_scores<'a>(p: &'a Patient, side: &str) -> &'a FootScores {
    if side == "right" { &p.inlow.kanan } else { &p.inlow.kiri }
}

/// One foot in words, e.g. "Lengkung normal (sudut Clarke 35°). Arch Index 0,36. Risiko rendah (B)".
/// Clarke gets a label only after a person confirmed its points; AI never gets Cavanagh's label,
/// because it is measured on the outline, which includes the lifted arch (2026-11-18 agreement).
pub fn foot_summary(p: &Patient, side: &str) -> String {
    let mut parts = vec![];
    match (entry(p, side), meas(p, side)) {
        (Some(e), Some(m)) => {
            match session::reported_clarke(e) {
                (Some(c), "diperiksa") => parts.push(format!("Lengkung {} (sudut Clarke {}°)", arch_words(c), comma(c, 0))),
                _ => parts.push("Lengkung belum dapat ditentukan (titik belum diperiksa)".into()),
            }
            parts.push(format!("Arch Index {}", comma(m.ai, 2)));
        }
        _ => parts.push(if p.scans.iter().any(|e| e.side == side) { "Belum terukur".into() } else { "Belum dipindai".into() }),
    }
    match foot_scores(p, side).total() {
        Some(t) => {
            let c = inlow::risk_class(t);
            parts.push(format!("{} ({})", c.name, c.letter));
        }
        None => parts.push("INLOW belum lengkap".into()),
    }
    parts.join(". ")
}

/// Clarke's label for a sentence: "datar / flat foot", "normal", "tinggi / cavus".
fn arch_words(deg: f32) -> String {
    clarke_label(deg).replace(" (", " / ").replace(')', "")
}

/// "Hasil skrining": one sentence per foot, right first as in the client's sheet (agreed 2026-11-19:
/// Clarke gives the label only after a person checked the points; AI is a number only).
pub fn hasil(p: &Patient) -> Vec<String> {
    vec![format!("Kaki kanan: {}.", foot_summary(p, "right")), format!("Kaki kiri: {}.", foot_summary(p, "left"))]
}

/// The client's advice for the worse foot (empty until at least one foot's INLOW is complete).
pub fn saran(p: &Patient) -> Vec<String> {
    inlow::worse_letter(&p.inlow).map(|l| inlow::management(l).1.iter().map(|s| s.to_string()).collect()).unwrap_or_default()
}

/// The KESIMPULAN cell: both feet, right first as in the client's sheet.
pub fn kesimpulan(p: &Patient) -> String {
    hasil(p).join(" ")
}

struct Fmt {
    title: Format,
    head: Format,
    head_lihat: Format,
    head_sentuh: Format,
    head_sensasi: Format,
    head_hasil: Format,
    head_extra: Format,
    cell: Format,
    cell_c: Format,
    num1: Format,
    num3: Format,
    wrap: Format,
}

fn formats() -> Fmt {
    let base = Format::new().set_border(FormatBorder::Thin).set_font_name("Calibri").set_font_size(10);
    let head = base.clone().set_bold().set_align(FormatAlign::Center).set_align(FormatAlign::VerticalCenter).set_text_wrap();
    Fmt {
        title: Format::new().set_bold().set_font_size(13).set_font_name("Calibri"),
        head_lihat: head.clone().set_background_color(Color::RGB(0xDDEBF7)),
        head_sentuh: head.clone().set_background_color(Color::RGB(0xE2EFDA)),
        head_sensasi: head.clone().set_background_color(Color::RGB(0xFFF2CC)),
        head_hasil: head.clone().set_background_color(Color::RGB(0xFCE4D6)),
        head_extra: head.clone().set_background_color(Color::RGB(0xEDEDED)),
        head,
        cell: base.clone(),
        cell_c: base.clone().set_align(FormatAlign::Center),
        num1: base.clone().set_align(FormatAlign::Center).set_num_format("0.0"),
        num3: base.clone().set_align(FormatAlign::Center).set_num_format("0.000"),
        wrap: base.clone().set_text_wrap().set_align(FormatAlign::Top),
    }
}

const EXTRA: [&str; 15] = ["Tanggal", "Tempat", "Operator", "Posisi", "Panjang kaki kanan (mm)", "Panjang kaki kiri (mm)", "Lebar kaki kanan (mm)", "Lebar kaki kiri (mm)", "Catatan", "Tanda manual", "Status", "ID internal", "Folder", "Status Clarke kanan", "Status Clarke kiri"];

/// Column of the first INLOW item (after No, Pasien, Usia, Kelamin, TD, BB).
const C_ITEMS: u16 = 6;

fn write_sheet(ws: &mut Worksheet, title: &str, rows: &[(&Sesi, String, &Patient)], f: &Fmt) -> Result<(), XlsxError> {
    ws.set_name("isian")?;
    ws.write_string_with_format(0, 0, title, &f.title)?;
    // --- header rows 3-5 (0-based 2-4)
    let (r0, r1, r2) = (2u32, 3u32, 4u32);
    for (c, name) in ["No", "PASIEN", "USIA", "KELAMIN", "TD", "BB"].iter().enumerate() {
        ws.merge_range(r0, c as u16, r2, c as u16, name, &f.head)?;
    }
    let mut c = C_ITEMS;
    for (group, fmt) in [("LIHAT", &f.head_lihat), ("SENTUH", &f.head_sentuh), ("SENSASI", &f.head_sensasi)] {
        let items: Vec<_> = ITEMS.iter().filter(|i| i.group == group).collect();
        let span = items.len() as u16 * 2;
        ws.merge_range(r0, c, r0, c + span - 1, group, fmt)?;
        for it in items {
            ws.merge_range(r1, c, r1, c + 1, &it.label.to_uppercase(), fmt)?;
            ws.write_string_with_format(r2, c, "KANAN", fmt)?;
            ws.write_string_with_format(r2, c + 1, "KIRI", fmt)?;
            c += 2;
        }
    }
    for name in ["TOTAL", "ANALISIS RISIKO", "CLARKE'S ANGLE (°)", "ARCH INDEX"] {
        ws.merge_range(r0, c, r1, c + 1, name, &f.head_hasil)?;
        ws.write_string_with_format(r2, c, "KANAN", &f.head_hasil)?;
        ws.write_string_with_format(r2, c + 1, "KIRI", &f.head_hasil)?;
        c += 2;
    }
    let c_kes = c;
    ws.merge_range(r0, c_kes, r2, c_kes, "KESIMPULAN RISIKO, CLARKE'S ANGLE DAN ARCH INDEX", &f.head_hasil)?;
    for (k, name) in EXTRA.iter().enumerate() {
        ws.merge_range(r0, c_kes + 1 + k as u16, r2, c_kes + 1 + k as u16, name, &f.head_extra)?;
    }
    // --- data
    for (i, (s, folder, p)) in rows.iter().enumerate() {
        let r = r2 + 1 + i as u32;
        ws.write_number_with_format(r, 0, p.no as f64, &f.cell_c)?;
        ws.write_string_with_format(r, 1, &p.nama, &f.cell)?;
        ws.write_number_with_format(r, 2, p.usia as f64, &f.cell_c)?;
        ws.write_string_with_format(r, 3, &p.kelamin, &f.cell_c)?;
        ws.write_string_with_format(r, 4, &p.td, &f.cell_c)?;
        match p.bb {
            Some(b) => ws.write_number_with_format(r, 5, b as f64, &f.cell_c)?,
            None => ws.write_blank(r, 5, &f.cell_c)?,
        };
        let mut c = C_ITEMS;
        for group in ["LIHAT", "SENTUH", "SENSASI"] {
            for it in ITEMS.iter().filter(|i| i.group == group) {
                for (k, fs) in [&p.inlow.kanan, &p.inlow.kiri].iter().enumerate() {
                    match fs.items.get(it.key) {
                        Some(v) => ws.write_number_with_format(r, c + k as u16, *v as f64, &f.cell_c)?,
                        None => ws.write_blank(r, c + k as u16, &f.cell_c)?,
                    };
                }
                c += 2;
            }
        }
        for (k, fs) in [&p.inlow.kanan, &p.inlow.kiri].iter().enumerate() {
            match fs.total() {
                Some(t) => {
                    ws.write_number_with_format(r, c + k as u16, t as f64, &f.cell_c)?;
                    ws.write_string_with_format(r, c + 2 + k as u16, &inlow::risk_class(t).letter.to_string(), &f.cell_c)?;
                }
                None => {
                    ws.write_blank(r, c + k as u16, &f.cell_c)?;
                    ws.write_blank(r, c + 2 + k as u16, &f.cell_c)?;
                }
            };
        }
        c += 4;
        for (k, side) in ["right", "left"].iter().enumerate() {
            let m = meas(p, side);
            match entry(p, side).and_then(|e| session::reported_clarke(e).0) {
                Some(v) => ws.write_number_with_format(r, c + k as u16, v as f64, &f.num1)?,
                None => ws.write_blank(r, c + k as u16, &f.cell_c)?,
            };
            match m {
                Some(m) => ws.write_number_with_format(r, c + 2 + k as u16, m.ai as f64, &f.num3)?,
                None => ws.write_blank(r, c + 2 + k as u16, &f.cell_c)?,
            };
        }
        ws.write_string_with_format(r, c_kes, &kesimpulan(p), &f.wrap)?;
        let notes = [("Kiri", &p.notes.kiri), ("Kanan", &p.notes.kanan), ("Umum", &p.notes.umum)]
            .iter()
            .filter(|(_, n)| !n.is_empty())
            .map(|(k, n)| format!("{k}: {n}"))
            .collect::<Vec<_>>()
            .join(" | ");
        let len = |side: &str, w: bool| meas(p, side).map(|m| if w { m.fw_mm } else { m.fl_mm });
        let status = if p.cancelled { "dibatalkan" } else if p.completed { "selesai" } else { "belum selesai" };
        let e = c_kes + 1;
        ws.write_string_with_format(r, e, &s.date, &f.cell_c)?;
        ws.write_string_with_format(r, e + 1, &s.place, &f.cell)?;
        ws.write_string_with_format(r, e + 2, &p.operator, &f.cell)?;
        ws.write_string_with_format(r, e + 3, &p.posisi, &f.cell_c)?;
        for (k, v) in [len("right", false), len("left", false), len("right", true), len("left", true)].iter().enumerate() {
            match v {
                Some(v) => ws.write_number_with_format(r, e + 4 + k as u16, *v as f64, &f.num1)?,
                None => ws.write_blank(r, e + 4 + k as u16, &f.cell_c)?,
            };
        }
        ws.write_string_with_format(r, e + 8, &notes, &f.wrap)?;
        ws.write_number_with_format(r, e + 9, p.marks.len() as f64, &f.cell_c)?;
        ws.write_string_with_format(r, e + 10, status, &f.cell_c)?;
        ws.write_string_with_format(r, e + 11, &p.id, &f.cell)?;
        ws.write_string_with_format(r, e + 12, folder, &f.cell)?;
        for (k, side) in ["right", "left"].iter().enumerate() {
            let st = match entry(p, side) {
                Some(en) if en.measurements.is_some() => session::reported_clarke(en).1,
                _ => "",
            };
            ws.write_string_with_format(r, e + 13 + k as u16, st, &f.cell_c)?;
        }
    }
    // --- widths, frozen header
    ws.set_column_width(0, 5)?;
    ws.set_column_width(1, 24)?;
    for c in 2..C_ITEMS {
        ws.set_column_width(c, 8)?;
    }
    for c in C_ITEMS..c_kes {
        ws.set_column_width(c, 7.5)?;
    }
    ws.set_column_width(c_kes, 60)?;
    for k in 0..EXTRA.len() as u16 {
        ws.set_column_width(c_kes + 1 + k, if k == 8 || k == 12 { 40.0 } else { 13.0 })?;
    }
    ws.set_row_height(r1, 45)?;
    ws.set_freeze_panes(r2 + 1, 2)?;
    Ok(())
}

fn write_legend(ws: &mut Worksheet, f: &Fmt) -> Result<(), XlsxError> {
    ws.set_name("Keterangan skor")?;
    ws.write_string_with_format(0, 0, "Keterangan skor (INLOW 60-second foot screen, versi skor klien)", &f.title)?;
    let mut r = 2u32;
    for it in ITEMS.iter() {
        ws.write_string_with_format(r, 0, &format!("{} - {}", it.group, it.label), &f.head)?;
        ws.write_string_with_format(r, 1, &it.options.iter().map(|(p, t)| format!("{p}: {t}")).collect::<Vec<_>>().join("; "), &f.wrap)?;
        r += 1;
    }
    ws.write_string_with_format(r, 0, "4 pertanyaan", &f.head)?;
    ws.write_string_with_format(r, 1, &inlow::QUESTIONS.join(" "), &f.wrap)?;
    r += 2;
    ws.write_string_with_format(r, 0, "Klasifikasi risiko", &f.head)?;
    r += 1;
    for (range, t) in [("0 - 6", 0), ("7 - 12", 7), ("13 - 19", 13), ("20 - 25", 20), ("26 atau lebih", 26)] {
        let c = inlow::risk_class(t);
        ws.write_string_with_format(r, 0, &format!("{} ({}) - skor {range}", c.name, c.letter), &f.cell)?;
        ws.write_string_with_format(r, 1, c.indication, &f.cell)?;
        r += 1;
    }
    r += 1;
    ws.write_string_with_format(r, 0, "Clarke's angle", &f.head)?;
    ws.write_string_with_format(r, 1, "< 31° datar (flat foot); 31-41° normal; > 41° tinggi (cavus). Tabel klien; batas 31° sesuai Pita-Fernández dkk. 2015, J Epidemiol 25(2). Titik sudut ditempatkan aplikasi lalu diperiksa operator; label hanya diberikan bila status \"diperiksa\". Status \"otomatis\" = perkiraan aplikasi, belum diperiksa.", &f.wrap)?;
    r += 1;
    ws.write_string_with_format(r, 0, "Arch index", &f.head)?;
    ws.write_string_with_format(r, 1, "Diukur pada garis tepi kaki di pindaian (termasuk lengkung yang tidak menapak), sehingga nilainya lebih tinggi daripada jejak kaki tinta. Karena itu batas Cavanagh & Rodgers 1987 (J Biomech 20(5); ≤ 0,21 tinggi, ≥ 0,26 datar) TIDAK dipakai untuk memberi label.", &f.wrap)?;
    r += 2;
    ws.write_string_with_format(r, 0, "Catatan", &f.head)?;
    ws.write_string_with_format(r, 1, inlow::SCORING_NOTES, &f.wrap)?;
    ws.set_column_width(0, 38)?;
    ws.set_column_width(1, 100)?;
    Ok(())
}

fn save(rows: &[(&Sesi, String, &Patient)], title: &str, path: &Path) -> Result<(), String> {
    let f = formats();
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    write_sheet(ws, title, rows, &f).map_err(|e| e.to_string())?;
    let ws2 = wb.add_worksheet();
    write_legend(ws2, &f).map_err(|e| e.to_string())?;
    // write next to the target, then replace it; a file open in Excel cannot be replaced
    let tmp = path.with_extension("xlsx.tmp");
    wb.save(&tmp).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("FILE_OPEN: {} could not be updated (is it open in Excel?): {e}", path.file_name().unwrap_or_default().to_string_lossy())
    })
}

pub fn sesi_xlsx_name(sesi_dir: &Path) -> String {
    format!("Rekap sesi {}.xlsx", sesi_dir.file_name().unwrap_or_default().to_string_lossy())
}

/// Rebuild the session file and the master file. Errors are collected, not fatal: a file open in
/// Excel must never stop the nurse from scanning (the files are rebuilt again on the next save).
pub fn rebuild(data_dir: &Path, sesi_dir: Option<&Path>) -> Vec<String> {
    let mut errs = vec![];
    let all = session::list_sesi(data_dir);
    let patients: Vec<(PathBuf, Sesi, Vec<(PathBuf, Patient)>)> = all.into_iter().map(|(d, s)| { let p = session::list_patients(&d); (d, s, p) }).collect();
    if let Some(sd) = sesi_dir {
        if let Some((d, s, ps)) = patients.iter().find(|(d, _, _)| d == sd) {
            let rows: Vec<(&Sesi, String, &Patient)> = ps.iter().map(|(pd, p)| (s, session::relative(data_dir, pd), p)).collect();
            let title = format!("Rekap skor pengkajian kaki - {}, {}", s.place, s.date);
            if let Err(e) = save(&rows, &title, &d.join(sesi_xlsx_name(d))) {
                errs.push(e);
            }
        }
    }
    let rows: Vec<(&Sesi, String, &Patient)> = patients.iter().flat_map(|(_, s, ps)| ps.iter().map(move |(pd, p)| (s, session::relative(data_dir, pd), p))).collect();
    if let Err(e) = save(&rows, "Rekap skor pengkajian kaki - semua sesi", &data_dir.join(session::MASTER_XLSX)) {
        errs.push(e);
    }
    if let Err(e) = session::rebuild_csv(data_dir) {
        errs.push(e);
    }
    errs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sf_xl_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn conclusion_reads_like_the_agreed_example() {
        let d = tmp("kes");
        let (sd, _) = start_sesi(&d, "X", "Rina", "7MY").unwrap();
        let dir = create_patient(&sd, &PatientForm { nama: "Siti".into(), usia: Some(41), kelamin: "Perempuan".into(), td: "".into(), bb: None, posisi: "berdiri".into() }, "dev").unwrap();
        let mut p = read_patient(&dir).unwrap();
        assert_eq!(kesimpulan(&p), "Kaki kanan: Belum dipindai. INLOW belum lengkap. Kaki kiri: Belum dipindai. INLOW belum lengkap.");
        assert!(saran(&p).is_empty(), "no advice before any INLOW is complete");
        for it in ITEMS.iter() {
            p.inlow.kanan.items.insert(it.key.into(), it.options[0].0);
        }
        p.inlow.kanan.items.insert("deformitas".into(), 4);
        p.inlow.kanan.items.insert("monofilamen".into(), 4);
        p.inlow.kanan.items.insert("alas_kaki".into(), 1); // 1 (nails) + 4 + 4 + 1 = 10 -> B
        let m = crate::measure::Measurements {
            scale: crate::measure::Scale::calibrated(),
            fl_mm: 245.0, fw_mm: 94.0, area_cm2: 150.0, toeless_area_cm2: 130.0, hind_cm2: 40.0, mid_cm2: 31.0, fore_cm2: 59.0,
            ai: 0.2384, csi_pct: 40.0, toe_base_method: "gap".into(), toeless_frac: 0.75,
            heel_centre: [0.0; 2], toe_tip: [0.0; 2], heel_end: [0.0; 2], fw_line: [[0.0; 2]; 2], toe_base: [0.0; 2], corrected: false,
            clarke_deg: Some(35.2), clarke_pts: None, clarke_source: None, contact: None,
        };
        p.scans.push(ScanEntry { file: raw_scan_name("right"), side: "right".into(), attempt: 1, seconds: 12.0, width_px: 1, height_px: 1, dpi: 300, rotated_180: true, accepted_at: now_iso(), warnings: vec![], measurements: Some(m), check: None, check_history: vec![], images_version: None });
        // the app's guess alone never gets a label
        assert_eq!(foot_summary(&p, "right"), "Lengkung belum dapat ditentukan (titik belum diperiksa). Arch Index 0,24. Risiko rendah (B)");
        assert_eq!(saran(&p).first().map(String::as_str), Some("Penggunaan alas kaki yang tepat"));
        let chk = |status: &str, deg: Option<f32>, reason: Option<&str>| PointCheck { status: status.into(), skip_reason: reason.map(Into::into), operator: "Rina".into(), at: now_iso(), clarke_pts: None, clarke_deg: deg, clarke_moved_mm: None, ai_moved_mm: None, guess_source: None };
        p.scans[0].check = Some(chk("dilewati", None, Some("antrean panjang")));
        assert_eq!(foot_summary(&p, "right"), "Lengkung belum dapat ditentukan (titik belum diperiksa). Arch Index 0,24. Risiko rendah (B)");
        assert_eq!(unchecked_sides(&p), vec!["right".to_string()]);
        p.scans[0].check = Some(chk("diperiksa", Some(38.6), None));
        assert_eq!(foot_summary(&p, "right"), "Lengkung normal (sudut Clarke 39°). Arch Index 0,24. Risiko rendah (B)");
        assert!(kesimpulan(&p).starts_with("Kaki kanan: Lengkung normal (sudut Clarke 39°). Arch Index 0,24. Risiko rendah (B). Kaki kiri: Belum dipindai. INLOW belum lengkap"));
        assert_eq!(arch_words(25.0), "datar / flat foot");
        assert_eq!(arch_words(50.0), "tinggi / cavus");
        assert!(unchecked_sides(&p).is_empty());
    }

    #[test]
    fn workbooks_are_written_and_an_open_file_is_reported_not_fatal() {
        let d = tmp("wb");
        let (sd, _) = start_sesi(&d, "Puskesmas Beji", "Rina", "7MY").unwrap();
        for n in ["Siti Aminah", "=HYPERLINK(\"x\")"] {
            create_patient(&sd, &PatientForm { nama: n.into(), usia: Some(41), kelamin: "Perempuan".into(), td: "100/70".into(), bb: Some(54.0), posisi: "berdiri".into() }, "dev").unwrap();
        }
        assert!(rebuild(&d, Some(&sd)).is_empty());
        let sesi_file = sd.join(sesi_xlsx_name(&sd));
        assert!(sesi_file.exists() && d.join(MASTER_XLSX).exists() && d.join(RESEARCH_CSV).exists());
        assert!(std::fs::metadata(&sesi_file).unwrap().len() > 3000);
        // a folder in the way of the master file stands in for "open in Excel": reported, others still written
        std::fs::remove_file(d.join(MASTER_XLSX)).unwrap();
        std::fs::create_dir(d.join(MASTER_XLSX)).unwrap();
        let errs = rebuild(&d, Some(&sd));
        assert_eq!(errs.len(), 1);
        assert!(errs[0].starts_with("FILE_OPEN"));
        assert!(!d.join("Rekap semua sesi.xlsx.tmp").exists());
    }
}
