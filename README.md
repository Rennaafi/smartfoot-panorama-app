# SmartFoot

Windows desktop app for the Smart Foot Panorama station: it scans the sole of a seated patient's foot on a Canon CanoScan LiDE 300 flatbed scanner, checks the scan, saves it to a session folder and measures foot length, width, contact area and the Cavanagh arch index.

Status: work in progress (UI P4 research project). Measurements are **uncalibrated** (nominal 300 dpi) until the calibration tool is built on the final casing, and the arch-index thresholds are not validated for a seated posture, so the app shows no flat / normal / high label.

## What it does

- Nurse flow in Bahasa Indonesia or English: start session, scan right and left foot (1 or 3 rounds), review the detected outline, accept or retake.
- Team settings: rounds, data folder, scanner, laptop code (patient IDs look like `SFP-<code>-0001`, unique per laptop).
- Research screen: numbers per scan, with the heel centre and second-toe tip draggable; each hand correction is recorded.
- Per session folder: `session.json`, `scans/`, `rejected/` (retaken scans are kept). One `measurements.csv` for all sessions.

## Layout

| Path | What |
|---|---|
| `src/` | Front end (plain HTML/CSS/JS, no framework) |
| `src-tauri/src/scanner.rs` | Scanner access through Windows WIA (PowerShell bridge), blank-scan (LOCK switch) detection |
| `src-tauri/src/analyze.rs` | Finds the foot in a scan and draws its outline |
| `src-tauri/src/measure.rs` | Measurements, with tests against a synthetic foot of known size |
| `src-tauri/src/session.rs` | Settings, sessions, `measurements.csv` |
| `src-tauri/src/bin/scan_cli.rs` | Hardware test tool (`list`, `scan`, `analyze`, `measure`) |
| `mockups/` | The clickable design mockup |

## Build and run

Needs Rust (MSVC toolchain), Node.js and WebView2 (included in Windows 11).

```
npm install
npm run tauri dev          # development window
npm run tauri build        # release build
cargo test --lib           # in src-tauri/
```

Patient data is stored outside the repository (default `D:\SmartFoot_Data`, changeable in team settings). Never commit scans or session files.
