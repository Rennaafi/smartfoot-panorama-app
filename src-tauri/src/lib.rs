pub mod analyze;
pub mod backup;
pub mod contact;
pub mod export;
pub mod figures;
pub mod inlow;
pub mod measure;
pub mod pdf;
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
    let mut errs = export::rebuild(&dd, sesi_dir);
    // the patient's result sheet first, so the backup below copies the new one
    if let Some(w) = patient_dir.and_then(|pd| pdf::ensure_sheet(pd, &s)) {
        errs.push(w);
    }
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

/// The operator book next to SmartFoot.exe (flash disk: E:\SmartFoot\Panduan SmartFoot.pdf).
pub const GUIDE_PDF: &str = "Panduan SmartFoot.pdf";

/// "? Panduan": open the book PDF that sits next to the exe in the default PDF viewer.
/// NOT_FOUND when it is not there (e.g. a development build), so the app can show its own pictures.
#[tauri::command]
fn open_guide() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let pdf = exe.parent().map(|d| d.join(GUIDE_PDF)).ok_or("NOT_FOUND: no exe folder")?;
    if !pdf.is_file() {
        return Err(format!("NOT_FOUND: {}", pdf.display()));
    }
    tauri_plugin_opener::open_path(&pdf, None::<&str>).map_err(|e| e.to_string())?;
    Ok(pdf.display().to_string())
}

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
    /// "Hasil skrining": one plain sentence per foot (right first) and the client's advice list.
    hasil: Vec<String>,
    saran: Vec<String>,
    /// Feet whose reference points nobody has confirmed yet ("Belum diperiksa").
    unchecked: Vec<String>,
    /// Sending of the result sheet: "tidak" / "belum" / "ulang" / "terkirim" (session::send_status).
    send: &'static str,
    /// Something is still to do ("Perlu dilengkapi"): points, INLOW or sending.
    todo: bool,
}

fn patient_row(dd: &Path, dir: &Path, p: &Patient) -> PatientRow {
    let send = session::send_status(p, &pdf::fingerprint_of(dir, p));
    let unchecked = session::unchecked_sides(p);
    let inlow_complete = p.inlow.kanan.complete() && p.inlow.kiri.complete();
    PatientRow {
        send,
        todo: p.completed && !p.cancelled && (!unchecked.is_empty() || !inlow_complete || send == "belum" || send == "ulang"),
        key: session::relative(dd, dir),
        no: p.no,
        nama: p.nama.clone(),
        operator: p.operator.clone(),
        scans: p.scans.len(),
        inlow_complete,
        completed: p.completed,
        cancelled: p.cancelled,
        kesimpulan: export::kesimpulan(p),
        hasil: export::hasil(p),
        saran: export::saran(p),
        unchecked,
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
    /// The two feet in this patient's scanning order.
    order: [&'static str; 2],
    /// Sending of the result sheet (session::send_status).
    send: &'static str,
    /// The result sheet in the folder, if made (file name only).
    sheet: Option<String>,
    file_warnings: Vec<String>,
}

fn patient_info(dd: &Path, dir: &Path, w: Vec<String>) -> Result<PatientInfo, String> {
    let p = session::read_patient(dir)?;
    Ok(PatientInfo {
        key: session::relative(dd, dir),
        next_side: session::next_side(&p),
        order: session::side_order(&p),
        send: session::send_status(&p, &pdf::fingerprint_of(dir, &p)),
        sheet: pdf::find_sheet(dir).and_then(|f| f.file_name().map(|n| n.to_string_lossy().to_string())),
        patient: p,
        file_warnings: w,
    })
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

/// Run a patient command off the window's thread (saving may make the result sheet, ~1 s).
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

/// Opening a patient also makes a missing or out-of-date result sheet (patients from before 0.3.0).
#[tauri::command]
async fn get_patient(app: tauri::AppHandle, key: String) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        let w = pdf::ensure_sheet(&dir, &settings(&app)?).into_iter().collect();
        patient_info(&dd, &dir, w)
    })
    .await
}

#[tauri::command]
async fn update_patient(app: tauri::AppHandle, key: String, form: session::PatientForm) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        session::update_patient_form(&dir, &form)?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
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

/// Only a standing patient turns around for the right foot; a seated patient just moves the other
/// foot onto the glass, so nothing is turned (found 2026-11-19: seated right feet came out upside
/// down). Older records without a posture count as standing.
pub fn rotates(side: &str, posisi: &str) -> bool {
    side == "right" && posisi != "duduk"
}

