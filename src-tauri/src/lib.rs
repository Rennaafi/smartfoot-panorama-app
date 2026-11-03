pub mod analyze;
pub mod backup;
pub mod measure;
pub mod scanner;
pub mod session;

use base64::Engine;
use serde::Serialize;
use session::Settings;
use std::path::PathBuf;
use tauri::Manager;

fn settings_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("settings.json"))
}

fn settings(app: &tauri::AppHandle) -> Result<Settings, String> {
    let file = settings_file(app)?;
    let mut s = session::load_settings(&file);
    if s.station.is_empty() {
        // first run on this laptop: give it its own code and remember it
        s.station = session::generate_station();
        session::save_settings(&file, &s)?;
    }
    Ok(s)
}

/// Mirror one session to the second-copy folder, if one is set. Never fails the caller: a failed
/// copy is remembered in `backup::status` and shown in the app.
fn mirror(app: &tauri::AppHandle, dir: &std::path::Path) {
    if let Ok(s) = settings(app) {
        if let Some(b) = s.backup_dir.as_deref().map(str::trim).filter(|b| !b.is_empty()) {
            let _ = backup::sync_session(&PathBuf::from(&s.data_dir), dir, &PathBuf::from(b));
        }
    }
}

fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(PathBuf::from(settings(app)?.data_dir))
}

/// The scanner patients are scanned on: the setting if present, else the LiDE 300 driverless route.
fn choose_device(s: &Settings) -> Result<String, String> {
    let all = scanner::list_scanners()?;
    scanner::pick(&all, s.scanner.as_deref())
        .ok_or_else(|| "NOT_FOUND: scanner not found. Check the USB cable and that the scanner is switched on.".to_string())
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    settings(&app)
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, mut new: Settings) -> Result<Settings, String> {
    new.station = session::clean_station(&new.station);
    if new.station.is_empty() {
        return Err("BAD_STATION: the laptop code needs 1-4 letters or digits".into());
    }
    if !(new.rounds == 1 || new.rounds == 3) {
        return Err("rounds must be 1 or 3".into());
    }
    std::fs::create_dir_all(&new.data_dir).map_err(|e| format!("cannot use data folder: {e}"))?;
    new.backup_dir = new.backup_dir.take().map(|b| b.trim().to_string()).filter(|b| !b.is_empty());
    if let Some(b) = &new.backup_dir {
        backup::check_dir(&PathBuf::from(&new.data_dir), &PathBuf::from(b))?;
    }
    session::save_settings(&settings_file(&app)?, &new)?;
    Ok(new)
}

#[tauri::command]
async fn list_scanners() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(scanner::list_scanners).await.map_err(|e| e.to_string())?
}

/// Is a scanner connected, and through which route? Cheap enough to call every few seconds.
#[tauri::command]
async fn scanner_status(app: tauri::AppHandle) -> Result<scanner::ScannerStatus, String> {
    tauri::async_runtime::spawn_blocking(move || scanner::status(settings(&app)?.scanner.as_deref()))
        .await
        .map_err(|e| e.to_string())?
}

/// Hardware test (a short strip scan): returns how many seconds it took.
#[tauri::command]
async fn test_scanner(app: tauri::AppHandle) -> Result<f64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = settings(&app)?;
        let dev = choose_device(&s)?;
        scanner::test_scan(&dev, &std::env::temp_dir())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// State of the second copy: set or not, folder reachable, last result.
#[tauri::command]
fn backup_status(app: tauri::AppHandle) -> Result<backup::BackupStatus, String> {
    Ok(backup::status(settings(&app)?.backup_dir.as_deref()))
}

/// Copy everything to the second folder now (also the first copy of older sessions).
#[tauri::command]
async fn backup_now(app: tauri::AppHandle) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = settings(&app)?;
        let b = s.backup_dir.as_deref().map(str::trim).filter(|b| !b.is_empty()).ok_or("BACKUP_OFF: no backup folder is set")?;
        backup::sync_all(&PathBuf::from(&s.data_dir), &PathBuf::from(b))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn new_session_id(app: tauri::AppHandle) -> Result<String, String> {
    let s = settings(&app)?;
    Ok(session::next_pid(&PathBuf::from(&s.data_dir), &s.station))
}

#[derive(Serialize)]
struct StartInfo {
    pid: String,
    rounds: u8,
    dir: String,
    scanner: String,
}

