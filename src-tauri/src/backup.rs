//! Second copy of the data. After every accepted scan (and when a session is corrected, finished
//! or cancelled) the session folder is mirrored to a second folder, e.g. on a USB drive or in a
//! synced cloud folder, so one failing disk never loses patient scans.
//!
//! The mirror only ever copies *into* the backup folder; it never deletes or changes anything in
//! the main data folder. A copy that fails does not stop the nurse: the failure is remembered and
//! shown in the app instead.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// What the app shows about the second copy.
#[derive(Serialize, Clone, Debug, Default)]
pub struct BackupStatus {
    /// A backup folder is set in team Settings.
    pub enabled: bool,
    pub dir: String,
    /// The folder exists right now (false = USB drive unplugged, network share gone, ...).
    pub reachable: bool,
    /// Local time of the last copy that finished without error.
    pub last_ok: Option<String>,
    /// Files copied by that last successful run.
    pub last_files: usize,
    /// Why the most recent copy failed (cleared by the next success).
    pub last_err: Option<String>,
}

static LAST: Mutex<(Option<String>, usize, Option<String>)> = Mutex::new((None, 0, None));

fn record(result: &Result<usize, String>) {
    if let Ok(mut g) = LAST.lock() {
        match result {
            Ok(n) => *g = (Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()), *n, None),
            Err(e) => g.2 = Some(e.clone()),
        }
    }
}

pub fn status(dir: Option<&str>) -> BackupStatus {
    let d = dir.map(str::trim).filter(|d| !d.is_empty());
    let (last_ok, last_files, last_err) = LAST.lock().map(|g| g.clone()).unwrap_or_default();
    BackupStatus {
        enabled: d.is_some(),
        dir: d.unwrap_or("").to_string(),
        reachable: d.map(|d| Path::new(d).is_dir()).unwrap_or(false),
        last_ok,
        last_files,
        last_err,
    }
}

/// The backup folder must be a different place from the data folder (not the same, not inside it,
/// not around it), otherwise it would be a copy on the same disk copying onto itself.
pub fn check_dir(data_dir: &Path, backup: &Path) -> Result<(), String> {
    let norm = |p: &Path| p.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase();
    let (a, b) = (norm(data_dir), norm(backup));
    if a == b || b.starts_with(&format!("{a}\\")) || a.starts_with(&format!("{b}\\")) {
        return Err("BACKUP_SAME: the backup folder must be a different folder from the data folder".into());
    }
    std::fs::create_dir_all(backup).map_err(|e| format!("BACKUP_FAILED: cannot use backup folder: {e}"))
}

/// Copy a file if the target is missing or differs in size or modified time. Returns whether it copied.
fn copy_if_changed(from: &Path, to: &Path) -> Result<bool, String> {
    let fm = std::fs::metadata(from).map_err(|e| format!("{}: {e}", from.display()))?;
    if let Ok(tm) = std::fs::metadata(to) {
        if tm.len() == fm.len() && tm.modified().ok() == fm.modified().ok() {
            return Ok(false);
        }
    }
    if let Some(d) = to.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    // write next to the target first so a half-copied file is never mistaken for a good one
    let tmp = to.with_extension("copying");
    std::fs::copy(from, &tmp).map_err(|e| format!("{}: {e}", to.display()))?;
    if std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0) != fm.len() {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("{}: copy has the wrong size", to.display()));
    }
    std::fs::rename(&tmp, to).map_err(|e| format!("{}: {e}", to.display()))?;
    // keep the source's modified time, so an unchanged file is recognised next time
    if let Ok(t) = fm.modified() {
        if let Ok(f) = std::fs::OpenOptions::new().write(true).open(to) {
            let _ = f.set_modified(t);
        }
    }
    Ok(true)
}

fn copy_tree(from: &Path, to: &Path, n: &mut usize) -> Result<(), String> {
    for e in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let p = e.path();
        if p.is_dir() {
            if name != "_pending" {
                copy_tree(&p, &to.join(&name), n)?;
            }
        } else if !name.ends_with(".tmp") && !name.ends_with(".copying") && copy_if_changed(&p, &to.join(&name))? {
            *n += 1;
        }
    }
    Ok(())
}

