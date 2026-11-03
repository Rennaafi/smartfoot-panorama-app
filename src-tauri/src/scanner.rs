//! Scanner layer. Talks to Windows WIA through a short PowerShell script, the same route that was
//! tested on 2026-10-29 (see the recovered scripts in the project log). Everything outside this
//! file only sees `list_scanners` and `scan`, so a native WIA backend can replace the PowerShell
//! bridge later without touching the rest of the app.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Standard A4 flatbed size in inches (the LiDE 300 bed is 8.5 x 11.69 in).
const BED_W_IN: f64 = 8.5;
const BED_H_IN: f64 = 11.69;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanResult {
    pub path: PathBuf,
    pub seconds: f64,
    pub width_px: u32,
    pub height_px: u32,
}

fn powershell(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script]);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let out = cmd.output().map_err(|e| format!("cannot start PowerShell: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() {
        Ok(text)
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if text.is_empty() { err } else { text })
    }
}

/// Single quotes doubled so a value can sit inside a PowerShell '...' string.
fn ps_quote(s: &str) -> String {
    s.replace('\'', "''")
}

/// Names of all WIA scanners Windows can see right now (e.g. "Canon LiDE 300 HTTP").
pub fn list_scanners() -> Result<Vec<String>, String> {
    let out = powershell(
        "$dm = New-Object -ComObject WIA.DeviceManager; \
         for ($i = 1; $i -le $dm.DeviceInfos.Count; $i++) { \
           $dm.DeviceInfos.Item($i).Properties.Item('Description').Value }",
    )?;
    Ok(out.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
}

/// Seconds since the Unix epoch of the last successful scan or test (0 = none this run).
static LAST_USED: AtomicU64 = AtomicU64::new(0);

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn mark_used() {
    LAST_USED.store(now_secs(), Ordering::Relaxed);
}

/// What the app shows about the scanner. `chosen` is the device a session would use.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScannerStatus {
    pub devices: Vec<String>,
    pub chosen: Option<String>,
    /// "driverless" (Windows eSCL, preferred), "canon" (Canon IJ driver), "other" or "none"
    pub route: String,
    /// Windows installed the useless "LiDE 300 Null Driver" and nothing usable is visible
    pub null_driver: bool,
}

/// The device to scan on: the preferred one if present, else the LiDE 300 driverless route, else
/// any LiDE 300 entry, else (when nothing says LiDE 300, e.g. "Canon IJ Scanner") the only device.
pub fn pick(all: &[String], preferred: Option<&str>) -> Option<String> {
    if let Some(want) = preferred {
        if all.iter().any(|d| d == want) {
            return Some(want.to_string());
        }
    }
    all.iter()
        .find(|d| d.contains("LiDE 300") && d.contains("HTTP"))
        .or_else(|| all.iter().find(|d| d.contains("LiDE 300")))
        .or_else(|| if all.len() == 1 { all.first() } else { None })
        .cloned()
}

pub fn route_of(device: &str) -> &'static str {
    if device.contains("HTTP") {
        "driverless"
    } else if device.contains("Canon IJ") || device.contains("LiDE") {
        "canon"
    } else {
        "other"
    }
}

/// Cheap check (no hardware access): which scanners does Windows list right now?
/// The slower Null Driver lookup only runs when nothing usable was found.
pub fn status(preferred: Option<&str>) -> Result<ScannerStatus, String> {
    let devices = list_scanners()?;
    let chosen = pick(&devices, preferred);
    let route = chosen.as_deref().map(route_of).unwrap_or("none").to_string();
    let null_driver = chosen.is_none() && has_null_driver();
    Ok(ScannerStatus { devices, chosen, route, null_driver })
}

fn has_null_driver() -> bool {
    powershell("Get-CimInstance Win32_PnPEntity -Filter \"Name like '%Null Driver%'\" | ForEach-Object { $_.Name }")
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false)
}

/// Quick hardware test: scan a 1.5 in strip at 100 dpi. It proves the scanner can scan, catches a
/// closed transport LOCK switch, and warms the scanner up so the first real scan is not slow.
pub fn test_scan(device: &str, tmp_dir: &Path) -> Result<f64, String> {
    let out = tmp_dir.join("smartfoot_scanner_test.png");
    let r = scan_region(device, 100, 1.5, &out);
    let _ = std::fs::remove_file(&out);
    r.map(|r| r.seconds)
}

/// Run `test_scan` only if the scanner has been idle for a while (the LiDE 300 needs about 9.5 s
/// extra for the first scan after ~25 min idle). Returns the seconds the test took, if it ran.
pub fn warm_if_cold(device: &str, tmp_dir: &Path) -> Result<Option<f64>, String> {
    const COLD_AFTER_SECS: u64 = 600;
    let last = LAST_USED.load(Ordering::Relaxed);
    if last != 0 && now_secs().saturating_sub(last) < COLD_AFTER_SECS {
        return Ok(None);
    }
    test_scan(device, tmp_dir).map(Some)
}

