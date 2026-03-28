use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub chip_type: String,
    pub flash_size: u64,
    pub firmware_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FlashProgress {
    pub current: u64,
    pub total: u64,
    pub status: String,
}

#[tauri::command]
pub fn detect_chip() -> Result<String, String> {
    Err("Flasher module not implemented yet".to_string())
}

#[tauri::command]
pub fn load_firmware(_path: String) -> Result<FirmwareInfo, String> {
    Err("Flasher module not implemented yet".to_string())
}

#[tauri::command]
pub fn flash_firmware(_chip_type: String, _firmware_path: String) -> Result<String, String> {
    Err("Flasher module not implemented yet".to_string())
}

#[tauri::command]
pub fn verify_firmware(_chip_type: String, _firmware_path: String) -> Result<bool, String> {
    Err("Flasher module not implemented yet".to_string())
}

#[tauri::command]
pub fn erase_chip(_chip_type: String) -> Result<String, String> {
    Err("Flasher module not implemented yet".to_string())
}
