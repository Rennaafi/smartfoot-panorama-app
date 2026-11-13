//! Settings and per-session storage.
//!
//! Layout on disk:  <data_dir>\<PID>\session.json
//!                                  \scans\right_1.png ...   accepted scans
//!                                  \_pending\               scan waiting for Accept / Retake
//!                                  \rejected\               scans the operator retook (kept, never silently deleted)

use crate::measure::Measurements;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub data_dir: String,
    pub rounds: u8,
    /// Exact WIA device name, or None = pick the LiDE 300 driverless route automatically.
    pub scanner: Option<String>,
    /// Short code that makes patient IDs unique per laptop: SFP-<station>-0001.
    #[serde(default)]
    pub station: String,
    /// Folder for the second copy of all data (None = no second copy).
    #[serde(default)]
    pub backup_dir: Option<String>,
    /// The first-run screen ("where is the data saved?") has been completed on this laptop.
    #[serde(default)]
    pub setup_done: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = if Path::new("D:\\").exists() {
            "D:\\SmartFoot_Data".to_string()
        } else {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into());
            format!("{home}\\Documents\\SmartFoot_Data")
        };
        Settings { data_dir, rounds: 3, scanner: None, station: String::new(), backup_dir: None, setup_done: false }
    }
}

/// Read the settings. A missing file gives defaults. A file that exists but cannot be understood
/// (hand-edited, damaged, saved with a byte-order mark) is first copied to `settings.json.bad` so
/// the original values are never lost when defaults replace it; a BOM itself is tolerated.
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
    if !(new.rounds == 1 || new.rounds == 3) {
        return Err("BAD_ROUNDS: rounds must be 1 or 3".into());
    }
    new.data_dir = new.data_dir.trim().to_string();
    if new.data_dir.is_empty() || !Path::new(&new.data_dir).is_absolute() {
        return Err("BAD_DIR: the data folder must be a full path such as D:\\SmartFoot_Data".into());
    }
    new.backup_dir = new.backup_dir.take().map(|b| b.trim().to_string()).filter(|b| !b.is_empty());
    if let Some(b) = &new.backup_dir {
        if !Path::new(b).is_absolute() {
            return Err("BAD_DIR: the backup folder must be a full path such as E:\\SmartFoot_Backup".into());
        }
    }
    Ok(new)
}

const RESERVED: [&str; 22] = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];

/// Check the new-session form and return the tidy (pid, operator, note). Everything that ends up in
/// a folder name or a CSV cell is restricted here, whatever the UI did or did not check.
pub fn validate_start(pid: &str, operator: &str, age: Option<u32>, sex: &str, note: &str) -> Result<(String, String, String), String> {
    let pid = clean_pid(pid)?;
    if pid.len() > 32 || !pid.chars().next().map(|c| c.is_ascii_alphanumeric()).unwrap_or(false) || RESERVED.contains(&pid.to_ascii_uppercase().as_str()) {
        return Err("BAD_ID: the patient ID must start with a letter or digit, be at most 32 characters and not be a reserved Windows name".into());
    }
    let op = operator.trim().to_string();
    if op.is_empty() {
        return Err("EMPTY_OP: the operator initials are empty".into());
    }
    // the operator lands in measurements.csv: a leading = + - @ would be run as a formula by Excel
    if op.chars().count() > 8 || !op.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) || op.chars().any(|c| c.is_control()) {
        return Err("BAD_OP: the operator initials must start with a letter or digit and be at most 8 characters".into());
    }
    if age.map(|a| a > 120).unwrap_or(false) {
        return Err("BAD_AGE: the age must be between 0 and 120".into());
    }
    if sex != "f" && sex != "m" {
        return Err("BAD_SEX: sex must be f or m".into());
    }
    let note: String = note.chars().map(|c| if c.is_control() { ' ' } else { c }).take(500).collect();
    Ok((pid, op, note.trim().to_string()))
}

