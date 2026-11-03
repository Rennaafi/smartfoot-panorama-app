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
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = if Path::new("D:\\").exists() {
            "D:\\SmartFoot_Data".to_string()
        } else {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into());
            format!("{home}\\Documents\\SmartFoot_Data")
        };
        Settings { data_dir, rounds: 3, scanner: None, station: String::new(), backup_dir: None }
    }
}

pub fn load_settings(file: &Path) -> Settings {
    std::fs::read_to_string(file).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
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
    /// Filled in once the scale calibration tool exists (mm per pixel, both axes).
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
        calibration: None,
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
