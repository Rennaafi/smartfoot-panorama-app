pub mod analyze;
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
    Ok(session::load_settings(&settings_file(app)?))
}

fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(PathBuf::from(settings(app)?.data_dir))
}

/// The scanner patients are scanned on: the setting if present, else the LiDE 300 driverless route.
fn choose_device(s: &Settings) -> Result<String, String> {
    let all = scanner::list_scanners()?;
    if let Some(want) = &s.scanner {
        if all.contains(want) {
            return Ok(want.clone());
        }
    }
    all.iter()
        .find(|d| d.contains("LiDE 300") && d.contains("HTTP"))
        .or_else(|| all.iter().find(|d| d.contains("LiDE 300")))
        .cloned()
        .ok_or_else(|| "NOT_FOUND: scanner not found. Check the USB cable and that the scanner is switched on.".to_string())
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    settings(&app)
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, new: Settings) -> Result<Settings, String> {
    if !(new.rounds == 1 || new.rounds == 3) {
        return Err("rounds must be 1 or 3".into());
    }
    std::fs::create_dir_all(&new.data_dir).map_err(|e| format!("cannot use data folder: {e}"))?;
    session::save_settings(&settings_file(&app)?, &new)?;
    Ok(new)
}

#[tauri::command]
async fn list_scanners() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(scanner::list_scanners).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn new_session_id(app: tauri::AppHandle) -> Result<String, String> {
    Ok(session::next_pid(&data_dir(&app)?))
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

#[tauri::command]
async fn scan_foot(app: tauri::AppHandle, pid: String, side: String, round: u8) -> Result<ScanOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = settings(&app)?;
        let dev = choose_device(&s)?;
        let dir = session::session_dir(&PathBuf::from(&s.data_dir), &pid)?;
        let out = dir.join("_pending").join(format!("{side}_{round}.png"));
        let r = scanner::scan(&dev, 300, &out)?;
        let a = analyze::analyze(&out)?;
        let mut warnings = vec![];
        if !a.found {
            warnings.push("NO_FOOT".to_string());
        }
        if a.touches_edge {
            warnings.push("EDGE".to_string());
        }
        Ok(ScanOutcome {
            preview: format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&a.preview_png)),
            seconds: r.seconds,
            width_px: r.width_px,
            height_px: r.height_px,
            found: a.found,
            touches_edge: a.touches_edge,
            area_pct: a.area_frac * 100.0,
            warnings,
        })
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
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn retake_scan(app: tauri::AppHandle, pid: String, side: String, round: u8) -> Result<(), String> {
    session::reject(&session::session_dir(&data_dir(&app)?, &pid)?, &side, round)
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
        Ok(m)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn cancel_session(app: tauri::AppHandle, pid: String) -> Result<bool, String> {
    session::cancel(&session::session_dir(&data_dir(&app)?, &pid)?)
}

#[tauri::command]
fn finish_session(app: tauri::AppHandle, pid: String) -> Result<(), String> {
    session::complete(&session::session_dir(&data_dir(&app)?, &pid)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            list_scanners,
            new_session_id,
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