/// Session folders that cannot be read (damaged session.json) or hold scans but have no
/// session.json at all. Such data is never deleted; the app only warns that it exists.
pub fn damaged_sessions(data_dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(data_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('_'))
        .filter(|e| {
            let p = e.path();
            if p.join("session.json").exists() {
                read_session(&p).is_err()
            } else {
                std::fs::read_dir(p.join("scans")).map(|mut r| r.next().is_some()).unwrap_or(false)
            }
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    v.sort();
    v
}

pub fn save_settings(file: &Path, s: &Settings) -> Result<(), String> {
    if let Some(d) = file.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    std::fs::write(file, serde_json::to_string_pretty(s).unwrap()).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanEntry {
    pub file: String,
    pub side: String,
    pub round: u8,
    pub attempt: u32,
    pub seconds: f64,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    pub accepted_at: String,
    /// Plain-language warnings the operator accepted anyway (empty = clean).
    pub warnings: Vec<String>,
    /// Filled in right after Accept; None if no foot could be measured.
    #[serde(default)]
    pub measurements: Option<Measurements>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionFile {
    pub pid: String,
    pub created: String,
    pub operator: String,
    pub age: Option<u32>,
    pub sex: String,
    pub note: String,
    pub rounds: u8,
    pub scanner: String,
    pub app_version: String,
    /// The ruler calibration in force when the session was created (mm per pixel, both axes).
    /// Sessions from before the calibration have null here.
    pub calibration: Option<serde_json::Value>,
    pub scans: Vec<ScanEntry>,
    pub completed: bool,
    /// Set when the operator cancelled a session that already had files worth keeping.
    #[serde(default)]
    pub cancelled: bool,
}

pub fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// Keep only characters that are safe in a folder name.
pub fn clean_pid(raw: &str) -> Result<String, String> {
    let p: String = raw.trim().chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    if p.is_empty() { Err("EMPTY_ID: patient ID is empty".into()) } else { Ok(p) }
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

/// Next free ID in the data folder: `SFP-<station>-0001`, or `SFP-0001` when no station code is set.
pub fn next_pid(data_dir: &Path, station: &str) -> String {
    let prefix = if station.is_empty() { "SFP-".to_string() } else { format!("SFP-{station}-") };
    let mut max = 0u32;
    if let Ok(rd) = std::fs::read_dir(data_dir) {
        for e in rd.flatten() {
            if let Some(n) = e.file_name().to_string_lossy().strip_prefix(prefix.as_str()).and_then(|r| r.parse::<u32>().ok()) {
                max = max.max(n);
            }
        }
    }
    format!("{prefix}{:04}", max + 1)
}

pub fn session_dir(data_dir: &Path, pid: &str) -> Result<PathBuf, String> {
    Ok(data_dir.join(clean_pid(pid)?))
}

pub fn read_session(dir: &Path) -> Result<SessionFile, String> {
    let s = std::fs::read_to_string(dir.join("session.json")).map_err(|e| format!("cannot read session.json: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("session.json is damaged: {e}"))
}

/// Write via a temp file so a crash never leaves a half-written session.json.
pub fn write_session(dir: &Path, s: &SessionFile) -> Result<(), String> {
    let tmp = dir.join("session.json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(s).unwrap()).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, dir.join("session.json")).map_err(|e| e.to_string())
}

pub fn create_session(
    data_dir: &Path,
    pid: &str,
    operator: &str,
    age: Option<u32>,
    sex: &str,
    note: &str,
    rounds: u8,
    scanner: &str,
) -> Result<PathBuf, String> {
    let dir = session_dir(data_dir, pid)?;
    if dir.exists() {
        return Err(format!("ID_USED: {} already exists in the data folder", clean_pid(pid)?));
    }
    for sub in ["scans", "_pending", "rejected"] {
        std::fs::create_dir_all(dir.join(sub)).map_err(|e| format!("cannot create folder: {e}"))?;
    }
    let s = SessionFile {
        pid: clean_pid(pid)?,
        created: now_iso(),
        operator: operator.trim().to_string(),
        age,
        sex: sex.to_string(),
        note: note.trim().to_string(),
        rounds,
        scanner: scanner.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        calibration: Some(crate::measure::calibration_record()),
        scans: vec![],
        completed: false,
        cancelled: false,
    };
    write_session(&dir, &s)?;
    Ok(dir)
}

/// Move a pending scan into scans/ and record it.
pub fn accept(dir: &Path, side: &str, round: u8, seconds: f64, w: u32, h: u32, warnings: Vec<String>) -> Result<(), String> {
    let name = format!("{side}_{round}.png");
    let from = dir.join("_pending").join(&name);
    let to = dir.join("scans").join(&name);
    std::fs::rename(&from, &to).map_err(|e| format!("cannot save scan: {e}"))?;
    // if recording it fails (disk full, session.json locked...), put the scan back so the operator
    // can press Accept again; a scan must never sit in scans/ without an entry in session.json
    record_accepted(dir, side, round, &name, seconds, w, h, warnings).map_err(|e| {
        let _ = std::fs::rename(&to, &from);
        e
    })
}

fn record_accepted(dir: &Path, side: &str, round: u8, name: &str, seconds: f64, w: u32, h: u32, warnings: Vec<String>) -> Result<(), String> {
    let mut s = read_session(dir)?;
    let attempt = 1 + std::fs::read_dir(dir.join("rejected"))
        .map(|rd| rd.flatten().filter(|e| e.file_name().to_string_lossy().starts_with(&format!("{side}_{round}_"))).count() as u32)
        .unwrap_or(0);
    s.scans.retain(|e| !(e.side == side && e.round == round));
    s.scans.push(ScanEntry {
        file: format!("scans/{name}"),
        side: side.to_string(),
        round,
        attempt,
        seconds,
        width_px: w,
        height_px: h,
        dpi: 300,
        accepted_at: now_iso(),
        warnings,
        measurements: None,
    });
    write_session(dir, &s)
}

/// Store (or replace) the measurements of one accepted scan.
pub fn set_measurements(dir: &Path, side: &str, round: u8, m: Option<Measurements>) -> Result<(), String> {
    let mut s = read_session(dir)?;
    let e = s.scans.iter_mut().find(|e| e.side == side && e.round == round).ok_or("scan not found in session")?;
    e.measurements = m;
    write_session(dir, &s)
}

/// Keep a retaken scan under rejected/ with a running number.
pub fn reject(dir: &Path, side: &str, round: u8) -> Result<(), String> {
    let from = dir.join("_pending").join(format!("{side}_{round}.png"));
    if !from.exists() {
        return Ok(());
    }
    let n = 1 + std::fs::read_dir(dir.join("rejected"))
        .map(|rd| rd.flatten().filter(|e| e.file_name().to_string_lossy().starts_with(&format!("{side}_{round}_"))).count())
        .unwrap_or(0);
    std::fs::rename(&from, dir.join("rejected").join(format!("{side}_{round}_{n}.png"))).map_err(|e| e.to_string())
}

/// Cancel a session. A session with no scans at all is removed (we created that empty folder);
/// anything with scans is kept and marked cancelled. Returns true if the folder was removed.
pub fn cancel(dir: &Path) -> Result<bool, String> {
    let mut s = read_session(dir)?;
    let has_files = |sub: &str| std::fs::read_dir(dir.join(sub)).map(|mut r| r.next().is_some()).unwrap_or(false);
    if s.scans.is_empty() && !has_files("scans") && !has_files("rejected") && !has_files("_pending") {
        std::fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
        return Ok(true);
    }
    s.cancelled = true;
    write_session(dir, &s)?;
    Ok(false)
}

pub fn complete(dir: &Path) -> Result<(), String> {
    let mut s = read_session(dir)?;
    s.completed = true;
    write_session(dir, &s)
}

/// The order scans are taken in: right foot then left foot, round by round.
pub fn seq(rounds: u8) -> Vec<(&'static str, u8)> {
    let mut v = Vec::new();
    for r in 1..=rounds {
        v.push(("right", r));
        v.push(("left", r));
    }
    v
}

/// Index into `seq` of the first scan that is not accepted yet (scans are taken in order).
pub fn next_index(s: &SessionFile) -> usize {
    let q = seq(s.rounds);
    q.iter().position(|(side, r)| !s.scans.iter().any(|e| e.side == *side && e.round == *r)).unwrap_or(q.len())
}

/// The newest session that was started but neither finished nor cancelled (e.g. the app closed or
/// crashed in the middle of a patient).
pub fn find_unfinished(data_dir: &Path) -> Option<SessionFile> {
    let mut v: Vec<SessionFile> = std::fs::read_dir(data_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| read_session(&e.path()).ok())
        .filter(|s| !s.completed && !s.cancelled)
        .collect();
    v.sort_by(|a, b| b.created.cmp(&a.created));
    v.into_iter().next()
}

#[derive(Serialize, Clone, Debug)]
pub struct SessionSummary {
    pub pid: String,
    pub created: String,
    pub operator: String,
    pub scans: usize,
    pub completed: bool,
    pub cancelled: bool,
}

/// All sessions in the data folder, newest first.
pub fn list_sessions(data_dir: &Path) -> Vec<SessionSummary> {
    let mut v: Vec<SessionSummary> = std::fs::read_dir(data_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| read_session(&e.path()).ok())
        .map(|s| SessionSummary { pid: s.pid, created: s.created, operator: s.operator, scans: s.scans.len(), completed: s.completed, cancelled: s.cancelled })
        .collect();
    v.sort_by(|a, b| b.created.cmp(&a.created));
    v
}

fn csv_field(s: &str) -> String {
    let s = &if s.starts_with(['=', '+', '-', '@']) && s.parse::<f64>().is_err() { format!("'{s}") } else { s.to_string() };
    if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

/// Rebuild `measurements.csv` (one row per accepted scan) from every session.json, so the file can
/// never disagree with the sessions, even after a hand correction.
pub fn rebuild_csv(data_dir: &Path) -> Result<PathBuf, String> {
    let mut sessions: Vec<SessionFile> = std::fs::read_dir(data_dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .filter_map(|e| read_session(&e.path()).ok())
        .collect();
    sessions.sort_by(|a, b| a.pid.cmp(&b.pid));
    let mut out = String::from("pid,created,operator,age,sex,rounds,side,round,attempt,scan_file,calibrated,mm_per_px_x,mm_per_px_y,fl_mm,fw_mm,area_cm2,toeless_area_cm2,hind_cm2,mid_cm2,fore_cm2,ai,csi_pct,toe_base_method,toeless_frac,points_corrected,warnings\n");
    for s in &sessions {
        let mut scans = s.scans.clone();
        scans.sort_by(|a, b| (a.round, &a.side).cmp(&(b.round, &b.side)));
        for e in &scans {
            let mut f: Vec<String> = vec![s.pid.clone(), s.created.clone(), s.operator.clone(), s.age.map(|a| a.to_string()).unwrap_or_default(), s.sex.clone(), s.rounds.to_string(), e.side.clone(), e.round.to_string(), e.attempt.to_string(), e.file.clone()];
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
                    m.toe_base_method.clone(),
                    format!("{:.3}", m.toeless_frac),
                    m.corrected.to_string(),
                ]),
                None => f.extend(std::iter::repeat(String::new()).take(15)),
            }
            f.push(e.warnings.join(";"));
            out.push_str(&f.iter().map(|x| csv_field(x)).collect::<Vec<_>>().join(","));
            out.push('\n');
        }
    }
    let path = data_dir.join("measurements.csv");
    std::fs::write(&path, out).map_err(|e| format!("cannot write measurements.csv: {e}"))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sf_test_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn ids_count_up_and_reject_duplicates() {
        let d = tmp("ids");
        assert_eq!(next_pid(&d, ""), "SFP-0001");
        create_session(&d, "SFP-0001", "RF", Some(40), "f", "", 3, "x").unwrap();
        assert_eq!(next_pid(&d, ""), "SFP-0002");
        // another laptop's folders never count toward this laptop's numbers
        create_session(&d, "SFP-K3F-0007", "RF", None, "f", "", 3, "x").unwrap();
        assert_eq!(next_pid(&d, "K3F"), "SFP-K3F-0008");
        assert_eq!(next_pid(&d, "Q9"), "SFP-Q9-0001");
        assert_eq!(next_pid(&d, ""), "SFP-0002");
        assert_eq!(clean_station(" k3-f!x9z "), "K3FX");
        let g = generate_station();
        assert!(g.len() == 3 && g == clean_station(&g));
        assert!(create_session(&d, "SFP-0001", "RF", None, "m", "", 3, "x").unwrap_err().starts_with("ID_USED"));
        assert_eq!(clean_pid(" SFP 0003/../ ").unwrap(), "SFP0003");
    }

    #[test]
    fn csv_has_one_row_per_scan_and_survives_missing_measurements() {
        let d = tmp("csv");
        let dir = create_session(&d, "SFP-0001", "R,F", Some(30), "m", "", 1, "x").unwrap();
        std::fs::write(dir.join("_pending/right_1.png"), b"a").unwrap();
        accept(&dir, "right", 1, 15.0, 10, 10, vec!["EDGE".into()]).unwrap();
        let p = rebuild_csv(&d).unwrap();
        let text = std::fs::read_to_string(p).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert!(text.lines().nth(1).unwrap().contains("\"R,F\"") && text.contains("EDGE"));
        assert_eq!(list_sessions(&d).len(), 1);
    }

    #[test]
    fn resume_finds_the_newest_unfinished_session_and_the_next_scan() {
        let d = tmp("resume");
        assert!(find_unfinished(&d).is_none());
        let old = create_session(&d, "SFP-0001", "RF", None, "f", "", 3, "x").unwrap();
        complete(&old).unwrap();
        assert!(find_unfinished(&d).is_none());
        std::thread::sleep(std::time::Duration::from_millis(1100)); // created is stored to the second
        let dir = create_session(&d, "SFP-0002", "RF", None, "f", "", 3, "x").unwrap();
        assert_eq!(find_unfinished(&d).unwrap().pid, "SFP-0002");
        assert_eq!(next_index(&read_session(&dir).unwrap()), 0);
        for (side, r) in [("right", 1u8), ("left", 1)] {
            std::fs::write(dir.join("_pending").join(format!("{side}_{r}.png")), b"a").unwrap();
            accept(&dir, side, r, 12.0, 10, 10, vec![]).unwrap();
        }
        let s = read_session(&dir).unwrap();
        assert_eq!(next_index(&s), 2); // next = right, round 2
        assert_eq!(seq(3)[next_index(&s)], ("right", 2));
        assert_eq!(seq(1).len(), 2);
        cancel(&dir).unwrap();
        assert!(find_unfinished(&d).is_none());
    }

    #[test]
    fn settings_file_with_bom_or_garbage_never_loses_the_original() {
        let d = tmp("settings");
        let f = d.join("settings.json");
        assert_eq!(load_settings(&f).rounds, 3); // missing file: defaults, nothing written
        assert!(!d.join("settings.json.bad").exists());
        let good = r#"{"data_dir":"D:\\X","rounds":1,"scanner":null,"station":"7MY"}"#;
        std::fs::write(&f, format!("\u{feff}{good}")).unwrap(); // saved by a tool that adds a BOM
        let s = load_settings(&f);
        assert_eq!((s.station.as_str(), s.rounds), ("7MY", 1));
        assert!(s.backup_dir.is_none()); // older file without the backup field still loads
        std::fs::write(&f, "{ this is not json").unwrap();
        assert_eq!(load_settings(&f).rounds, 3);
        assert_eq!(std::fs::read_to_string(d.join("settings.json.bad")).unwrap(), "{ this is not json");
    }

    #[test]
    fn settings_from_the_ui_are_checked() {
        let ok = Settings { data_dir: "D:\\SmartFoot_Data".into(), rounds: 3, scanner: None, station: " 7my ".into(), backup_dir: Some("  ".into()), setup_done: true };
        let v = validate_settings(ok.clone()).unwrap();
        assert_eq!(v.station, "7MY");
        assert!(v.backup_dir.is_none()); // blank = off
        for (bad, code) in [
            (Settings { data_dir: "".into(), ..ok.clone() }, "BAD_DIR"),
            (Settings { data_dir: "   ".into(), ..ok.clone() }, "BAD_DIR"),
            (Settings { data_dir: "data".into(), ..ok.clone() }, "BAD_DIR"), // relative: would land next to the exe
            (Settings { backup_dir: Some("backup".into()), ..ok.clone() }, "BAD_DIR"),
            (Settings { rounds: 2, ..ok.clone() }, "BAD_ROUNDS"),
            (Settings { rounds: 0, ..ok.clone() }, "BAD_ROUNDS"),
            (Settings { station: "!!".into(), ..ok.clone() }, "BAD_STATION"),
            (Settings { station: "".into(), ..ok.clone() }, "BAD_STATION"),
        ] {
            assert!(validate_settings(bad).unwrap_err().starts_with(code), "{code}");
        }
    }

    #[test]
    fn the_new_session_form_is_checked_even_if_the_ui_is_bypassed() {
        let ok = |pid: &str, op: &str, age: Option<u32>, sex: &str| validate_start(pid, op, age, sex, "");
        assert_eq!(ok("SFP-7MY-0001", " RF ", Some(40), "f").unwrap().1, "RF");
        for (r, code) in [
            (ok("SFP-1", "", None, "f"), "EMPTY_OP"),
            (ok("SFP-1", "   ", None, "f"), "EMPTY_OP"),
            (ok("SFP-1", "=1+1", None, "f"), "BAD_OP"), // spreadsheet formula
            (ok("SFP-1", "-cmd", None, "f"), "BAD_OP"),
            (ok("SFP-1", "ABCDEFGHIJ", None, "f"), "BAD_OP"),
            (ok("SFP-1", "R\nF", None, "f"), "BAD_OP"),
            (ok("SFP-1", "RF", Some(121), "f"), "BAD_AGE"),
            (ok("SFP-1", "RF", Some(u32::MAX), "f"), "BAD_AGE"),
            (ok("SFP-1", "RF", None, "x"), "BAD_SEX"),
            (ok("SFP-1", "RF", None, ""), "BAD_SEX"),
            (ok("", "RF", None, "f"), "EMPTY_ID"),
            (ok("../..", "RF", None, "f"), "EMPTY_ID"), // everything stripped
            (ok("CON", "RF", None, "f"), "BAD_ID"),
            (ok("nul", "RF", None, "f"), "BAD_ID"),
            (ok("com1", "RF", None, "f"), "BAD_ID"),
            (ok("-1+2", "RF", None, "f"), "BAD_ID"),
            (ok(&"A".repeat(33), "RF", None, "f"), "BAD_ID"),
        ] {
            assert!(r.unwrap_err().starts_with(code), "{code}");
        }
        // path tricks are reduced to a safe name, never a path
        assert_eq!(ok("..\\..\\Windows", "RF", None, "f").unwrap().0, "Windows");
        assert_eq!(validate_start("A1", "RF", Some(0), "m", "line1\nline2\t\u{7}x").unwrap().2, "line1 line2  x");
        assert_eq!(validate_start("A1", "RF", None, "m", &"n".repeat(900)).unwrap().2.len(), 500);
    }

    #[test]
    fn csv_cells_cannot_run_as_formulas() {
        assert_eq!(csv_field("=1+1"), "'=1+1");
        assert_eq!(csv_field("@SUM(A1)"), "'@SUM(A1)");
        assert_eq!(csv_field("-5"), "-5"); // a plain negative number stays a number
        assert_eq!(csv_field("a,\"b\""), "\"a,\"\"b\"\"\"");
        assert_eq!(csv_field("line\nbreak"), "\"line\nbreak\"");
        assert_eq!(csv_field("Ünïcode ✓"), "Ünïcode ✓");
    }

    #[test]
    fn a_failed_accept_puts_the_scan_back_so_nothing_is_lost() {
        let d = tmp("rollback");
        let dir = create_session(&d, "SFP-0001", "RF", None, "f", "", 1, "x").unwrap();
        std::fs::write(dir.join("_pending/right_1.png"), b"precious").unwrap();
        // make writing session.json impossible: its temp file name is taken by a folder
        std::fs::create_dir(dir.join("session.json.tmp")).unwrap();
        assert!(accept(&dir, "right", 1, 12.0, 10, 10, vec![]).is_err());
        assert_eq!(std::fs::read(dir.join("_pending/right_1.png")).unwrap(), b"precious", "scan must be back in _pending");
        assert!(!dir.join("scans/right_1.png").exists(), "no orphan scan without a session entry");
        assert!(read_session(&dir).unwrap().scans.is_empty());
        // once the obstacle is gone the same Accept works
        std::fs::remove_dir(dir.join("session.json.tmp")).unwrap();
        accept(&dir, "right", 1, 12.0, 10, 10, vec![]).unwrap();
        assert_eq!(read_session(&dir).unwrap().scans.len(), 1);
    }

    #[test]
    fn damaged_session_folders_are_reported_not_hidden() {
        let d = tmp("damaged");
        create_session(&d, "SFP-0001", "RF", None, "f", "", 3, "x").unwrap();
        assert!(damaged_sessions(&d).is_empty());
        std::fs::create_dir_all(d.join("SFP-0002")).unwrap(); // empty folder: nothing to lose
        std::fs::create_dir_all(d.join("_scratch/scans")).unwrap(); // not a session
        std::fs::write(d.join("_scratch/scans/a.png"), b"x").unwrap();
        assert!(damaged_sessions(&d).is_empty());
        let bad = create_session(&d, "SFP-0003", "RF", None, "f", "", 3, "x").unwrap();
        std::fs::write(bad.join("session.json"), b"{ truncated").unwrap();
        std::fs::create_dir_all(d.join("SFP-0004/scans")).unwrap(); // scans but no session.json
        std::fs::write(d.join("SFP-0004/scans/right_1.png"), b"x").unwrap();
        assert_eq!(damaged_sessions(&d), vec!["SFP-0003".to_string(), "SFP-0004".to_string()]);
        // a damaged folder still counts for numbering, so its ID is never reused
        assert_eq!(next_pid(&d, ""), "SFP-0005");
        // and it does not break listing or the CSV for the healthy sessions
        assert_eq!(list_sessions(&d).len(), 1);
        rebuild_csv(&d).unwrap();
        assert!(find_unfinished(&d).is_some());
    }

    #[test]
    fn cancel_removes_empty_keeps_scanned() {
        let d = tmp("cancel");
        let a = create_session(&d, "SFP-0001", "RF", None, "f", "", 3, "x").unwrap();
        assert!(cancel(&a).unwrap() && !a.exists());
        let b = create_session(&d, "SFP-0002", "RF", None, "f", "", 3, "x").unwrap();
        std::fs::write(b.join("_pending/right_1.png"), b"a").unwrap();
        assert!(!cancel(&b).unwrap() && read_session(&b).unwrap().cancelled);
    }

    #[test]
    fn accept_and_retake_keep_every_file() {
        let d = tmp("flow");
        let dir = create_session(&d, "SFP-0001", "RF", Some(40), "f", "", 1, "x").unwrap();
        std::fs::write(dir.join("_pending/right_1.png"), b"a").unwrap();
        reject(&dir, "right", 1).unwrap();
        std::fs::write(dir.join("_pending/right_1.png"), b"b").unwrap();
        accept(&dir, "right", 1, 15.0, 10, 10, vec![]).unwrap();
        let s = read_session(&dir).unwrap();
        assert_eq!(s.scans.len(), 1);
        assert_eq!(s.scans[0].attempt, 2);
        assert!(dir.join("rejected/right_1_1.png").exists() && dir.join("scans/right_1.png").exists());
    }
}