/// Mirror one session folder (and the master CSV next to the data folder) into `backup_root`.
pub fn sync_session(data_dir: &Path, pid_dir: &Path, backup_root: &Path) -> Result<usize, String> {
    let r = (|| {
        check_dir(data_dir, backup_root)?;
        let mut n = 0;
        if let Some(name) = pid_dir.file_name() {
            if pid_dir.is_dir() {
                copy_tree(pid_dir, &backup_root.join(name), &mut n)?;
            }
        }
        copy_csv(data_dir, backup_root, &mut n)?;
        Ok(n)
    })();
    record(&r);
    r
}

/// Mirror every session (the "back up now" button, and the first copy of older sessions).
pub fn sync_all(data_dir: &Path, backup_root: &Path) -> Result<usize, String> {
    let r = (|| {
        check_dir(data_dir, backup_root)?;
        let mut n = 0;
        for e in std::fs::read_dir(data_dir).map_err(|e| e.to_string())?.flatten() {
            if e.path().join("session.json").is_file() {
                copy_tree(&e.path(), &backup_root.join(e.file_name()), &mut n)?;
            }
        }
        copy_csv(data_dir, backup_root, &mut n)?;
        Ok(n)
    })();
    record(&r);
    r
}

fn copy_csv(data_dir: &Path, backup_root: &Path, n: &mut usize) -> Result<(), String> {
    let csv: PathBuf = data_dir.join("measurements.csv");
    if csv.is_file() && copy_if_changed(&csv, &backup_root.join("measurements.csv"))? {
        *n += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sf_bk_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn mirrors_new_and_changed_files_but_not_pending() {
        let root = tmp("mirror");
        let (data, bk) = (root.join("data"), root.join("bk"));
        let s = data.join("SFP-0001");
        for sub in ["scans", "_pending", "rejected"] {
            std::fs::create_dir_all(s.join(sub)).unwrap();
        }
        std::fs::write(s.join("session.json"), b"{}").unwrap();
        std::fs::write(s.join("scans/right_1.png"), b"scan-a").unwrap();
        std::fs::write(s.join("_pending/left_1.png"), b"not yet accepted").unwrap();
        std::fs::write(data.join("measurements.csv"), b"a,b\n").unwrap();

        assert_eq!(sync_session(&data, &s, &bk).unwrap(), 3); // json, png, csv
        assert_eq!(std::fs::read(bk.join("SFP-0001/scans/right_1.png")).unwrap(), b"scan-a");
        assert!(!bk.join("SFP-0001/_pending").exists());
        assert_eq!(sync_session(&data, &s, &bk).unwrap(), 0); // nothing changed, nothing copied

        std::fs::write(s.join("session.json"), b"{\"x\":1}").unwrap();
        std::fs::write(s.join("scans/left_1.png"), b"scan-b").unwrap();
        assert_eq!(sync_all(&data, &bk).unwrap(), 2);
        assert_eq!(std::fs::read(bk.join("SFP-0001/session.json")).unwrap(), b"{\"x\":1}");
        // the main data is never touched
        assert!(s.join("_pending/left_1.png").exists());
    }

    #[test]
    fn refuses_a_backup_folder_that_overlaps_the_data_folder() {
        let root = tmp("overlap");
        let data = root.join("data");
        assert!(check_dir(&data, &data).unwrap_err().starts_with("BACKUP_SAME"));
        assert!(check_dir(&data, &data.join("copy")).unwrap_err().starts_with("BACKUP_SAME"));
        assert!(check_dir(&data, &root).unwrap_err().starts_with("BACKUP_SAME"));
        assert!(check_dir(&data, &root.join("elsewhere")).is_ok());
    }

    #[test]
    fn status_reports_unreachable_folder() {
        let root = tmp("status");
        assert!(!status(None).enabled);
        let st = status(Some(root.join("missing").to_str().unwrap()));
        assert!(st.enabled && !st.reachable);
        assert!(status(Some(root.to_str().unwrap())).reachable);
    }
}
