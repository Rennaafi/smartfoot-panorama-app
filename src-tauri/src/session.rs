//! Settings and per-session storage.
//!
//! Layout on disk:  <data_dir>\<PID>\session.json
//!                                  \scans\right_1.png ...   accepted scans
//!                                  \_pending\               scan waiting for Accept / Retake
//!                                  \rejected\               scans the operator retook (kept, never silently deleted)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub data_dir: String,
    pub rounds: u8,
    /// Exact WIA device name, or None = pick the LiDE 300 driverless route automatically.
    pub scanner: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = if Path::new("D:\\").exists() {
            "D:\\SmartFoot_Data".to_string()
        } else {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into());
            format!("{home}\\Documents\\SmartFoot_Data")
        };
        Settings { data_dir, rounds: 3, scanner: None }
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

/// Next free `SFP-0001`-style ID in the data folder.
pub fn next_pid(data_dir: &Path) -> String {
    let mut max = 0u32;
    if let Ok(rd) = std::fs::read_dir(data_dir) {
        for e in rd.flatten() {
            if let Some(n) = e.file_name().to_string_lossy().strip_prefix("SFP-").and_then(|r| r.parse::<u32>().ok()) {
                max = max.max(n);
            }
        }
    }
    format!("SFP-{:04}", max + 1)
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
    });
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
        assert_eq!(next_pid(&d), "SFP-0001");
        create_session(&d, "SFP-0001", "RF", Some(40), "f", "", 3, "x").unwrap();
        assert_eq!(next_pid(&d), "SFP-0002");
        assert!(create_session(&d, "SFP-0001", "RF", None, "m", "", 3, "x").unwrap_err().starts_with("ID_USED"));
        assert_eq!(clean_pid(" SFP 0003/../ ").unwrap(), "SFP0003");
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