/// The scanner writes here first; only a finished (and, if needed, turned) image is renamed to the
/// pending file, so closing the app mid-scan can never leave an unturned pending scan (2026-11-19).
fn scanning_file(dir: &Path, side: &str) -> PathBuf {
    dir.join(session::PENDING).join(format!("{side}_scanning.png"))
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
        let posisi = session::read_patient(&dir)?.posisi;
        let out = session::pending_file(&dir, &side);
        let tmp = scanning_file(&dir, &side);
        std::fs::create_dir_all(dir.join(session::PENDING)).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&tmp);
        let r = scanner::scan(&dev, 300, &tmp)?;
        if rotates(&side, &posisi) {
            rotate_saved_scan(&tmp)?;
        }
        std::fs::rename(&tmp, &out).map_err(|e| format!("cannot keep the scan: {e}"))?;
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
        // a scan that was still being written when the app closed is incomplete: drop it
        for side in session::SIDES {
            let _ = std::fs::remove_file(scanning_file(&dir, side));
        }
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
        let posisi = session::read_patient(&dir)?.posisi;
        session::accept(&dir, &side, seconds, width_px, height_px, rotates(&side, &posisi), warnings)?;
        // measure right away; a failure to find a foot leaves the scan saved but unmeasured
        let m = measure::measure_scan(&dir.join(session::raw_scan_name(&side)), &current_scale(), None).ok();
        session::set_measurements(&dir, &side, m)?;
        let mut w = write_pictures(&dir, &side, true);
        w.extend(after_save(&app, sesi_of(&dir), Some(&dir)));
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
    /// The latest point check of this foot (None = not checked yet).
    check: Option<session::PointCheck>,
    /// Graded contact picture ("Peta kontak") at preview size, JPEG data URL, for the layer switch.
    contact_preview: Option<String>,
    /// The scan with the detected outline drawn on (PNG data URL), only when asked for: it needs the
    /// outline finder to run again (the "Garis tepi" layer when a foot is checked later).
    outline_preview: Option<String>,
}

const VIEW_DOWN: u32 = 4;

