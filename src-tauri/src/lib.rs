pub mod scanner;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn list_scanners() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(scanner::list_scanners).await.map_err(|e| e.to_string())?
}

/// Hardware test scan into D:\SmartFoot_Data\_hwtest\ (the real session flow comes later).
#[tauri::command]
async fn scan_test(device: String) -> Result<scanner::ScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let out = std::path::PathBuf::from(format!(r"D:\SmartFoot_Data\_hwtest\scan_{ts}.png"));
        scanner::scan(&device, 300, &out)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, list_scanners, scan_test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
