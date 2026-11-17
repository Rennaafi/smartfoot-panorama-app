//! Settings, screening sessions and patient records on disk.
//!
//! Layout (agreed 17 Nov 2026):
//!
//! ```text
//! <data_dir>\
//!   Rekap semua sesi.xlsx                  every session, rebuilt from the patient files
//!   Data pengukuran (riset).csv            one row per accepted scan, all numbers (research)
//!   2026-11-17 Puskesmas Beji\             one screening session = one day at one place
//!     sesi.json
//!     Rekap sesi 2026-11-17 Puskesmas Beji.xlsx
//!     Operator Rina\
//!       001 Siti Aminah\
//!         data pasien.json
//!         Kaki kiri - pindaian asli.png    accepted scans (right foot already turned 180°)
//!         Kaki kanan - pindaian asli.png
//!         Kaki kiri - indeks lengkung.png  analysis images (written by the analysis step)
//!         _pending\                        scan waiting for Accept / Retake
//!         ditolak\                         retaken scans, kept, never silently deleted
//! ```
//!
//! The session is the day; the operator can change during it (each patient records who scanned
//! them). The left foot is scanned first with the patient facing the normal way, then the patient
//! turns around for the right foot, so the right scan is turned 180° before it is saved.

use crate::inlow::Inlow;
use crate::measure::Measurements;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

pub const SESI_FILE: &str = "sesi.json";
pub const PATIENT_FILE: &str = "data pasien.json";
pub const MASTER_XLSX: &str = "Rekap semua sesi.xlsx";
pub const RESEARCH_CSV: &str = "Data pengukuran (riset).csv";
pub const PENDING: &str = "_pending";
pub const REJECTED: &str = "ditolak";

/// Scan order: left foot (patient facing the normal way), then right foot (patient turned around).
pub const SIDES: [&str; 2] = ["left", "right"];

pub fn side_id(side: &str) -> &'static str {
    if side == "right" { "kanan" } else { "kiri" }
}

/// File name of an accepted raw scan, e.g. "Kaki kiri - pindaian asli.png".
pub fn raw_scan_name(side: &str) -> String {
    format!("Kaki {} - pindaian asli.png", side_id(side))
}

// ------------------------------------------------------------------------------------------------
// Settings

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub data_dir: String,
    /// Exact WIA device name, or None = pick the LiDE 300 driverless route automatically.
    pub scanner: Option<String>,
    /// Short code that makes internal patient IDs unique per laptop.
    #[serde(default)]
    pub station: String,
    /// Folder for the second copy of all data (None = no second copy).
    #[serde(default)]
    pub backup_dir: Option<String>,
    /// The first-run screen ("where is the data saved?") has been completed on this laptop.
    #[serde(default)]
    pub setup_done: bool,
    /// Operator names used before on this laptop, most recent first (for the quick pick list).
    #[serde(default)]
    pub operators: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = if Path::new("D:\\").exists() {
            "D:\\Data SmartFoot".to_string()
        } else {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into());
            format!("{home}\\Documents\\Data SmartFoot")
        };
        Settings { data_dir, scanner: None, station: String::new(), backup_dir: None, setup_done: false, operators: vec![] }
    }
}

/// Read the settings. A missing file gives defaults. A file that exists but cannot be understood
/// (hand-edited, damaged) is first copied to `settings.json.bad` so the original values are never
/// lost when defaults replace it; a BOM itself is tolerated. Fields of older versions (e.g.
/// "rounds") are ignored.
pub fn load_settings(file: &Path) -> Settings {
    let Ok(text) = std::fs::read_to_string(file) else { return Settings::default() };
    match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(s) => s,
        Err(_) => {
            let _ = std::fs::write(file.with_extension("json.bad"), &text);
            Settings::default()
        }
    }
}

/// Check and tidy settings coming from the UI.
pub fn validate_settings(mut new: Settings) -> Result<Settings, String> {
    new.station = clean_station(&new.station);
    if new.station.is_empty() {
        return Err("BAD_STATION: the laptop code needs 1-4 letters or digits".into());
    }
    new.data_dir = new.data_dir.trim().to_string();
    if new.data_dir.is_empty() || !Path::new(&new.data_dir).is_absolute() {
        return Err("BAD_DIR: the data folder must be a full path such as E:\\Data SmartFoot".into());
    }
    new.backup_dir = new.backup_dir.take().map(|b| b.trim().to_string()).filter(|b| !b.is_empty());
    if let Some(b) = &new.backup_dir {
        if !Path::new(b).is_absolute() {
            return Err("BAD_DIR: the backup folder must be a full path such as C:\\SmartFoot_Backup".into());
        }
    }
    new.operators = new.operators.iter().filter_map(|o| clean_person(o).ok()).collect();
    new.operators.dedup();
    new.operators.truncate(20);
    Ok(new)
}

pub fn save_settings(file: &Path, s: &Settings) -> Result<(), String> {
    if let Some(d) = file.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    std::fs::write(file, serde_json::to_string_pretty(s).unwrap()).map_err(|e| e.to_string())
}

/// Put `name` first in the remembered operator list.
pub fn remember_operator(s: &mut Settings, name: &str) {
    s.operators.retain(|o| !o.eq_ignore_ascii_case(name));
    s.operators.insert(0, name.to_string());
    s.operators.truncate(20);
}

/// Keep a station code to 1-4 upper-case letters or digits.
pub fn clean_station(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).take(4).collect()
}