#[tauri::command]
async fn start_session(app: tauri::AppHandle, pid: String, operator: String, age: Option<u32>, sex: String, note: String) -> Result<StartInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = settings(&app)?;
        let dev = choose_device(&s)?; // fail early if the scanner is not there
        scanner::warm_if_cold(&dev, &std::env::temp_dir())?; // also catches a closed LOCK switch before the patient sits down
        let dir = session::create_session(&PathBuf::from(&s.data_dir), &pid, &operator, age, &sex, &note, s.rounds, &dev)?;
        Ok(StartInfo { pid: session::clean_pid(&pid)?, rounds: s.rounds, dir: dir.to_string_lossy().into(), scanner: dev })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
struct ScanOutcome {
    preview: String, // PNG data URL, outline drawn on
    seconds: f64,
    width_px: u32,
    height_px: u32,
    found: bool,
    touches_edge: bool,
    area_pct: f32,
    warnings: Vec<String>,
}

fn outcome_from(path: &std::path::Path, seconds: f64, width_px: u32, height_px: u32) -> Result<ScanOutcome, String> {
    let a = analyze::analyze(path)?;
    let mut warnings = vec![];
    if !a.found {
        warnings.push("NO_FOOT".to_string());
    }
    if a.touches_edge {
        warnings.push("EDGE".to_string());
    }
    Ok(ScanOutcome {
        preview: format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&a.preview_png)),
        seconds,
        width_px,
        height_px,
        found: a.found,
        touches_edge: a.touches_edge,
        area_pct: a.area_frac * 100.0,
        warnings,
    })
}

#[tauri::command]
async fn scan_foot(app: tauri::AppHandle, pid: String, side: String, round: u8) -> Result<ScanOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = settings(&app)?;
        let dev = choose_device(&s)?;
        let dir = session::session_dir(&PathBuf::from(&s.data_dir), &pid)?;
        let out = dir.join("_pending").join(format!("{side}_{round}.png"));
        let r = scanner::scan(&dev, 300, &out)?;
        outcome_from(&out, r.seconds, r.width_px, r.height_px)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// A session left open (app closed or crashed mid-patient), if any.
#[derive(Serialize)]
struct Unfinished {
    pid: String,
    created: String,
    operator: String,
    done: usize,
    total: usize,
}

#[tauri::command]
fn find_unfinished(app: tauri::AppHandle) -> Result<Option<Unfinished>, String> {
    Ok(session::find_unfinished(&data_dir(&app)?).map(|s| Unfinished { total: s.rounds as usize * 2, done: s.scans.len(), pid: s.pid, created: s.created, operator: s.operator }))
}

#[derive(Serialize)]
struct ResumeInfo {
    start: StartInfo,
    /// index into the right/left x rounds sequence of the next scan to take
    idx: usize,
    /// a scan that was taken but not yet accepted or retaken when the app stopped, ready for review
    pending: Option<ScanOutcome>,
}

/// Re-open an unfinished session. Scans already accepted stay as they are; a scan that was waiting
/// for Accept/Retake comes back for review (its scan time is unknown, stored as 0). Any other
/// leftover file in _pending is moved to rejected/, never deleted.
#[tauri::command]
async fn resume_session(app: tauri::AppHandle, pid: String) -> Result<ResumeInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = session::session_dir(&data_dir(&app)?, &pid)?;
        let s = session::read_session(&dir)?;
        if s.completed || s.cancelled {
            return Err("NOT_RESUMABLE: this session is already finished or cancelled".to_string());
        }
        let idx = session::next_index(&s);
        let want = session::seq(s.rounds).get(idx).map(|(side, r)| format!("{side}_{r}.png"));
        let mut pending = None;
        if let Ok(rd) = std::fs::read_dir(dir.join("_pending")) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                let p = e.path();
                if Some(&name) == want.as_ref() && pending.is_none() {
                    if let Ok((w, h)) = image::image_dimensions(&p) {
                        pending = outcome_from(&p, 0.0, w, h).ok();
                    }
                }
                if pending.is_none() || Some(&name) != want.as_ref() {
                    let stem = name.trim_end_matches(".png");
                    let _ = std::fs::rename(&p, dir.join("rejected").join(format!("{stem}_resumed.png")));
                }
            }
        }
        Ok(ResumeInfo { start: StartInfo { pid: s.pid.clone(), rounds: s.rounds, dir: dir.to_string_lossy().into(), scanner: s.scanner.clone() }, idx, pending })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The scale used until the calibration tool exists: nominal 300 dpi, flagged as not calibrated.
fn current_scale() -> measure::Scale {
    measure::Scale::nominal(300.0)
}

