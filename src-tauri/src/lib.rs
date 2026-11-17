pub mod analyze;
pub mod backup;
pub mod export;
pub mod inlow;
pub mod measure;
pub mod scanner;
pub mod session;

use base64::Engine;
use serde::Serialize;
use session::{Patient, Sesi, Settings};
use std::path::{Path, PathBuf};
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

fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(PathBuf::from(settings(app)?.data_dir))
}

/// After anything was saved: rebuild the Excel files and the research CSV, then mirror what changed
/// to the second-copy folder. Never fails the caller; problems come back as messages for the screen
/// (e.g. "Rekap semua sesi.xlsx is open in Excel").
fn after_save(app: &tauri::AppHandle, sesi_dir: Option<&Path>, patient_dir: Option<&Path>) -> Vec<String> {
    let Ok(s) = settings(app) else { return vec![] };
    let dd = PathBuf::from(&s.data_dir);
    let errs = export::rebuild(&dd, sesi_dir);
    if let Some(b) = s.backup_dir.as_deref().map(str::trim).filter(|b| !b.is_empty()) {
        let _ = backup::sync_part(&dd, sesi_dir, patient_dir, &PathBuf::from(b));
    }
    errs
}

/// The scanner patients are scanned on: the setting if present, else the LiDE 300 driverless route.
fn choose_device(s: &Settings) -> Result<String, String> {
    let all = scanner::list_scanners()?;
    scanner::pick(&all, s.scanner.as_deref())
        .ok_or_else(|| "NOT_FOUND: scanner not found. Check the USB cable and that the scanner is switched on.".to_string())
}

/// The ruler calibration of 13 Nov 2026 (see measure::CAL_DPI_*).
fn current_scale() -> measure::Scale {
    measure::Scale::calibrated()
}

// ------------------------------------------------------------------------------------------------
// Settings, scanner, backup