/// A random-looking 3-character code (no look-alike characters) for a laptop that has none yet.
pub fn generate_station() -> String {
    const A: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) ^ (std::process::id() as u64).wrapping_mul(7919);
    let mut x = n.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (0..3).map(|_| { x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); A[((x >> 33) as usize) % A.len()] as char }).collect()
}

// ------------------------------------------------------------------------------------------------
// Names and folder names

pub fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// A person's name or a place as typed: trimmed, inner spaces collapsed, no control characters,
/// 1-60 characters, must contain a letter or digit. Anything typed is allowed otherwise (it goes into
/// JSON and Excel as text, never as a formula).
pub fn clean_person(raw: &str) -> Result<String, String> {
    let s: String = raw.chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
    if !s.chars().any(|c| c.is_alphanumeric()) {
        return Err("EMPTY_NAME: the name is empty".into());
    }
    if s.chars().count() > 60 {
        return Err("LONG_NAME: the name is longer than 60 characters".into());
    }
    Ok(s)
}

const RESERVED: [&str; 22] = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];

/// Make text safe as (part of) a Windows folder name: characters Windows forbids become spaces,
/// no leading/trailing dots or spaces, at most 60 characters, never a reserved device name.
pub fn folder_safe(raw: &str) -> String {
    let s: String = raw.chars().map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { ' ' } else { c }).collect();
    let mut s: String = s.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(60).collect();
    s = s.trim_matches(|c: char| c == '.' || c == ' ').to_string();
    if s.is_empty() {
        s = "tanpa nama".into();
    }
    if RESERVED.contains(&s.to_ascii_uppercase().as_str()) {
        s = format!("_{s}");
    }
    s
}

/// A path given by the UI (relative to the data folder) turned back into a folder that is surely
/// inside the data folder: only plain name parts, no "..", no drive, no root.
pub fn resolve(data_dir: &Path, rel: &str) -> Result<PathBuf, String> {
    let p = Path::new(rel);
    if rel.trim().is_empty() || !p.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err("BAD_PATH: not a folder inside the data folder".into());
    }
    Ok(data_dir.join(p))
}

pub fn relative(data_dir: &Path, p: &Path) -> String {
    p.strip_prefix(data_dir).unwrap_or(p).to_string_lossy().to_string()
}

fn write_json<T: Serialize>(file: &Path, v: &T) -> Result<(), String> {
    // via a temp file so a crash never leaves a half-written file
    let tmp = file.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(v).unwrap()).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, file).map_err(|e| e.to_string())
}

