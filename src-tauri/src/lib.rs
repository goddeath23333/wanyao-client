use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::process::Command;
use std::sync::Mutex;
use sysinfo::System;
use serialport::{SerialPort, SerialPortType, DataBits, StopBits, Parity, FlowControl};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerialPortInfo {
    pub name: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialMessage {
    pub timestamp: String,
    pub data: String,
    pub direction: String,
    pub hex: bool,
}

static SYSTEM: Mutex<Option<System>> = Mutex::new(None);
static SERIAL_PORT: Mutex<Option<Box<dyn SerialPort>>> = Mutex::new(None);

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

#[tauri::command]
fn list_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
    let ports = serialport::available_ports()
        .map_err(|e| format!("Failed to list serial ports: {}", e))?;
    
    let port_infos: Vec<SerialPortInfo> = ports
        .into_iter()
        .map(|port| {
            let (manufacturer, product, serial_number) = match &port.port_type {
                SerialPortType::UsbPort(info) => {
                    (
                        info.manufacturer.clone(),
                        info.product.clone(),
                        info.serial_number.clone(),
                    )
                }
                _ => (None, None, None),
            };
            
            SerialPortInfo {
                name: port.port_name,
                manufacturer,
                product,
                serial_number,
            }
        })
        .collect();
    
    Ok(port_infos)
}

#[tauri::command]
fn open_serial_port(
    port_name: String,
    baud_rate: u32,
    data_bits: String,
    stop_bits: String,
    parity: String,
    flow_control: String,
) -> Result<String, String> {
    let data_bits = match data_bits.as_str() {
        "5" => DataBits::Five,
        "6" => DataBits::Six,
        "7" => DataBits::Seven,
        "8" => DataBits::Eight,
        _ => DataBits::Eight,
    };
    
    let stop_bits = match stop_bits.as_str() {
        "1" => StopBits::One,
        "2" => StopBits::Two,
        _ => StopBits::One,
    };
    
    let parity = match parity.as_str() {
        "None" => Parity::None,
        "Odd" => Parity::Odd,
        "Even" => Parity::Even,
        _ => Parity::None,
    };
    
    let flow_control = match flow_control.as_str() {
        "None" => FlowControl::None,
        "Software" => FlowControl::Software,
        "Hardware" => FlowControl::Hardware,
        _ => FlowControl::None,
    };
    
    let port = serialport::new(&port_name, baud_rate)
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .flow_control(flow_control)
        .timeout(std::time::Duration::from_millis(100))
        .open()
        .map_err(|e| format!("Failed to open serial port: {}", e))?;
    
    let mut serial_guard = SERIAL_PORT.lock().map_err(|e| e.to_string())?;
    *serial_guard = Some(port);
    
    Ok(format!("Serial port {} opened successfully", port_name))
}

#[tauri::command]
fn close_serial_port() -> Result<String, String> {
    let mut serial_guard = SERIAL_PORT.lock().map_err(|e| e.to_string())?;
    if serial_guard.is_some() {
        *serial_guard = None;
        Ok("Serial port closed successfully".to_string())
    } else {
        Ok("No serial port is open".to_string())
    }
}

#[tauri::command]
fn send_serial_data(data: String, is_hex: bool) -> Result<SerialMessage, String> {
    let mut serial_guard = SERIAL_PORT.lock().map_err(|e| e.to_string())?;
    let port = serial_guard.as_mut().ok_or("No serial port is open")?;
    
    let bytes_to_send = if is_hex {
        let hex_str: String = data.chars().filter(|c| !c.is_whitespace()).collect();
        (0..hex_str.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex_str[i..i + 2], 16)
                    .map_err(|e| format!("Invalid hex data: {}", e))
            })
            .collect::<Result<Vec<u8>, String>>()?
    } else {
        data.as_bytes().to_vec()
    };
    
    port.write_all(&bytes_to_send)
        .map_err(|e| format!("Failed to send data: {}", e))?;
    
    let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
    
    Ok(SerialMessage {
        timestamp,
        data: if is_hex {
            bytes_to_send.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::from_utf8_lossy(&bytes_to_send).to_string()
        },
        direction: "TX".to_string(),
        hex: is_hex,
    })
}

#[tauri::command]
fn read_serial_data(is_hex: bool) -> Result<Option<SerialMessage>, String> {
    let mut serial_guard = SERIAL_PORT.lock().map_err(|e| e.to_string())?;
    let port = serial_guard.as_mut().ok_or("No serial port is open")?;
    
    let mut buffer = [0u8; 4096];
    
    match port.read(&mut buffer) {
        Ok(bytes_read) if bytes_read > 0 => {
            let data = &buffer[..bytes_read];
            let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
            
            Ok(Some(SerialMessage {
                timestamp,
                data: if is_hex {
                    data.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    String::from_utf8_lossy(data).to_string()
                },
                direction: "RX".to_string(),
                hex: is_hex,
            }))
        }
        Ok(_) => Ok(None),
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
        Err(e) => Err(format!("Failed to read data: {}", e)),
    }
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
            run_python_script,
            list_serial_ports,
            open_serial_port,
            close_serial_port,
            send_serial_data,
            read_serial_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
