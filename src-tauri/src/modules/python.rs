use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonResult {
    pub success: bool,
    pub result: String,
    pub error: String,
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
pub fn check_python_version() -> Result<String, String> {
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
pub fn run_python_code(code: String) -> Result<PythonResult, String> {
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
pub fn run_python_script(script_path: String, args: Vec<String>) -> Result<PythonResult, String> {
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