/// Windows folder picker for the Settings page. `start` is the folder currently typed (if it exists the
/// dialog opens there). Returns None when the user cancels.
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle, start: Option<String>) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let mut d = app.dialog().file();
        if let Some(s) = start.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let mut p = PathBuf::from(s);
            while !p.is_dir() && p.pop() {}
            if p.is_dir() {
                d = d.set_directory(p);
            }
        }
        Ok(d.blocking_pick_folder().and_then(|f| f.into_path().ok()).map(|p| p.to_string_lossy().to_string()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    settings(&app)
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, new: Settings) -> Result<Settings, String> {
    let mut new = session::validate_settings(new)?;
    std::fs::create_dir_all(&new.data_dir).map_err(|e| format!("cannot use data folder: {e}"))?;
    if let Some(b) = &new.backup_dir {
        backup::check_dir(&PathBuf::from(&new.data_dir), &PathBuf::from(b))?;
    }
    // the remembered operator list is kept by the app, not edited on the Settings page
    if new.operators.is_empty() {
        new.operators = settings(&app)?.operators;
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

/// Where data is saved and whether that place can be used right now (shown on the start screen).
#[derive(Serialize)]
struct DataDirStatus {
    path: String,
    /// The folder exists, or its drive exists so the folder can be created on the first save.
    ok: bool,
}

pub fn dir_usable(p: &Path) -> bool {
    // "E:\Data SmartFoot" on a flash disk that was pulled out: the drive root E:\ is gone
    let root: PathBuf = p.components().take(2).collect();
    p.is_dir() || (p.is_absolute() && root.is_dir())
}

#[tauri::command]
fn data_dir_status(app: tauri::AppHandle) -> Result<DataDirStatus, String> {
    let d = data_dir(&app)?;
    Ok(DataDirStatus { ok: dir_usable(&d), path: d.to_string_lossy().into() })
}

/// The INLOW items, options and questions, so the screen is built from the same table as the scores.
#[derive(Serialize)]
struct InlowItem {
    key: &'static str,
    group: &'static str,
    label: &'static str,
    options: Vec<(u8, &'static str)>,
}

#[derive(Serialize)]
struct InlowSpec {
    items: Vec<InlowItem>,
    questions: [&'static str; 4],
    notes: &'static str,
}

#[tauri::command]
fn inlow_spec() -> InlowSpec {
    InlowSpec {
        items: inlow::ITEMS.iter().map(|i| InlowItem { key: i.key, group: i.group, label: i.label, options: i.options.to_vec() }).collect(),
        questions: inlow::QUESTIONS,
        notes: inlow::SCORING_NOTES,
    }
}

// ------------------------------------------------------------------------------------------------
// Screening session (one day, one place)

/// One line of the session's patient list.
#[derive(Serialize)]
struct PatientRow {
    key: String,
    no: u32,
    nama: String,
    operator: String,
    scans: usize,
    inlow_complete: bool,
    completed: bool,
    cancelled: bool,
    kesimpulan: String,
}

fn patient_row(dd: &Path, dir: &Path, p: &Patient) -> PatientRow {
    PatientRow {
        key: session::relative(dd, dir),
        no: p.no,
        nama: p.nama.clone(),
        operator: p.operator.clone(),
        scans: p.scans.len(),
        inlow_complete: p.inlow.kanan.complete() && p.inlow.kiri.complete(),
        completed: p.completed,
        cancelled: p.cancelled,
        kesimpulan: export::kesimpulan(p),
    }
}

#[derive(Serialize)]
struct SesiState {
    key: String,
    sesi: Sesi,
    /// The session belongs to an earlier day (offer to end it).
    stale: bool,
    patients: Vec<PatientRow>,
    /// A patient that was started but not finished (app closed mid-patient), if any.
    unfinished: Option<String>,
    /// Messages from rebuilding the Excel files (e.g. a file open in Excel).
    file_warnings: Vec<String>,
}

fn sesi_state(dd: &Path, dir: &Path, s: Sesi, file_warnings: Vec<String>) -> SesiState {
    let pats = session::list_patients(dir);
    SesiState {
        key: session::relative(dd, dir),
        stale: s.date != session::today(),
        unfinished: session::find_unfinished(dir).map(|(d, _)| session::relative(dd, &d)),
        patients: pats.iter().map(|(d, p)| patient_row(dd, d, p)).collect(),
        sesi: s,
        file_warnings,
    }
}

fn open_sesi_dir(app: &tauri::AppHandle) -> Result<(PathBuf, PathBuf, Sesi), String> {
    let dd = data_dir(app)?;
    let (d, s) = session::open_sesi(&dd).ok_or("NO_SESI: no session is open")?;
    Ok((dd, d, s))
}

/// The open session, if any (shown when the app starts).
#[tauri::command]
fn current_sesi(app: tauri::AppHandle) -> Result<Option<SesiState>, String> {
    let dd = data_dir(&app)?;
    Ok(session::open_sesi(&dd).map(|(d, s)| sesi_state(&dd, &d, s, vec![])))
}

#[tauri::command]
fn start_sesi(app: tauri::AppHandle, place: String, operator: String) -> Result<SesiState, String> {
    let file = settings_file(&app)?;
    let mut st = settings(&app)?;
    let dd = PathBuf::from(&st.data_dir);
    std::fs::create_dir_all(&dd).map_err(|e| format!("cannot use data folder: {e}"))?;
    let (d, s) = session::start_sesi(&dd, &place, &operator, &st.station)?;
    session::remember_operator(&mut st, &s.operator);
    let _ = session::save_settings(&file, &st);
    let w = after_save(&app, Some(&d), None);
    Ok(sesi_state(&dd, &d, s, w))
}

#[tauri::command]
fn change_operator(app: tauri::AppHandle, operator: String) -> Result<SesiState, String> {
    let (dd, d, _) = open_sesi_dir(&app)?;
    let s = session::change_operator(&d, &operator)?;
    let file = settings_file(&app)?;
    let mut st = settings(&app)?;
    session::remember_operator(&mut st, &s.operator);
    let _ = session::save_settings(&file, &st);
    let w = after_save(&app, Some(&d), None);
    Ok(sesi_state(&dd, &d, s, w))
}

/// Break (pause = true) or back to work (false).
#[tauri::command]
fn pause_sesi(app: tauri::AppHandle, pause: bool) -> Result<SesiState, String> {
    let (dd, d, _) = open_sesi_dir(&app)?;
    let s = session::set_paused(&d, pause)?;
    let w = after_save(&app, Some(&d), None);
    Ok(sesi_state(&dd, &d, s, w))
}

#[tauri::command]
fn end_sesi(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let (_, d, _) = open_sesi_dir(&app)?;
    if let Some((pd, _)) = session::find_unfinished(&d) {
        return Err(format!("PATIENT_OPEN: patient {} is not finished yet", pd.file_name().unwrap_or_default().to_string_lossy()));
    }
    session::end_sesi(&d)?;
    Ok(after_save(&app, Some(&d), None))
}

/// Every session in the data folder with its patients, newest first (for Detail pengukuran).
#[derive(Serialize)]
struct SesiListItem {
    key: String,
    date: String,
    place: String,
    ended: bool,
    patients: Vec<PatientRow>,
}

#[tauri::command]
fn list_all(app: tauri::AppHandle) -> Result<Vec<SesiListItem>, String> {
    let dd = data_dir(&app)?;
    let mut v: Vec<SesiListItem> = session::list_sesi(&dd)
        .into_iter()
        .map(|(d, s)| SesiListItem {
            key: session::relative(&dd, &d),
            date: s.date.clone(),
            place: s.place.clone(),
            ended: s.ended.is_some(),
            patients: session::list_patients(&d).iter().map(|(pd, p)| patient_row(&dd, pd, p)).collect(),
        })
        .collect();
    v.reverse();
    Ok(v)
}

/// Patient folders that cannot be read; shown as a warning, never deleted.
#[tauri::command]
fn damaged_patients(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    Ok(session::damaged_patients(&data_dir(&app)?))
}

// ------------------------------------------------------------------------------------------------
// Patient

fn patient_dir(app: &tauri::AppHandle, key: &str) -> Result<(PathBuf, PathBuf), String> {
    let dd = data_dir(app)?;
    let dir = session::resolve(&dd, key)?;
    if !dir.join(session::PATIENT_FILE).is_file() {
        return Err("NO_PATIENT: patient folder not found".into());
    }
    Ok((dd, dir))
}

/// The session folder a patient folder belongs to (two levels up).
fn sesi_of(patient_dir: &Path) -> Option<&Path> {
    patient_dir.parent().and_then(Path::parent)
}

#[derive(Serialize)]
struct PatientInfo {
    key: String,
    patient: Patient,
    /// "left", "right" or None when both feet are scanned.
    next_side: Option<&'static str>,
    file_warnings: Vec<String>,
}

fn patient_info(dd: &Path, dir: &Path, w: Vec<String>) -> Result<PatientInfo, String> {
    let p = session::read_patient(dir)?;
    Ok(PatientInfo { key: session::relative(dd, dir), next_side: session::next_side(&p), patient: p, file_warnings: w })
}

#[tauri::command]
async fn start_patient(app: tauri::AppHandle, form: session::PatientForm) -> Result<PatientInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = settings(&app)?;
        let (dd, sd, sesi) = open_sesi_dir(&app)?;
        if sesi.paused() {
            return Err("SESI_PAUSED: the session is on a break".into());
        }
        // check the form before touching the scanner or the disk
        session::validate_patient(&form)?;
        let dev = choose_device(&s)?; // fail early if the scanner is not there
        scanner::warm_if_cold(&dev, &std::env::temp_dir())?; // also catches a closed LOCK switch before the patient steps up
        let dir = session::create_patient(&sd, &form, &dev)?;
        let w = after_save(&app, Some(&sd), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_patient(app: tauri::AppHandle, key: String) -> Result<PatientInfo, String> {
    let (dd, dir) = patient_dir(&app, &key)?;
    patient_info(&dd, &dir, vec![])
}

#[tauri::command]
fn update_patient(app: tauri::AppHandle, key: String, form: session::PatientForm) -> Result<PatientInfo, String> {
    let (dd, dir) = patient_dir(&app, &key)?;
    session::update_patient_form(&dir, &form)?;
    let w = after_save(&app, sesi_of(&dir), Some(&dir));
    patient_info(&dd, &dir, w)
}

#[derive(Serialize)]
struct ScanOutcome {
    preview: String, // PNG data URL, outline drawn on
    plain: String,   // same image without the outline
    preview_w: u32,
    preview_h: u32,
    seconds: f64,
    width_px: u32,
    height_px: u32,
    found: bool,
    touches_edge: bool,
    area_pct: f32,
    warnings: Vec<String>,
    /// Shown on the review screen; None when no foot could be measured.
    measurements: Option<measure::Measurements>,
}

fn outcome_from(path: &Path, seconds: f64, width_px: u32, height_px: u32) -> Result<ScanOutcome, String> {
    let a = analyze::analyze(path)?;
    let mut warnings = vec![];
    if !a.found {
        warnings.push("NO_FOOT".to_string());
    }
    // no foot means no edge question: "no foot" and "touches the edge" together only confuse
    if a.touches_edge && a.found {
        warnings.push("EDGE".to_string());
    }
    if a.odd_shape {
        warnings.push("ODD_SHAPE".to_string());
    }
    if a.extra_objects {
        warnings.push("TWO_OBJECTS".to_string());
    }
    let b64 = |b: &[u8]| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(b));
    let measurements = if a.found { measure::measure_scan(path, &current_scale(), None).ok() } else { None };
    Ok(ScanOutcome {
        preview: b64(&a.preview_png),
        plain: b64(&a.plain_png),
        preview_w: a.preview_w,
        preview_h: a.preview_h,
        measurements,
        seconds,
        width_px,
        height_px,
        found: a.found,
        touches_edge: a.touches_edge && a.found,
        area_pct: a.area_frac * 100.0,
        warnings,
    })
}

/// The right foot is scanned with the patient turned around: turn that image 180° so both feet are
/// stored the same way up. A 180° turn keeps the image axes, so the calibration still applies.
pub fn rotate_saved_scan(path: &Path) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("cannot read scan: {e}"))?;
    img.rotate180().save(path).map_err(|e| format!("cannot save turned scan: {e}"))
}

pub fn rotates(side: &str) -> bool {
    side == "right"
}

#[tauri::command]
async fn scan_foot(app: tauri::AppHandle, key: String, side: String) -> Result<ScanOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !session::SIDES.contains(&side.as_str()) {
            return Err("BAD_SIDE".to_string());
        }
        let s = settings(&app)?;
        let dev = choose_device(&s)?;
        let (_, dir) = patient_dir(&app, &key)?;
        let out = session::pending_file(&dir, &side);
        std::fs::create_dir_all(dir.join(session::PENDING)).map_err(|e| e.to_string())?;
        let r = scanner::scan(&dev, 300, &out)?;
        if rotates(&side) {
            rotate_saved_scan(&out)?;
        }
        outcome_from(&out, r.seconds, r.width_px, r.height_px)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Re-open an unfinished patient. A scan that was waiting for Accept/Retake comes back for review
/// (its scan time is unknown, stored as 0); any other leftover in _pending goes to ditolak\.
#[derive(Serialize)]
struct ResumeInfo {
    info: PatientInfo,
    pending: Option<ScanOutcome>,
}

#[tauri::command]
async fn resume_patient(app: tauri::AppHandle, key: String) -> Result<ResumeInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        let p = session::read_patient(&dir)?;
        if p.completed || p.cancelled {
            return Err("NOT_RESUMABLE: this patient is already finished or cancelled".to_string());
        }
        let want = session::next_side(&p);
        let mut pending = None;
        for side in session::SIDES {
            let f = session::pending_file(&dir, side);
            if !f.exists() {
                continue;
            }
            if Some(side) == want && pending.is_none() {
                if let Ok((w, h)) = image::image_dimensions(&f) {
                    pending = outcome_from(&f, 0.0, w, h).ok();
                }
            }
            if pending.is_none() || Some(side) != want {
                session::reject(&dir, side)?;
            }
        }
        Ok(ResumeInfo { info: patient_info(&dd, &dir, vec![])?, pending })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn accept_scan(app: tauri::AppHandle, key: String, side: String, seconds: f64, width_px: u32, height_px: u32, warnings: Vec<String>) -> Result<PatientInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        session::accept(&dir, &side, seconds, width_px, height_px, rotates(&side), warnings)?;
        // measure right away; a failure to find a foot leaves the scan saved but unmeasured
        let m = measure::measure_scan(&dir.join(session::raw_scan_name(&side)), &current_scale(), None).ok();
        session::set_measurements(&dir, &side, m)?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn retake_scan(app: tauri::AppHandle, key: String, side: String) -> Result<(), String> {
    let (_, dir) = patient_dir(&app, &key)?;
    session::reject(&dir, &side)?;
    after_save(&app, sesi_of(&dir), Some(&dir));
    Ok(())
}

#[derive(Serialize)]
struct ScanView {
    preview: String, // JPEG data URL, scan at 1/4 size, no outline
    preview_w: u32,
    preview_h: u32,
    down: u32, // full-resolution pixels per preview pixel
    measurements: Option<measure::Measurements>,
}

const VIEW_DOWN: u32 = 4;

#[tauri::command]
async fn get_scan_view(app: tauri::AppHandle, key: String, side: String) -> Result<ScanView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (_, dir) = patient_dir(&app, &key)?;
        let p = session::read_patient(&dir)?;
        let e = p.scans.iter().find(|e| e.side == side).ok_or("scan not found")?;
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
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Measure again, with hand-placed points (full-resolution pixels) or, if both are None, automatically.
#[tauri::command]
async fn remeasure(app: tauri::AppHandle, key: String, side: String, heel: Option<[f32; 2]>, toe: Option<[f32; 2]>) -> Result<measure::Measurements, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (_, dir) = patient_dir(&app, &key)?;
        let pts = match (heel, toe) {
            (Some(h), Some(t)) => Some((h, t)),
            _ => None,
        };
        let m = measure::measure_scan(&dir.join(session::raw_scan_name(&side)), &current_scale(), pts)?;
        session::set_measurements(&dir, &side, Some(m.clone()))?;
        after_save(&app, sesi_of(&dir), Some(&dir));
        Ok(m)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn save_inlow(app: tauri::AppHandle, key: String, inlow: inlow::Inlow) -> Result<PatientInfo, String> {
    let (dd, dir) = patient_dir(&app, &key)?;
    session::set_inlow(&dir, inlow)?;
    let w = after_save(&app, sesi_of(&dir), Some(&dir));
    patient_info(&dd, &dir, w)
}

#[tauri::command]
fn save_notes(app: tauri::AppHandle, key: String, notes: session::Notes) -> Result<PatientInfo, String> {
    let (dd, dir) = patient_dir(&app, &key)?;
    session::set_notes(&dir, &notes)?;
    let w = after_save(&app, sesi_of(&dir), Some(&dir));
    patient_info(&dd, &dir, w)
}

#[tauri::command]
fn save_marks(app: tauri::AppHandle, key: String, marks: Vec<session::Mark>) -> Result<PatientInfo, String> {
    let (dd, dir) = patient_dir(&app, &key)?;
    session::set_marks(&dir, marks)?;
    let w = after_save(&app, sesi_of(&dir), Some(&dir));
    patient_info(&dd, &dir, w)
}

#[tauri::command]
fn cancel_patient(app: tauri::AppHandle, key: String) -> Result<bool, String> {
    let (_, dir) = patient_dir(&app, &key)?;
    let sd = sesi_of(&dir).map(Path::to_path_buf);
    let removed = session::cancel(&dir)?;
    after_save(&app, sd.as_deref(), if removed { None } else { Some(&dir) });
    Ok(removed)
}

#[tauri::command]
fn finish_patient(app: tauri::AppHandle, key: String) -> Result<PatientInfo, String> {
    let (dd, dir) = patient_dir(&app, &key)?;
    session::complete(&dir)?;
    let w = after_save(&app, sesi_of(&dir), Some(&dir));
    patient_info(&dd, &dir, w)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            pick_folder,
            set_settings,
            list_scanners,
            scanner_status,
            backup_status,
            backup_now,
            test_scanner,
            data_dir_status,
            inlow_spec,
            current_sesi,
            start_sesi,
            change_operator,
            pause_sesi,
            end_sesi,
            list_all,
            damaged_patients,
            start_patient,
            get_patient,
            update_patient,
            scan_foot,
            resume_patient,
            accept_scan,
            retake_scan,
            get_scan_view,
            remeasure,
            save_inlow,
            save_notes,
            save_marks,
            cancel_patient,
            finish_patient
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_folder_is_usable_only_when_its_drive_is_there() {
        let tmp = std::env::temp_dir();
        assert!(dir_usable(&tmp));
        assert!(dir_usable(&tmp.join("not_created_yet_sf")), "a missing folder on a present drive can be created");
        let gone = ('G'..='Z').map(|c| format!("{c}:\\")).find(|r| !Path::new(r).exists()).expect("a free drive letter");
        assert!(!dir_usable(&Path::new(&gone).join("Data SmartFoot")));
        assert!(!dir_usable(Path::new("relative_data_folder_sf")));
    }

    #[test]
    fn turned_right_foot_is_rotated_180_and_keeps_its_size() {
        use image::{Rgb, RgbImage};
        let mut img = RgbImage::from_pixel(40, 60, Rgb([0, 0, 0]));
        img.put_pixel(1, 2, Rgb([255, 0, 0])); // marker near the top-left
        let p = std::env::temp_dir().join(format!("sf_rot_{}.png", std::process::id()));
        img.save(&p).unwrap();
        rotate_saved_scan(&p).unwrap();
        let r = image::open(&p).unwrap().to_rgb8();
        let _ = std::fs::remove_file(&p);
        assert_eq!(r.dimensions(), (40, 60));
        assert_eq!(r.get_pixel(40 - 1 - 1, 60 - 1 - 2), &Rgb([255, 0, 0]));
        assert!(rotates("right") && !rotates("left"));
    }
}