// ------------------------------------------------------------------------------------------------
// Screening session (one day, one place)

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Shift {
    pub operator: String,
    pub from: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Pause {
    pub start: String,
    pub end: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sesi {
    /// YYYY-MM-DD, the day the session belongs to.
    pub date: String,
    pub place: String,
    pub started: String,
    /// Operator at work now.
    pub operator: String,
    /// Every operator change, in order (the first entry is the starting operator).
    pub shifts: Vec<Shift>,
    #[serde(default)]
    pub pauses: Vec<Pause>,
    pub ended: Option<String>,
    /// Number the next patient gets (001, 002, ...), counted over the whole session.
    pub next_no: u32,
    pub station: String,
    pub app_version: String,
}

impl Sesi {
    pub fn paused(&self) -> bool {
        self.pauses.last().map(|p| p.end.is_none()).unwrap_or(false)
    }
}

pub fn read_sesi(dir: &Path) -> Result<Sesi, String> {
    let s = std::fs::read_to_string(dir.join(SESI_FILE)).map_err(|e| format!("cannot read {SESI_FILE}: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("{SESI_FILE} is damaged: {e}"))
}

pub fn write_sesi(dir: &Path, s: &Sesi) -> Result<(), String> {
    write_json(&dir.join(SESI_FILE), s)
}

/// All session folders in the data folder (those holding a readable sesi.json), oldest first.
pub fn list_sesi(data_dir: &Path) -> Vec<(PathBuf, Sesi)> {
    let mut v: Vec<(PathBuf, Sesi)> = std::fs::read_dir(data_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| read_sesi(&e.path()).ok().map(|s| (e.path(), s)))
        .collect();
    v.sort_by(|a, b| a.1.started.cmp(&b.1.started));
    v
}

/// The session that is still open (not ended), newest if somehow there are several.
pub fn open_sesi(data_dir: &Path) -> Option<(PathBuf, Sesi)> {
    list_sesi(data_dir).into_iter().rev().find(|(_, s)| s.ended.is_none())
}

pub fn start_sesi(data_dir: &Path, place: &str, operator: &str, station: &str) -> Result<(PathBuf, Sesi), String> {
    if let Some((d, _)) = open_sesi(data_dir) {
        return Err(format!("SESI_OPEN: session {} is still open", relative(data_dir, &d)));
    }
    let place = clean_person(place).map_err(|e| e.replace("name", "place"))?;
    let operator = clean_person(operator)?;
    let date = today();
    let base = format!("{date} {}", folder_safe(&place));
    // a second session at the same place on the same day gets "(2)", "(3)", ...
    let mut dir = data_dir.join(&base);
    let mut k = 2;
    while dir.exists() {
        dir = data_dir.join(format!("{base} ({k})"));
        k += 1;
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create session folder: {e}"))?;
    let now = now_iso();
    let s = Sesi {
        date,
        place,
        started: now.clone(),
        operator: operator.clone(),
        shifts: vec![Shift { operator, from: now }],
        pauses: vec![],
        ended: None,
        next_no: 1,
        station: station.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    write_sesi(&dir, &s)?;
    Ok((dir, s))
}

pub fn change_operator(dir: &Path, operator: &str) -> Result<Sesi, String> {
    let operator = clean_person(operator)?;
    let mut s = read_sesi(dir)?;
    if s.operator != operator {
        s.operator = operator.clone();
        s.shifts.push(Shift { operator, from: now_iso() });
        write_sesi(dir, &s)?;
    }
    Ok(s)
}

/// Start (true) or end (false) a break. Repeating the same call changes nothing.
pub fn set_paused(dir: &Path, pause: bool) -> Result<Sesi, String> {
    let mut s = read_sesi(dir)?;
    if pause && !s.paused() {
        s.pauses.push(Pause { start: now_iso(), end: None });
    } else if !pause && s.paused() {
        if let Some(p) = s.pauses.last_mut() {
            p.end = Some(now_iso());
        }
    }
    write_sesi(dir, &s)?;
    Ok(s)
}

pub fn end_sesi(dir: &Path) -> Result<Sesi, String> {
    let mut s = read_sesi(dir)?;
    if s.paused() {
        if let Some(p) = s.pauses.last_mut() {
            p.end = Some(now_iso());
        }
    }
    if s.ended.is_none() {
        s.ended = Some(now_iso());
    }
    write_sesi(dir, &s)?;
    Ok(s)
}

// ------------------------------------------------------------------------------------------------
// Patient

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanEntry {
    /// File name inside the patient folder.
    pub file: String,
    /// "left" or "right".
    pub side: String,
    pub attempt: u32,
    pub seconds: f64,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    /// The saved image was turned 180° from what the scanner delivered (right foot, patient turned).
    pub rotated_180: bool,
    pub accepted_at: String,
    /// Plain-language warnings the operator accepted anyway (empty = clean).
    pub warnings: Vec<String>,
    /// Filled in right after Accept; None if no foot could be measured.
    #[serde(default)]
    pub measurements: Option<Measurements>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Notes {
    #[serde(default)]
    pub kiri: String,
    #[serde(default)]
    pub kanan: String,
    /// About the patient as a whole.
    #[serde(default)]
    pub umum: String,
}

/// A spot the operator marked by hand (e.g. suspected callus), full-resolution pixels of the saved scan.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Mark {
    pub side: String,
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Patient {
    /// Number within the session (001 ...).
    pub no: u32,
    /// Internal ID, never typed: SFP-<station>-<yyyymmdd>-<no>.
    pub id: String,
    pub nama: String,
    pub usia: u32,
    /// "Laki-laki" or "Perempuan".
    pub kelamin: String,
    /// Blood pressure as typed, e.g. "120/80" (empty = not measured).
    pub td: String,
    /// Body weight in kg.
    pub bb: Option<f32>,
    /// "berdiri" or "duduk".
    pub posisi: String,
    pub operator: String,
    pub created: String,
    pub scanner: String,
    pub app_version: String,
    /// The ruler calibration in force when the patient was scanned.
    pub calibration: Option<serde_json::Value>,
    pub scans: Vec<ScanEntry>,
    #[serde(default)]
    pub notes: Notes,
    #[serde(default)]
    pub inlow: Inlow,
    #[serde(default)]
    pub marks: Vec<Mark>,
    pub completed: bool,
    #[serde(default)]
    pub cancelled: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PatientForm {
    pub nama: String,
    pub usia: Option<u32>,
    pub kelamin: String,
    #[serde(default)]
    pub td: String,
    pub bb: Option<f32>,
    pub posisi: String,
}

/// Check the patient form. Everything is checked here, whatever the UI did or did not check.
pub fn validate_patient(f: &PatientForm) -> Result<PatientForm, String> {
    let nama = clean_person(&f.nama)?;
    let usia = f.usia.ok_or("EMPTY_AGE: the age is empty")?;
    if usia > 120 {
        return Err("BAD_AGE: the age must be between 0 and 120".into());
    }
    if f.kelamin != "Laki-laki" && f.kelamin != "Perempuan" {
        return Err("BAD_SEX: kelamin must be Laki-laki or Perempuan".into());
    }
    let td: String = f.td.chars().filter(|c| !c.is_whitespace()).collect();
    if !td.is_empty() {
        let ok = td.split_once('/').map(|(a, b)| {
            let (a, b) = (a.parse::<u32>(), b.parse::<u32>());
            matches!((a, b), (Ok(s), Ok(d)) if (50..=300).contains(&s) && (20..=200).contains(&d) && s > d)
        });
        if ok != Some(true) {
            return Err("BAD_TD: blood pressure must look like 120/80".into());
        }
    }
    if let Some(b) = f.bb {
        if !(20.0..=250.0).contains(&b) || !b.is_finite() {
            return Err("BAD_BB: weight must be between 20 and 250 kg".into());
        }
    }
    if f.posisi != "berdiri" && f.posisi != "duduk" {
        return Err("BAD_POSISI: posisi must be berdiri or duduk".into());
    }
    Ok(PatientForm { nama, usia: Some(usia), kelamin: f.kelamin.clone(), td, bb: f.bb.map(|b| (b * 10.0).round() / 10.0), posisi: f.posisi.clone() })
}

pub fn read_patient(dir: &Path) -> Result<Patient, String> {
    let s = std::fs::read_to_string(dir.join(PATIENT_FILE)).map_err(|e| format!("cannot read {PATIENT_FILE}: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("{PATIENT_FILE} is damaged: {e}"))
}

pub fn write_patient(dir: &Path, p: &Patient) -> Result<(), String> {
    write_json(&dir.join(PATIENT_FILE), p)
}

/// Create the next patient of the open session under the current operator's folder.
pub fn create_patient(sesi_dir: &Path, form: &PatientForm, scanner: &str) -> Result<PathBuf, String> {
    let f = validate_patient(form)?;
    let mut s = read_sesi(sesi_dir)?;
    if s.ended.is_some() {
        return Err("SESI_ENDED: this session has ended".into());
    }
    let no = s.next_no;
    let op_dir = sesi_dir.join(format!("Operator {}", folder_safe(&s.operator)));
    let dir = op_dir.join(format!("{no:03} {}", folder_safe(&f.nama)));
    if dir.exists() {
        return Err(format!("ID_USED: {} already exists", dir.display()));
    }
    for sub in [PENDING, REJECTED] {
        std::fs::create_dir_all(dir.join(sub)).map_err(|e| format!("cannot create folder: {e}"))?;
    }
    let p = Patient {
        no,
        id: format!("SFP-{}-{}-{no:03}", if s.station.is_empty() { "X" } else { &s.station }, s.date.replace('-', "")),
        nama: f.nama,
        usia: f.usia.unwrap_or(0),
        kelamin: f.kelamin,
        td: f.td,
        bb: f.bb,
        posisi: f.posisi,
        operator: s.operator.clone(),
        created: now_iso(),
        scanner: scanner.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        calibration: Some(crate::measure::calibration_record()),
        scans: vec![],
        notes: Notes::default(),
        inlow: Inlow::default(),
        marks: vec![],
        completed: false,
        cancelled: false,
    };
    write_patient(&dir, &p)?;
    // count up only after the patient exists, so a failed create never skips a number
    s.next_no = no + 1;
    write_sesi(sesi_dir, &s)?;
    Ok(dir)
}

/// Correct the patient form later (typo in the name, BB added...). The folder keeps its name.
pub fn update_patient_form(dir: &Path, form: &PatientForm) -> Result<Patient, String> {
    let f = validate_patient(form)?;
    let mut p = read_patient(dir)?;
    p.nama = f.nama;
    p.usia = f.usia.unwrap_or(0);
    p.kelamin = f.kelamin;
    p.td = f.td;
    p.bb = f.bb;
    p.posisi = f.posisi;
    write_patient(dir, &p)?;
    Ok(p)
}

pub fn pending_file(dir: &Path, side: &str) -> PathBuf {
    dir.join(PENDING).join(format!("{side}.png"))
}

/// Move a pending scan into the patient folder and record it.
pub fn accept(dir: &Path, side: &str, seconds: f64, w: u32, h: u32, rotated: bool, warnings: Vec<String>) -> Result<(), String> {
    let from = pending_file(dir, side);
    let name = raw_scan_name(side);
    let to = dir.join(&name);
    if to.exists() {
        // an earlier accepted scan of this foot (accepted, then retaken from the patient screen)
        move_to_rejected(dir, &to, side)?;
    }
    std::fs::rename(&from, &to).map_err(|e| format!("cannot save scan: {e}"))?;
    // if recording it fails (disk full, file locked...), put the scan back so the operator can
    // press Accept again; a scan must never sit in the folder without an entry in the patient file
    record_accepted(dir, side, &name, seconds, w, h, rotated, warnings).map_err(|e| {
        let _ = std::fs::rename(&to, &from);
        e
    })
}

fn rejected_count(dir: &Path, side: &str) -> usize {
    let prefix = format!("Kaki {} - percobaan ", side_id(side));
    std::fs::read_dir(dir.join(REJECTED)).map(|rd| rd.flatten().filter(|e| e.file_name().to_string_lossy().starts_with(&prefix)).count()).unwrap_or(0)
}

fn move_to_rejected(dir: &Path, from: &Path, side: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir.join(REJECTED)).map_err(|e| e.to_string())?;
    let n = rejected_count(dir, side) + 1;
    std::fs::rename(from, dir.join(REJECTED).join(format!("Kaki {} - percobaan {n}.png", side_id(side)))).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
fn record_accepted(dir: &Path, side: &str, name: &str, seconds: f64, w: u32, h: u32, rotated: bool, warnings: Vec<String>) -> Result<(), String> {
    let mut p = read_patient(dir)?;
    let attempt = 1 + rejected_count(dir, side) as u32;
    p.scans.retain(|e| e.side != side);
    p.scans.push(ScanEntry {
        file: name.to_string(),
        side: side.to_string(),
        attempt,
        seconds,
        width_px: w,
        height_px: h,
        dpi: 300,
        rotated_180: rotated,
        accepted_at: now_iso(),
        warnings,
        measurements: None,
    });
    p.scans.sort_by_key(|e| SIDES.iter().position(|s| *s == e.side));
    write_patient(dir, &p)
}

/// Store (or replace) the measurements of one accepted scan.
pub fn set_measurements(dir: &Path, side: &str, m: Option<Measurements>) -> Result<(), String> {
    let mut p = read_patient(dir)?;
    let e = p.scans.iter_mut().find(|e| e.side == side).ok_or("scan not found in patient file")?;
    e.measurements = m;
    write_patient(dir, &p)
}

/// Keep a retaken (pending) scan under ditolak\ with a running number.
pub fn reject(dir: &Path, side: &str) -> Result<(), String> {
    let from = pending_file(dir, side);
    if !from.exists() {
        return Ok(());
    }
    move_to_rejected(dir, &from, side)
}

pub fn set_notes(dir: &Path, notes: &Notes) -> Result<Patient, String> {
    let clean = |s: &str| -> String { s.chars().map(|c| if c.is_control() && c != '\n' { ' ' } else { c }).take(1000).collect::<String>().trim().to_string() };
    let mut p = read_patient(dir)?;
    p.notes = Notes { kiri: clean(&notes.kiri), kanan: clean(&notes.kanan), umum: clean(&notes.umum) };
    write_patient(dir, &p)?;
    Ok(p)
}

pub fn set_inlow(dir: &Path, inl: Inlow) -> Result<Patient, String> {
    let inl = Inlow { kanan: crate::inlow::clean(inl.kanan)?, kiri: crate::inlow::clean(inl.kiri)? };
    let mut p = read_patient(dir)?;
    p.inlow = inl;
    write_patient(dir, &p)?;
    Ok(p)
}

pub fn set_marks(dir: &Path, marks: Vec<Mark>) -> Result<Patient, String> {
    let mut p = read_patient(dir)?;
    for m in &marks {
        if !SIDES.contains(&m.side.as_str()) || ![m.x, m.y, m.r].iter().all(|v| v.is_finite() && *v >= 0.0) {
            return Err("BAD_MARK: a mark is outside the image".into());
        }
    }
    p.marks = marks
        .into_iter()
        .take(50)
        .map(|mut m| {
            m.label = m.label.chars().filter(|c| !c.is_control()).take(80).collect::<String>().trim().to_string();
            m
        })
        .collect();
    write_patient(dir, &p)?;
    Ok(p)
}

/// Cancel a patient. One with no scans at all is removed (we created that empty folder; the number
/// is not reused); anything with scans is kept and marked cancelled. Returns true if removed.
pub fn cancel(dir: &Path) -> Result<bool, String> {
    let mut p = read_patient(dir)?;
    let has_files = |sub: &str| std::fs::read_dir(dir.join(sub)).map(|mut r| r.next().is_some()).unwrap_or(false);
    if p.scans.is_empty() && !has_files(REJECTED) && !has_files(PENDING) {
        std::fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
        // an operator folder left empty goes too
        if let Some(op) = dir.parent() {
            if std::fs::read_dir(op).map(|mut r| r.next().is_none()).unwrap_or(false) {
                let _ = std::fs::remove_dir(op);
            }
        }
        return Ok(true);
    }
    p.cancelled = true;
    write_patient(dir, &p)?;
    Ok(false)
}

pub fn complete(dir: &Path) -> Result<(), String> {
    let mut p = read_patient(dir)?;
    p.completed = true;
    write_patient(dir, &p)
}

/// The next foot to scan ("left" then "right"), or None when both are accepted.
pub fn next_side(p: &Patient) -> Option<&'static str> {
    SIDES.iter().copied().find(|s| !p.scans.iter().any(|e| e.side == *s))
}

/// Every patient folder of a session: (folder, patient), in number order. Unreadable ones are skipped
/// here and reported by `damaged_patients`.
pub fn list_patients(sesi_dir: &Path) -> Vec<(PathBuf, Patient)> {
    let mut v: Vec<(PathBuf, Patient)> = patient_dirs(sesi_dir).into_iter().filter_map(|d| read_patient(&d).ok().map(|p| (d, p))).collect();
    v.sort_by_key(|(_, p)| p.no);
    v
}

/// Folders two levels below a session folder that look like patient folders.
fn patient_dirs(sesi_dir: &Path) -> Vec<PathBuf> {
    let mut v = vec![];
    for op in std::fs::read_dir(sesi_dir).into_iter().flatten().flatten() {
        if !op.path().is_dir() {
            continue;
        }
        for pd in std::fs::read_dir(op.path()).into_iter().flatten().flatten() {
            let p = pd.path();
            if p.is_dir() && (p.join(PATIENT_FILE).exists() || p.join(PENDING).exists()) {
                v.push(p);
            }
        }
    }
    v
}

/// Patient folders whose data file cannot be read. Such data is never deleted; the app warns.
pub fn damaged_patients(data_dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = list_sesi(data_dir)
        .iter()
        .flat_map(|(d, _)| patient_dirs(d))
        .filter(|d| read_patient(d).is_err())
        .map(|d| relative(data_dir, &d))
        .collect();
    v.sort();
    v
}

/// The newest patient of the session that was started but neither finished nor cancelled.
pub fn find_unfinished(sesi_dir: &Path) -> Option<(PathBuf, Patient)> {
    list_patients(sesi_dir).into_iter().rev().find(|(_, p)| !p.completed && !p.cancelled)
}

// ------------------------------------------------------------------------------------------------
// Research CSV (all numbers, one row per accepted scan)

fn csv_field(s: &str) -> String {
    let s = &if s.starts_with(['=', '+', '-', '@']) && s.parse::<f64>().is_err() { format!("'{s}") } else { s.to_string() };
    if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

/// Rebuild the research CSV from every patient file, so it can never disagree with them.
pub fn rebuild_csv(data_dir: &Path) -> Result<PathBuf, String> {
    let mut out = String::from("sesi,tempat,no,id,nama,usia,kelamin,posisi,operator,side,attempt,file,rotated_180,calibrated,mm_per_px_x,mm_per_px_y,fl_mm,fw_mm,area_cm2,toeless_area_cm2,hind_cm2,mid_cm2,fore_cm2,ai,csi_pct,clarke_deg,toe_base_method,toeless_frac,points_corrected,warnings\n");
    for (sd, s) in list_sesi(data_dir) {
        for (_, p) in list_patients(&sd) {
            for e in &p.scans {
                let mut f: Vec<String> = vec![relative(data_dir, &sd), s.place.clone(), format!("{:03}", p.no), p.id.clone(), p.nama.clone(), p.usia.to_string(), p.kelamin.clone(), p.posisi.clone(), p.operator.clone(), e.side.clone(), e.attempt.to_string(), e.file.clone(), e.rotated_180.to_string()];
                match &e.measurements {
                    Some(m) => f.extend([
                        m.scale.calibrated.to_string(),
                        format!("{:.5}", m.scale.x_mm_per_px),
                        format!("{:.5}", m.scale.y_mm_per_px),
                        format!("{:.1}", m.fl_mm),
                        format!("{:.1}", m.fw_mm),
                        format!("{:.1}", m.area_cm2),
                        format!("{:.1}", m.toeless_area_cm2),
                        format!("{:.1}", m.hind_cm2),
                        format!("{:.1}", m.mid_cm2),
                        format!("{:.1}", m.fore_cm2),
                        format!("{:.3}", m.ai),
                        format!("{:.1}", m.csi_pct),
                        m.clarke_deg.map(|c| format!("{c:.1}")).unwrap_or_default(),
                        m.toe_base_method.clone(),
                        format!("{:.3}", m.toeless_frac),
                        m.corrected.to_string(),
                    ]),
                    None => f.extend(std::iter::repeat_n(String::new(), 16)),
                }
                f.push(e.warnings.join(";"));
                out.push_str(&f.iter().map(|x| csv_field(x)).collect::<Vec<_>>().join(","));
                out.push('\n');
            }
        }
    }
    let path = data_dir.join(RESEARCH_CSV);
    // BOM so Excel opens names with accents correctly
    std::fs::write(&path, format!("\u{feff}{out}")).map_err(|e| format!("cannot write {RESEARCH_CSV}: {e}"))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sf_test_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    pub fn form(nama: &str) -> PatientForm {
        PatientForm { nama: nama.into(), usia: Some(41), kelamin: "Perempuan".into(), td: "100/70".into(), bb: Some(54.0), posisi: "berdiri".into() }
    }

    #[test]
    fn session_patients_and_operators_land_in_the_agreed_folders() {
        let d = tmp("layout");
        let (sd, s) = start_sesi(&d, "Puskesmas Beji", " Rina ", "7MY").unwrap();
        assert_eq!(sd.file_name().unwrap().to_string_lossy(), format!("{} Puskesmas Beji", today()));
        assert_eq!(s.operator, "Rina");
        let p1 = create_patient(&sd, &form("Siti Aminah"), "dev").unwrap();
        assert!(p1.ends_with("Operator Rina/001 Siti Aminah"));
        change_operator(&sd, "Budi").unwrap();
        let p2 = create_patient(&sd, &form("Siti Aminah"), "dev").unwrap(); // same name, different number
        assert!(p2.ends_with("Operator Budi/002 Siti Aminah"));
        let p = read_patient(&p2).unwrap();
        assert_eq!((p.no, p.operator.as_str()), (2, "Budi"));
        assert_eq!(p.id, format!("SFP-7MY-{}-002", today().replace('-', "")));
        assert_eq!(list_patients(&sd).len(), 2);
        let s = read_sesi(&sd).unwrap();
        assert_eq!(s.shifts.len(), 2);
        assert_eq!(s.next_no, 3);
        // only one open session at a time; a second one the same day at the same place gets (2)
        assert!(start_sesi(&d, "Puskesmas Beji", "Rina", "7MY").unwrap_err().starts_with("SESI_OPEN"));
        end_sesi(&sd).unwrap();
        assert!(open_sesi(&d).is_none());
        assert!(create_patient(&sd, &form("X"), "dev").unwrap_err().starts_with("SESI_ENDED"));
        let (sd2, _) = start_sesi(&d, "Puskesmas Beji", "Rina", "7MY").unwrap();
        assert!(sd2.file_name().unwrap().to_string_lossy().ends_with("(2)"));
    }

    #[test]
    fn breaks_open_and_close() {
        let d = tmp("pause");
        let (sd, _) = start_sesi(&d, "Posyandu", "Rina", "7MY").unwrap();
        assert!(set_paused(&sd, true).unwrap().paused());
        assert_eq!(set_paused(&sd, true).unwrap().pauses.len(), 1); // pressing twice is one break
        assert!(!set_paused(&sd, false).unwrap().paused());
        set_paused(&sd, true).unwrap();
        let s = end_sesi(&sd).unwrap(); // ending during a break closes the break
        assert!(!s.paused() && s.ended.is_some() && s.pauses.len() == 2);
    }

    #[test]
    fn names_become_safe_folder_names() {
        assert_eq!(folder_safe("Siti: A/B*?"), "Siti A B");
        assert_eq!(folder_safe("  ..con.. "), "_con"); // reserved device name in any case
        assert_eq!(folder_safe(" ..Siti.. "), "Siti");
        assert_eq!(folder_safe("CON"), "_CON");
        assert_eq!(folder_safe("<>|"), "tanpa nama");
        assert_eq!(folder_safe(&"a".repeat(90)).len(), 60);
        assert_eq!(clean_person("  Siti \n  Aminah ").unwrap(), "Siti Aminah");
        assert!(clean_person("   ").unwrap_err().starts_with("EMPTY_NAME"));
        assert!(clean_person(&"x".repeat(61)).unwrap_err().starts_with("LONG_NAME"));
        // names with accents and apostrophes are kept as typed
        assert_eq!(clean_person("Ni Luh Putu Ayu D'Silva").unwrap(), "Ni Luh Putu Ayu D'Silva");
    }

    #[test]
    fn ui_paths_cannot_escape_the_data_folder() {
        let d = tmp("resolve");
        assert!(resolve(&d, "2026-11-17 X/Operator A/001 B").is_ok());
        for bad in ["", "..", "../x", "a/../../x", "C:\\Windows", "\\x", "/x"] {
            assert!(resolve(&d, bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_patient_form_is_checked_even_if_the_ui_is_bypassed() {
        assert!(validate_patient(&form("Siti")).is_ok());
        let v = validate_patient(&PatientForm { td: " 120 / 80 ".into(), bb: Some(70.26), ..form("Siti") }).unwrap();
        assert_eq!((v.td.as_str(), v.bb), ("120/80", Some(70.3)));
        for (f, code) in [
            (PatientForm { nama: " ".into(), ..form("") }, "EMPTY_NAME"),
            (PatientForm { usia: None, ..form("A") }, "EMPTY_AGE"),
            (PatientForm { usia: Some(121), ..form("A") }, "BAD_AGE"),
            (PatientForm { kelamin: "f".into(), ..form("A") }, "BAD_SEX"),
            (PatientForm { td: "120".into(), ..form("A") }, "BAD_TD"),
            (PatientForm { td: "80/120".into(), ..form("A") }, "BAD_TD"),
            (PatientForm { td: "abc/def".into(), ..form("A") }, "BAD_TD"),
            (PatientForm { bb: Some(5.0), ..form("A") }, "BAD_BB"),
            (PatientForm { bb: Some(f32::NAN), ..form("A") }, "BAD_BB"),
            (PatientForm { posisi: "tidur".into(), ..form("A") }, "BAD_POSISI"),
        ] {
            assert!(validate_patient(&f).unwrap_err().starts_with(code), "{code}");
        }
        // TD and BB may be left empty
        assert!(validate_patient(&PatientForm { td: "".into(), bb: None, ..form("A") }).is_ok());
    }

    #[test]
    fn left_then_right_accept_retake_and_nothing_lost() {
        let d = tmp("flow");
        let (sd, _) = start_sesi(&d, "X", "Rina", "7MY").unwrap();
        let dir = create_patient(&sd, &form("Siti"), "dev").unwrap();
        assert_eq!(next_side(&read_patient(&dir).unwrap()), Some("left"));
        std::fs::write(pending_file(&dir, "left"), b"a").unwrap();
        reject(&dir, "left").unwrap();
        std::fs::write(pending_file(&dir, "left"), b"b").unwrap();
        accept(&dir, "left", 15.0, 10, 10, false, vec![]).unwrap();
        let p = read_patient(&dir).unwrap();
        assert_eq!((p.scans[0].attempt, p.scans[0].file.as_str()), (2, "Kaki kiri - pindaian asli.png"));
        assert!(dir.join("ditolak/Kaki kiri - percobaan 1.png").exists());
        assert_eq!(next_side(&p), Some("right"));
        std::fs::write(pending_file(&dir, "right"), b"c").unwrap();
        accept(&dir, "right", 15.0, 10, 10, true, vec!["EDGE".into()]).unwrap();
        let p = read_patient(&dir).unwrap();
        assert_eq!(next_side(&p), None);
        assert!(p.scans[1].rotated_180);
        // accepting a new left scan later keeps the old accepted one under ditolak\
        std::fs::write(pending_file(&dir, "left"), b"d").unwrap();
        accept(&dir, "left", 15.0, 10, 10, false, vec![]).unwrap();
        assert_eq!(std::fs::read(dir.join("Kaki kiri - pindaian asli.png")).unwrap(), b"d");
        assert_eq!(std::fs::read(dir.join("ditolak/Kaki kiri - percobaan 2.png")).unwrap(), b"b");
        assert_eq!(read_patient(&dir).unwrap().scans.len(), 2);
    }

    #[test]
    fn a_failed_accept_puts_the_scan_back_so_nothing_is_lost() {
        let d = tmp("rollback");
        let (sd, _) = start_sesi(&d, "X", "Rina", "7MY").unwrap();
        let dir = create_patient(&sd, &form("Siti"), "dev").unwrap();
        std::fs::write(pending_file(&dir, "left"), b"precious").unwrap();
        // make writing the patient file impossible: its temp file name is taken by a folder
        std::fs::create_dir(dir.join("data pasien.json.tmp")).unwrap();
        assert!(accept(&dir, "left", 12.0, 10, 10, false, vec![]).is_err());
        assert_eq!(std::fs::read(pending_file(&dir, "left")).unwrap(), b"precious");
        assert!(!dir.join(raw_scan_name("left")).exists());
        std::fs::remove_dir(dir.join("data pasien.json.tmp")).unwrap();
        accept(&dir, "left", 12.0, 10, 10, false, vec![]).unwrap();
        assert_eq!(read_patient(&dir).unwrap().scans.len(), 1);
    }

    #[test]
    fn resume_cancel_and_damaged_files() {
        let d = tmp("resume");
        let (sd, _) = start_sesi(&d, "X", "Rina", "7MY").unwrap();
        assert!(find_unfinished(&sd).is_none());
        let a = create_patient(&sd, &form("A"), "dev").unwrap();
        complete(&a).unwrap();
        let b = create_patient(&sd, &form("B"), "dev").unwrap();
        assert_eq!(find_unfinished(&sd).unwrap().1.nama, "B");
        // empty patient: cancel removes the folder (and its now-empty operator folder stays because A is there)
        assert!(cancel(&b).unwrap() && !b.exists());
        assert!(find_unfinished(&sd).is_none());
        // the number is not reused
        let c = create_patient(&sd, &form("C"), "dev").unwrap();
        assert_eq!(read_patient(&c).unwrap().no, 3);
        std::fs::write(pending_file(&c, "left"), b"a").unwrap();
        assert!(!cancel(&c).unwrap() && read_patient(&c).unwrap().cancelled);
        // a damaged patient file is reported, not hidden, and does not break the CSV
        std::fs::write(a.join(PATIENT_FILE), b"{ truncated").unwrap();
        assert_eq!(damaged_patients(&d).len(), 1);
        rebuild_csv(&d).unwrap();
        // an operator folder left empty by a cancel is removed
        change_operator(&sd, "Budi").unwrap();
        let e = create_patient(&sd, &form("E"), "dev").unwrap();
        assert!(cancel(&e).unwrap());
        assert!(!sd.join("Operator Budi").exists());
    }

    #[test]
    fn notes_inlow_and_marks_are_stored_and_checked() {
        let d = tmp("extras");
        let (sd, _) = start_sesi(&d, "X", "Rina", "7MY").unwrap();
        let dir = create_patient(&sd, &form("A"), "dev").unwrap();
        let p = set_notes(&dir, &Notes { kiri: "jari ditekuk\u{7}".into(), kanan: "".into(), umum: " pasien gelisah\nbaris 2 ".into() }).unwrap();
        assert_eq!((p.notes.kiri.as_str(), p.notes.umum.as_str()), ("jari ditekuk", "pasien gelisah\nbaris 2"));
        let mut inl = Inlow::default();
        inl.kiri.items.insert("kuku".into(), 2);
        assert_eq!(set_inlow(&dir, inl.clone()).unwrap().inlow.kiri.items["kuku"], 2);
        inl.kiri.items.insert("kuku".into(), 9);
        assert!(set_inlow(&dir, inl).is_err());
        let m = Mark { side: "left".into(), x: 10.0, y: 20.0, r: 5.0, label: "kalus?".into() };
        assert_eq!(set_marks(&dir, vec![m.clone()]).unwrap().marks.len(), 1);
        assert!(set_marks(&dir, vec![Mark { x: f32::NAN, ..m.clone() }]).is_err());
        assert!(set_marks(&dir, vec![Mark { side: "middle".into(), ..m }]).is_err());
    }

    #[test]
    fn settings_file_with_bom_garbage_or_old_fields_never_loses_the_original() {
        let d = tmp("settings");
        let f = d.join("settings.json");
        assert!(load_settings(&f).operators.is_empty());
        // a file from v0.1 (has "rounds") still loads
        let old = r#"{"data_dir":"E:\\hasil smartfoot","rounds":1,"scanner":null,"station":"7MY","backup_dir":"C:\\SmartFoot_Backup","setup_done":true}"#;
        std::fs::write(&f, format!("\u{feff}{old}")).unwrap();
        let s = load_settings(&f);
        assert_eq!(s.station, "7MY");
        assert!(s.setup_done && s.operators.is_empty());
        std::fs::write(&f, "{ this is not json").unwrap();
        assert_eq!(load_settings(&f).station, "");
        assert_eq!(std::fs::read_to_string(d.join("settings.json.bad")).unwrap(), "{ this is not json");
    }

    #[test]
    fn settings_from_the_ui_are_checked_and_operators_remembered() {
        let ok = Settings { data_dir: "E:\\Data SmartFoot".into(), scanner: None, station: " 7my ".into(), backup_dir: Some("  ".into()), setup_done: true, operators: vec![] };
        let v = validate_settings(ok.clone()).unwrap();
        assert_eq!(v.station, "7MY");
        assert!(v.backup_dir.is_none());
        for (bad, code) in [
            (Settings { data_dir: "".into(), ..ok.clone() }, "BAD_DIR"),
            (Settings { data_dir: "data".into(), ..ok.clone() }, "BAD_DIR"),
            (Settings { backup_dir: Some("backup".into()), ..ok.clone() }, "BAD_DIR"),
            (Settings { station: "!!".into(), ..ok.clone() }, "BAD_STATION"),
        ] {
            assert!(validate_settings(bad).unwrap_err().starts_with(code), "{code}");
        }
        let mut s = ok;
        remember_operator(&mut s, "Rina");
        remember_operator(&mut s, "Budi");
        remember_operator(&mut s, "rina");
        assert_eq!(s.operators, vec!["rina", "Budi"]);
    }

    #[test]
    fn csv_cells_cannot_run_as_formulas() {
        assert_eq!(csv_field("=1+1"), "'=1+1");
        assert_eq!(csv_field("@SUM(A1)"), "'@SUM(A1)");
        assert_eq!(csv_field("-5"), "-5");
        assert_eq!(csv_field("a,\"b\""), "\"a,\"\"b\"\"\"");
    }
}