#[tauri::command]
async fn get_scan_view(app: tauri::AppHandle, key: String, side: String, outline: Option<bool>) -> Result<ScanView, String> {
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
            check: e.check.clone(),
            contact_preview: image::open(dir.join(figures::contact_map_name(&side))).ok().and_then(|g| {
                let g = image::imageops::resize(&g.to_rgb8(), small.width(), small.height(), image::imageops::FilterType::Triangle);
                let mut b = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut b, 85).encode_image(&g).ok()?;
                Some(format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(&b)))
            }),
            outline_preview: if outline.unwrap_or(false) {
                analyze::analyze(&dir.join(&e.file)).ok().filter(|a| a.found).map(|a| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&a.preview_png)))
            } else {
                None
            },
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
        write_pictures(&dir, &side, false);
        after_save(&app, sesi_of(&dir), Some(&dir));
        Ok(m)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Pictures for one accepted scan. `scan_pictures` = also the scan-only ones (Peta kontak + Arsip
/// analisis, frozen on the day); the arch-index picture follows the current measurements, the Clarke
/// picture the confirmed check. Problems come back as warnings: a picture must never block a patient.
fn write_pictures(dir: &Path, side: &str, scan_pictures: bool) -> Vec<String> {
    let mut warn = vec![];
    let Ok(p) = session::read_patient(dir) else { return warn };
    let Some(e) = p.scans.iter().find(|e| e.side == side) else { return warn };
    let scan = dir.join(&e.file);
    if scan_pictures {
        match figures::write_scan_images(dir, side, &scan) {
            Ok(_) => {
                if let Ok(mut p2) = session::read_patient(dir) {
                    if let Some(e2) = p2.scans.iter_mut().find(|e| e.side == side) {
                        e2.images_version = Some(env!("CARGO_PKG_VERSION").to_string());
                    }
                    let _ = session::write_patient(dir, &p2);
                }
            }
            Err(er) => warn.push(format!("PICTURE: {er}")),
        }
    }
    if let Some(m) = &e.measurements {
        if let Err(er) = figures::write_arch_index_image(dir, side, &scan, m) {
            warn.push(format!("PICTURE: {er}"));
        }
    }
    if let Some(c) = e.check.as_ref().filter(|c| c.status == "diperiksa") {
        if let (Some(pts), Some(deg)) = (&c.clarke_pts, c.clarke_deg) {
            if let Err(er) = figures::write_clarke_image(dir, side, &scan, pts, deg) {
                warn.push(format!("PICTURE: {er}"));
            }
        }
    } else {
        // an older confirmed picture must not outlive a newer unconfirmed state
        let _ = std::fs::remove_file(dir.join(figures::clarke_name(side)));
    }
    warn
}

/// The operator on duty in the patient's session (falls back to the one who registered the patient).
fn operator_now(dir: &Path, p: &Patient) -> String {
    sesi_of(dir).and_then(|sd| session::read_sesi(sd).ok()).map(|s| s.operator).unwrap_or_else(|| p.operator.clone())
}

fn dist_mm(a: [f32; 2], b: [f32; 2], sc: &measure::Scale) -> f32 {
    (((a[0] - b[0]) * sc.x_mm_per_px).powi(2) + ((a[1] - b[1]) * sc.y_mm_per_px).powi(2)).sqrt()
}

/// "Titik sudah benar": the operator confirms (after dragging, if needed) the AI heel-centre and
/// second-toe points and Clarke's three points, full-resolution pixels. Moved AI points remeasure
/// the foot; how far every point moved from the app's guess is recorded.
#[tauri::command]
async fn confirm_points(app: tauri::AppHandle, key: String, side: String, heel: [f32; 2], toe: [f32; 2], clarke: measure::ClarkePts) -> Result<PatientInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        let p = session::read_patient(&dir)?;
        let e = p.scans.iter().find(|e| e.side == side).ok_or("scan not found")?;
        let m = e.measurements.clone().ok_or("NO_FOOT: this scan has no measurements")?;
        let sc = current_scale();
        let ai_moved = [dist_mm(heel, m.heel_centre, &sc), dist_mm(toe, m.toe_tip, &sc)];
        if ai_moved.iter().any(|&d| d > 0.5) {
            let m2 = measure::measure_scan(&dir.join(&e.file), &sc, Some((heel, toe)))?;
            session::set_measurements(&dir, &side, Some(m2))?;
        }
        let moved = m.clarke_pts.as_ref().map(|g| [dist_mm(clarke.fore, g.fore, &sc), dist_mm(clarke.heel, g.heel, &sc), dist_mm(clarke.arch, g.arch, &sc)]);
        let check = session::PointCheck {
            status: "diperiksa".into(),
            skip_reason: None,
            operator: operator_now(&dir, &p),
            at: session::now_iso(),
            clarke_deg: Some(measure::clarke_from_points(&clarke, &sc)),
            clarke_pts: Some(clarke),
            clarke_moved_mm: moved,
            ai_moved_mm: Some(ai_moved),
            guess_source: m.clarke_source.clone(),
        };
        session::set_check(&dir, &side, check)?;
        let mut w = write_pictures(&dir, &side, false);
        w.extend(after_save(&app, sesi_of(&dir), Some(&dir)));
        patient_info(&dd, &dir, w)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// "Lewati": the operator skips the check, with one of session::SKIP_REASONS.
#[tauri::command]
async fn skip_points(app: tauri::AppHandle, key: String, side: String, reason: String) -> Result<PatientInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        let p = session::read_patient(&dir)?;
        let m = p.scans.iter().find(|e| e.side == side).and_then(|e| e.measurements.clone());
        let check = session::PointCheck {
            status: "dilewati".into(),
            skip_reason: Some(reason),
            operator: operator_now(&dir, &p),
            at: session::now_iso(),
            clarke_pts: None,
            clarke_deg: None,
            clarke_moved_mm: None,
            ai_moved_mm: None,
            guess_source: m.and_then(|m| m.clarke_source),
        };
        session::set_check(&dir, &side, check)?;
        let mut w = write_pictures(&dir, &side, false);
        w.extend(after_save(&app, sesi_of(&dir), Some(&dir)));
        patient_info(&dd, &dir, w)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
struct UncheckedFoot {
    key: String,
    no: u32,
    nama: String,
    side: String,
    /// "dilewati" (see skip_reason) or "belum" (never checked)
    status: String,
    skip_reason: Option<String>,
}

/// Feet in the open session whose points nobody has confirmed: for the reminder on "Akhiri sesi"
/// and the "Belum diperiksa" filter.
#[tauri::command]
fn unchecked_feet(app: tauri::AppHandle) -> Result<Vec<UncheckedFoot>, String> {
    let (dd, d, _) = open_sesi_dir(&app)?;
    let mut out = vec![];
    for (pd, p) in session::list_patients(&d) {
        if p.cancelled {
            continue;
        }
        for side in session::unchecked_sides(&p) {
            let c = p.scans.iter().find(|e| e.side == side).and_then(|e| e.check.clone());
            out.push(UncheckedFoot {
                key: session::relative(&dd, &pd),
                no: p.no,
                nama: p.nama.clone(),
                side,
                status: c.as_ref().map(|c| c.status.clone()).unwrap_or_else(|| "belum".into()),
                skip_reason: c.and_then(|c| c.skip_reason),
            });
        }
    }
    Ok(out)
}

/// `by` = who completes it later from Detail pengukuran (recorded with the old answers); None
/// during the patient's turn.
#[tauri::command]
async fn save_inlow(app: tauri::AppHandle, key: String, inlow: inlow::Inlow, by: Option<String>) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        session::set_inlow(&dir, inlow, by.as_deref())?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
}