/// Scan the full bed in colour at `dpi` and save a PNG at `out`.
pub fn scan(device: &str, dpi: u32, out: &Path) -> Result<ScanResult, String> {
    scan_region(device, dpi, BED_H_IN, out)
}

/// Scan the top `height_in` inches of the bed (full width). The head still has to leave its home
/// position, so a blank result from a short strip means the same thing as from a full scan.
fn scan_region(device: &str, dpi: u32, height_in: f64, out: &Path) -> Result<ScanResult, String> {
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    let w = (BED_W_IN * dpi as f64).round() as u32;
    let h = (height_in * dpi as f64).round() as u32;
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
try {{
  $dm = New-Object -ComObject WIA.DeviceManager
  $k = 0
  for ($i = 1; $i -le $dm.DeviceInfos.Count; $i++) {{
    if ($dm.DeviceInfos.Item($i).Properties.Item('Description').Value -eq '{dev}') {{ $k = $i }}
  }}
  if ($k -eq 0) {{ throw 'scanner not found' }}
  $item = $dm.DeviceInfos.Item($k).Connect().Items.Item(1)
  function P($id) {{ $item.Properties | Where-Object {{ $_.PropertyID -eq $id }} }}
  (P 4103).Value = 3; (P 6147).Value = {dpi}; (P 6148).Value = {dpi}
  (P 6149).Value = 0; (P 6150).Value = 0; (P 6151).Value = {w}; (P 6152).Value = {h}
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $img = $item.Transfer('{{B96B3CAB-0728-11D3-9D7B-0000F81EF32E}}')
  $sw.Stop()
  $bmp = '{out}.tmp.bmp'
  if (Test-Path $bmp) {{ Remove-Item $bmp }}
  $img.SaveFile($bmp)
  $b = [System.Drawing.Bitmap]::FromFile($bmp)
  $b.SetResolution({dpi}, {dpi})
  $b.Save('{out}', [System.Drawing.Imaging.ImageFormat]::Png)
  $b.Dispose(); Remove-Item $bmp
  'OK|' + $sw.Elapsed.TotalSeconds.ToString([Globalization.CultureInfo]::InvariantCulture)
}} catch {{ 'ERR|' + $_.Exception.Message; exit 1 }}"#,
        dev = ps_quote(device),
        out = ps_quote(&out.to_string_lossy()),
        dpi = dpi,
        w = w,
        h = h,
    );
    let text = powershell(&script).map_err(|e| friendly(&e))?;
    let secs = text
        .lines()
        .find_map(|l| l.strip_prefix("OK|"))
        .and_then(|s| s.trim().parse::<f64>().ok())
        .ok_or_else(|| friendly(&text))?;

    let (width_px, height_px) = check_image(out)?;
    mark_used();
    Ok(ScanResult { path: out.to_path_buf(), seconds: secs, width_px, height_px })
}

/// Reject the blank scan produced when the scanner's transport LOCK switch is still closed:
/// the head never moves, so every row is an exact copy of the first.
fn check_image(path: &Path) -> Result<(u32, u32), String> {
    let img = image::open(path).map_err(|e| format!("cannot read scan: {e}"))?.to_rgb8();
    let (w, h) = img.dimensions();
    let row0 = &img.as_raw()[..(w as usize * 3)];
    let identical = (1..h).step_by((h as usize / 60).max(1)).all(|y| {
        let s = y as usize * w as usize * 3;
        &img.as_raw()[s..s + w as usize * 3] == row0
    });
    if identical {
        return Err("LOCKED: every row is identical. Open the scanner's transport LOCK switch and scan again.".into());
    }
    Ok((w, h))
}

/// Turn raw PowerShell/WIA errors into short messages the UI can show.
fn friendly(raw: &str) -> String {
    let r = raw.trim_start_matches("ERR|");
    if r.contains("scanner not found") {
        "NOT_FOUND: scanner not found. Check the USB cable and that the scanner is switched on.".into()
    } else if r.contains("driver threw an exception") {
        // measured 2026-11-03 on the LiDE 300 driverless route: LOCK closed -> this error,
        // LOCK open -> the same scan succeeds. (The Canon IJ route returns a blank image instead.)
        "LOCKED: the scan could not start, most likely because the transport LOCK switch is closed. Open it and try again.".into()
    } else if r.to_lowercase().contains("busy") {
        "BUSY: the scanner is busy. Wait a few seconds and try again.".into()
    } else {
        format!("SCAN_FAILED: {r}")
    }
}
