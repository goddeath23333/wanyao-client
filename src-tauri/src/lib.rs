use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct SystemInfo {
    message: String,
}

#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String> {
    Ok(SystemInfo {
        message: "系统信息获取功能暂时禁用".to_string(),
    })
}

#[tauri::command]
fn close_app() {
    std::process::exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            close_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