#[tauri::command]
async fn accept_scan(app: tauri::AppHandle, pid: String, side: String, round: u8, seconds: f64, width_px: u32, height_px: u32, warnings: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dd = data_dir(&app)?;
        let dir = session::session_dir(&dd, &pid)?;
        session::accept(&dir, &side, round, seconds, width_px, height_px, warnings)?;
        // measure right away; a failure to find a foot leaves the scan saved but unmeasured
        let m = measure::measure_scan(&dir.join("scans").join(format!("{side}_{round}.png")), &current_scale(), None).ok();
        session::set_measurements(&dir, &side, round, m)?;
        session::rebuild_csv(&dd)?;
        mirror(&app, &dir);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn retake_scan(app: tauri::AppHandle, pid: String, side: String, round: u8) -> Result<(), String> {
    let dir = session::session_dir(&data_dir(&app)?, &pid)?;
    session::reject(&dir, &side, round)?;
    mirror(&app, &dir);
    Ok(())
}

#[tauri::command]
fn list_sessions(app: tauri::AppHandle) -> Result<Vec<session::SessionSummary>, String> {
    Ok(session::list_sessions(&data_dir(&app)?))
}

#[tauri::command]
fn get_session(app: tauri::AppHandle, pid: String) -> Result<session::SessionFile, String> {
    session::read_session(&session::session_dir(&data_dir(&app)?, &pid)?)
}

#[derive(Serialize)]
struct ScanView {
    preview: String, // JPEG data URL, scan at 1/4 size, no outline
    preview_w: u32,
    preview_h: u32,
    down: u32, // full-resolution pixels per preview pixel
    measurements: Option<measure::Measurements>,
    csv: String,
}

const VIEW_DOWN: u32 = 4;

#[tauri::command]
async fn get_scan_view(app: tauri::AppHandle, pid: String, side: String, round: u8) -> Result<ScanView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dd = data_dir(&app)?;
        let dir = session::session_dir(&dd, &pid)?;
        let s = session::read_session(&dir)?;
        let e = s.scans.iter().find(|e| e.side == side && e.round == round).ok_or("scan not found")?;
        let full = image::open(dir.join(&e.file)).map_err(|e| e.to_string())?.to_rgb8();
        let small = image::imageops::resize(&full, full.width() / VIEW_DOWN, full.height() / VIEW_DOWN, image::imageops::FilterType::Triangle);
        let mut jpg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpg, 85).encode_image(&small).map_err(|e| e.to_string())?;
        Ok(ScanView {
            preview: format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(&jpg)),
            preview_w: small.width(),
            preview_h: small.height(),
            down: VIEW_DOWN,
            measurements: e.measurements.clone(),
            csv: dd.join("measurements.csv").to_string_lossy().into(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Measure again, with hand-placed points (full-resolution pixels) or, if both are None, automatically.
#[tauri::command]
async fn remeasure(app: tauri::AppHandle, pid: String, side: String, round: u8, heel: Option<[f32; 2]>, toe: Option<[f32; 2]>) -> Result<measure::Measurements, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dd = data_dir(&app)?;
        let dir = session::session_dir(&dd, &pid)?;
        let pts = match (heel, toe) {
            (Some(h), Some(t)) => Some((h, t)),
            _ => None,
        };
        let m = measure::measure_scan(&dir.join("scans").join(format!("{side}_{round}.png")), &current_scale(), pts)?;
        session::set_measurements(&dir, &side, round, Some(m.clone()))?;
        session::rebuild_csv(&dd)?;
        mirror(&app, &dir);
        Ok(m)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn cancel_session(app: tauri::AppHandle, pid: String) -> Result<bool, String> {
    let dir = session::session_dir(&data_dir(&app)?, &pid)?;
    let removed = session::cancel(&dir)?;
    mirror(&app, &dir); // a removed (empty) session leaves nothing to copy
    Ok(removed)
}

#[tauri::command]
fn finish_session(app: tauri::AppHandle, pid: String) -> Result<(), String> {
    let dir = session::session_dir(&data_dir(&app)?, &pid)?;
    session::complete(&dir)?;
    mirror(&app, &dir);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            list_scanners,
            scanner_status,
            backup_status,
            backup_now,
            test_scanner,
            new_session_id,
            find_unfinished,
            resume_session,
            start_session,
            scan_foot,
            accept_scan,
            retake_scan,
            finish_session,
            cancel_session,
            list_sessions,
            get_session,
            get_scan_view,
            remeasure
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
