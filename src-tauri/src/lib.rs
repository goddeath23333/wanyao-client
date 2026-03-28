use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Mutex;
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_usage: f32,
    pub gpu_name: String,
    pub gpu_usage: f32,
    pub os_name: String,
    pub os_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonResult {
    pub success: bool,
    pub result: String,
    pub error: String,
}

static SYSTEM: Mutex<Option<System>> = Mutex::new(None);

fn get_system() -> &'static Mutex<Option<System>> {
    &SYSTEM
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String> {
    let mut system_guard = get_system().lock().map_err(|e| e.to_string())?;
    
    if system_guard.is_none() {
        *system_guard = Some(System::new_all());
    }
    
    let system = system_guard.as_mut().ok_or("Failed to initialize system")?;
    system.refresh_all();
    
    let cpu_name = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    
    let cpu_cores = system.cpus().len();
    
    let cpu_usage = system.global_cpu_usage();
    
    let memory_total = system.total_memory();
    let memory_used = system.used_memory();
    let memory_usage = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64 * 100.0) as f32
    } else {
        0.0
    };
    
    let gpu_name = get_gpu_info();
    let gpu_usage = 0.0;
    
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    
    Ok(SystemInfo {
        cpu_name,
        cpu_cores,
        cpu_usage,
        memory_total,
        memory_used,
        memory_usage,
        gpu_name,
        gpu_usage,
        os_name,
        os_version,
    })
}

#[cfg(target_os = "windows")]
fn get_gpu_info() -> String {
    use std::process::Command;
    
    let output = Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "name"])
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            if lines.len() > 1 {
                lines[1].trim().to_string()
            } else {
                "Unknown GPU".to_string()
            }
        }
        Err(_) => "Unknown GPU".to_string(),
    }
}

#[cfg(not(target_os = "windows"))]
fn get_gpu_info() -> String {
    "Unknown GPU".to_string()
}

fn get_embedded_python_path() -> Option<std::path::PathBuf> {
    let exe_dir = std::env::current_exe().ok()?;
    let app_dir = exe_dir.parent()?;
    let python_embed = app_dir.join("python-embed").join("python.exe");
    
    if python_embed.exists() {
        return Some(python_embed);
    }
    
    let current_dir = std::env::current_dir().ok()?;
    let python_embed_current = current_dir.join("python-embed").join("python.exe");
    
    if python_embed_current.exists() {
        return Some(python_embed_current);
    }
    
    None
}

#[tauri::command]
fn check_python_version() -> Result<String, String> {
    let python_path = get_embedded_python_path()
        .ok_or("Embedded Python not found. Please ensure python-embed directory exists.")?;
    
    let output = Command::new(&python_path)
        .args(["--version"])
        .output()
        .map_err(|e| format!("Failed to execute Python: {}", e))?;
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("Python version check failed: {}", stderr))
    }
}

#[tauri::command]
fn run_python_code(code: String) -> Result<PythonResult, String> {
    let python_path = get_embedded_python_path()
        .ok_or("Embedded Python not found. Please ensure python-embed directory exists.")?;
    
    let output = Command::new(&python_path)
        .args(["-c", &code])
        .output()
        .map_err(|e| format!("Failed to execute Python: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    Ok(PythonResult {
        success: output.status.success(),
        result: stdout,
        error: stderr,
    })
}

#[tauri::command]
fn run_python_script(script_path: String, args: Vec<String>) -> Result<PythonResult, String> {
    let python_path = get_embedded_python_path()
        .ok_or("Embedded Python not found. Please ensure python-embed directory exists.")?;
    
    let mut cmd = Command::new(&python_path);
    cmd.arg(&script_path);
    for arg in args {
        cmd.arg(arg);
    }
    
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute Python script: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    Ok(PythonResult {
        success: output.status.success(),
        result: stdout,
        error: stderr,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            check_python_version,
            run_python_code,
            run_python_script
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