#[tauri::command]
async fn save_notes(app: tauri::AppHandle, key: String, notes: session::Notes, by: Option<String>) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        session::set_notes(&dir, &notes, by.as_deref())?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
}

#[tauri::command]
async fn save_marks(app: tauri::AppHandle, key: String, marks: Vec<session::Mark>) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        session::set_marks(&dir, marks)?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
}

/// WhatsApp number and consent, changed later from Detail pengukuran.
#[tauri::command]
async fn save_contact(app: tauri::AppHandle, key: String, wa: String, setuju: bool, by: String) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        session::set_contact(&dir, &wa, setuju, &by)?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
}

/// "Buka ringkasan": make the result sheet if needed and open it in the PDF viewer.
#[tauri::command]
async fn open_sheet(app: tauri::AppHandle, key: String) -> Result<String, String> {
    blocking(move || {
        let (_, dir) = patient_dir(&app, &key)?;
        if let Some(w) = pdf::ensure_sheet(&dir, &settings(&app)?) {
            return Err(w);
        }
        let f = pdf::find_sheet(&dir).ok_or("NO_SHEET: the patient is not finished yet")?;
        tauri_plugin_opener::open_path(&f, None::<&str>).map_err(|e| e.to_string())?;
        Ok(f.display().to_string())
    })
    .await
}

/// Is WhatsApp Desktop installed (it registers the whatsapp: link type)?
fn has_whatsapp_desktop() -> bool {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("reg")
        .args(["query", r"HKEY_CLASSES_ROOT\whatsapp"])
        .creation_flags(0x0800_0000)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[derive(Serialize)]
struct WaOpened {
    /// "desktop" (WhatsApp app) or "web" (web.whatsapp.com in the browser)
    via: &'static str,
    /// The sheet was put on the clipboard: Ctrl+V in the chat attaches it.
    copied: bool,
    sheet: String,
}

/// "Kirim lewat WhatsApp": make the sheet if needed, put the file on the clipboard and open the
/// patient's chat with the message typed in. The operator pastes (Ctrl+V), sends, then reports it
/// with mark_sent. Nothing goes out without the operator.
#[tauri::command]
async fn send_whatsapp(app: tauri::AppHandle, key: String) -> Result<WaOpened, String> {
    blocking(move || {
        let s = settings(&app)?;
        let (_, dir) = patient_dir(&app, &key)?;
        let p = session::read_patient(&dir)?;
        if p.wa.is_empty() || !p.wa_setuju {
            return Err("NO_CONSENT: no WhatsApp number or no consent".into());
        }
        if let Some(w) = pdf::ensure_sheet(&dir, &s) {
            return Err(w);
        }
        let f = pdf::find_sheet(&dir).ok_or("NO_SHEET: the patient is not finished yet")?;
        let copied = scanner::powershell(&format!("Set-Clipboard -LiteralPath {}", scanner::ps_quote(&f.to_string_lossy()))).is_ok();
        let text = url_encode(&pdf::wa_message(&p, pdf::sesi_of(&dir).as_ref(), &s));
        let desktop = has_whatsapp_desktop();
        let url = if desktop { format!("whatsapp://send?phone={}&text={text}", p.wa) } else { format!("https://web.whatsapp.com/send?phone={}&text={text}", p.wa) };
        tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| format!("WA_OPEN: {e}"))?;
        Ok(WaOpened { via: if desktop { "desktop" } else { "web" }, copied, sheet: f.file_name().unwrap_or_default().to_string_lossy().to_string() })
    })
    .await
}

/// "Sudah terkirim": the operator reports the sheet went out (`by` = operator).
#[tauri::command]
async fn mark_sent(app: tauri::AppHandle, key: String, by: String) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        let p = session::read_patient(&dir)?;
        session::mark_sent(&dir, &by, &pdf::fingerprint_of(&dir, &p))?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
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
async fn finish_patient(app: tauri::AppHandle, key: String) -> Result<PatientInfo, String> {
    blocking(move || {
        let (dd, dir) = patient_dir(&app, &key)?;
        session::complete(&dir)?;
        let w = after_save(&app, sesi_of(&dir), Some(&dir));
        patient_info(&dd, &dir, w)
    })
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            pick_folder,
            open_guide,
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
            confirm_points,
            skip_points,
            unchecked_feet,
            save_inlow,
            save_notes,
            save_marks,
            save_contact,
            open_sheet,
            send_whatsapp,
            mark_sent,
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
        assert!(rotates("right", "berdiri") && !rotates("left", "berdiri"));
        assert!(!rotates("right", "duduk") && !rotates("left", "duduk"), "a seated patient does not turn around");
        assert!(rotates("right", ""), "older records without a posture count as standing");
    }
}
