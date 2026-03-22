use serde::{Deserialize, Serialize};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;
use std::sync::Mutex;
use serialport::{SerialPort, SerialPortType, DataBits, StopBits, Parity, FlowControl};
use std::time::Duration;
use chrono::Local;
use std::io::{Read, Write};

static SERIAL_PORT: Mutex<Option<Box<dyn SerialPort>>> = Mutex::new(None);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CpuInfo {
    name: String,
    usage: f32,
    cores: usize,
    #[serde(default)]
    physical_cores: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MemoryInfo {
    total: u64,
    used: u64,
    #[serde(default)]
    available: Option<u64>,
    usage: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GpuInfo {
    name: String,
    usage: f32,
    #[serde(default)]
    memory_total: Option<f64>,
    #[serde(default)]
    memory_used: Option<f64>,
    #[serde(default)]
    memory_usage: Option<f32>,
    #[serde(default)]
    temperature: Option<f64>,
    vendor: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OsInfo {
    name: String,
    version: String,
    #[serde(default)]
    release: Option<String>,
    #[serde(default)]
    arch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SystemInfo {
    cpu: CpuInfo,
    memory: MemoryInfo,
    gpus: Vec<GpuInfo>,
    os: OsInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerialPortInfo {
    name: String,
    port_type: String,
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerialMessage {
    timestamp: String,
    data: String,
    direction: String,
    hex: bool,
}

fn get_exe_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
    exe_path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf()
}

fn get_resource_dir() -> PathBuf {
    let exe_dir = get_exe_dir();
    
    let up_dir = exe_dir.join("_up_");
    if up_dir.exists() {
        return up_dir;
    }
    
    exe_dir
}

fn get_embedded_python_path() -> PathBuf {
    let resource_dir = get_resource_dir();
    
    let embed_path = resource_dir.join("python-embed").join("python.exe");
    if embed_path.exists() {
        return embed_path;
    }
    
    let dev_embed_path = resource_dir.join("..").join("..").join("..").join("python-embed").join("python.exe");
    if dev_embed_path.exists() {
        return dev_embed_path;
    }
    
    PathBuf::from("python")
}

fn get_script_path() -> String {
    let resource_dir = get_resource_dir();
    
    let prod_path = resource_dir.join("python-scripts").join("system_monitor.py");
    if prod_path.exists() {
        return prod_path.to_string_lossy().to_string();
    }
    
    let dev_path = resource_dir.join("..").join("..").join("..").join("python-scripts").join("system_monitor.py");
    if dev_path.exists() {
        return dev_path.to_string_lossy().to_string();
    }
    
    "python-scripts/system_monitor.py".to_string()
}

#[tauri::command]
async fn get_system_info() -> Result<SystemInfo, String> {
    let python_path = get_embedded_python_path();
    let script_path = get_script_path();
    let python_dir = python_path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    
    let output = tokio::process::Command::new(&python_path)
        .arg(&script_path)
        .env("PYTHONPATH", &python_dir)
        .env("PYTHONHOME", &python_dir)
        .output()
        .await
        .map_err(|e| format!("执行 Python 脚本失败: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !stderr.is_empty() {
        eprintln!("Python stderr: {}", stderr);
    }
    
    serde_json::from_str::<SystemInfo>(&stdout)
        .map_err(|e| format!("解析系统信息失败: {} - 原始输出: {}", e, stdout))
}

#[derive(Serialize, Deserialize, Debug)]
struct PythonResult {
    success: bool,
    result: String,
    error: String,
}

#[tauri::command]
async fn run_python_code(code: String) -> Result<PythonResult, String> {
    let code_clone = code.clone();
    
    let result = tokio::task::spawn_blocking(move || {
        Python::with_gil(|py| {
            match py.run_bound(&code_clone, None, None) {
                Ok(_) => Ok(PythonResult {
                    success: true,
                    result: "代码执行成功".to_string(),
                    error: String::new(),
                }),
                Err(e) => Ok(PythonResult {
                    success: false,
                    result: String::new(),
                    error: e.to_string(),
                })
            }
        }).map_err(|e: PyErr| format!("Python GIL 错误: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行错误: {}", e))??;
    
    Ok(result)
}

#[tauri::command]
async fn run_python_script(script_path: String, args: Vec<String>) -> Result<PythonResult, String> {
    let result = tokio::task::spawn_blocking(move || {
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            
            let args_json = serde_json::to_string(&args).unwrap_or_else(|_| "[]".to_string());
            if let Err(e) = locals.set_item("args", &args_json) {
                return Ok(PythonResult {
                    success: false,
                    result: String::new(),
                    error: format!("设置参数失败: {}", e),
                });
            }
            
            let code = format!(
                r#"
import sys
sys.argv = ['{}'] + {}
with open('{}', 'r', encoding='utf-8') as f:
    code = f.read()
exec(code)
"#,
                script_path.replace('\\', "\\\\").replace('\'', "\\'"),
                args_json,
                script_path.replace('\\', "\\\\").replace('\'', "\\'")
            );
            
            match py.run_bound(&code, None, Some(&locals)) {
                Ok(_) => Ok(PythonResult {
                    success: true,
                    result: "脚本执行成功".to_string(),
                    error: String::new(),
                }),
                Err(e) => Ok(PythonResult {
                    success: false,
                    result: String::new(),
                    error: e.to_string(),
                })
            }
        }).map_err(|e: PyErr| format!("Python GIL 错误: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行错误: {}", e))??;
    
    Ok(result)
}

#[tauri::command]
async fn run_python_function(module_path: String, function_name: String, args: Vec<String>) -> Result<PythonResult, String> {
    let result = tokio::task::spawn_blocking(move || {
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            
            let args_str = args.iter().map(|a| format!("'{}'", a.replace('\'', "\\'"))).collect::<Vec<_>>().join(", ");
            let code = format!(
                r#"
import sys
import importlib.util
spec = importlib.util.spec_from_file_location("user_module", '{}')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
result = module.{}({})
"#,
                module_path.replace('\\', "\\\\").replace('\'', "\\'"),
                function_name,
                args_str
            );
            
            match py.run_bound(&code, None, Some(&locals)) {
                Ok(_) => {
                    let result_str = match locals.get_item("result") {
                        Ok(Some(r)) => r.extract::<String>().unwrap_or_else(|_| "函数执行成功".to_string()),
                        _ => "函数执行成功".to_string(),
                    };
                    Ok(PythonResult {
                        success: true,
                        result: result_str,
                        error: String::new(),
                    })
                }
                Err(e) => Ok(PythonResult {
                    success: false,
                    result: String::new(),
                    error: e.to_string(),
                })
            }
        }).map_err(|e: PyErr| format!("Python GIL 错误: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行错误: {}", e))??;
    
    Ok(result)
}

#[tauri::command]
async fn check_python_version() -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || {
        Python::with_gil(|py| {
            let version = py.version_info();
            Ok::<String, PyErr>(format!("Python {}.{}.{}", version.major, version.minor, version.patch))
        }).map_err(|e: PyErr| format!("Python GIL 错误: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行错误: {}", e))??;
    
    Ok(result)
}

#[tauri::command]
fn close_app() {
    std::process::exit(0);
}

#[tauri::command]
fn list_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
    let ports = serialport::available_ports()
        .map_err(|e| format!("获取串口列表失败: {}", e))?;
    
    let port_infos: Vec<SerialPortInfo> = ports.into_iter().map(|port| {
        let (port_type, manufacturer, product, serial_number) = match &port.port_type {
            SerialPortType::UsbPort(info) => {
                ("USB".to_string(), 
                 info.manufacturer.clone(), 
                 info.product.clone(), 
                 info.serial_number.clone())
            }
            SerialPortType::BluetoothPort => ("Bluetooth".to_string(), None, None, None),
            SerialPortType::PciPort => ("PCI".to_string(), None, None, None),
            SerialPortType::Unknown => ("Unknown".to_string(), None, None, None),
        };
        
        SerialPortInfo {
            name: port.port_name,
            port_type,
            manufacturer,
            product,
            serial_number,
        }
    }).collect();
    
    Ok(port_infos)
}

#[tauri::command]
fn open_serial_port(port_name: String, baud_rate: u32, data_bits: String, stop_bits: String, parity: String, flow_control: String) -> Result<(), String> {
    let data_bits_val = match data_bits.as_str() {
        "5" => DataBits::Five,
        "6" => DataBits::Six,
        "7" => DataBits::Seven,
        "8" => DataBits::Eight,
        _ => DataBits::Eight,
    };
    
    let stop_bits_val = match stop_bits.as_str() {
        "1" => StopBits::One,
        "2" => StopBits::Two,
        _ => StopBits::One,
    };
    
    let parity_val = match parity.as_str() {
        "None" => Parity::None,
        "Odd" => Parity::Odd,
        "Even" => Parity::Even,
        _ => Parity::None,
    };
    
    let flow_control_val = match flow_control.as_str() {
        "None" => FlowControl::None,
        "Software" => FlowControl::Software,
        "Hardware" => FlowControl::Hardware,
        _ => FlowControl::None,
    };
    
    let port = serialport::new(&port_name, baud_rate)
        .data_bits(data_bits_val)
        .stop_bits(stop_bits_val)
        .parity(parity_val)
        .flow_control(flow_control_val)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| format!("打开串口失败: {}", e))?;
    
    let mut serial_port = SERIAL_PORT.lock()
        .map_err(|_| "无法获取串口锁".to_string())?;
    *serial_port = Some(port);
    
    Ok(())
}

#[tauri::command]
fn close_serial_port() -> Result<(), String> {
    let mut serial_port = SERIAL_PORT.lock()
        .map_err(|_| "无法获取串口锁".to_string())?;
    *serial_port = None;
    Ok(())
}

#[tauri::command]
fn is_serial_port_open() -> Result<bool, String> {
    let serial_port = SERIAL_PORT.lock()
        .map_err(|_| "无法获取串口锁".to_string())?;
    Ok(serial_port.is_some())
}

#[tauri::command]
fn send_serial_data(data: String, is_hex: bool) -> Result<SerialMessage, String> {
    let mut serial_port = SERIAL_PORT.lock()
        .map_err(|_| "无法获取串口锁".to_string())?;
    
    let port = serial_port.as_mut()
        .ok_or("串口未打开".to_string())?;
    
    let bytes = if is_hex {
        let hex_str = data.replace(" ", "").replace("\n", "").replace("\r", "");
        (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i+2], 16)
                .map_err(|e| format!("十六进制解析失败: {}", e)))
            .collect::<Result<Vec<u8>, String>>()?
    } else {
        data.as_bytes().to_vec()
    };
    
    port.write_all(&bytes)
        .map_err(|e| format!("发送数据失败: {}", e))?;
    port.flush()
        .map_err(|e| format!("刷新缓冲区失败: {}", e))?;
    
    Ok(SerialMessage {
        timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
        data: if is_hex { bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ") } else { data },
        direction: "TX".to_string(),
        hex: is_hex,
    })
}

#[tauri::command]
fn read_serial_data(is_hex: bool) -> Result<Option<SerialMessage>, String> {
    let mut serial_port = SERIAL_PORT.lock()
        .map_err(|_| "无法获取串口锁".to_string())?;
    
    let port = serial_port.as_mut()
        .ok_or("串口未打开".to_string())?;
    
    let mut buffer = [0u8; 4096];
    match port.read(&mut buffer) {
        Ok(n) if n > 0 => {
            let data = &buffer[..n];
            let message = SerialMessage {
                timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
                data: if is_hex {
                    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
                } else {
                    String::from_utf8_lossy(data).to_string()
                },
                direction: "RX".to_string(),
                hex: is_hex,
            };
            Ok(Some(message))
        }
        Ok(_) => Ok(None),
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
        Err(e) => Err(format!("读取数据失败: {}", e)),
    }
}

#[tauri::command]
fn set_dtr_rts(dtr: bool, rts: bool) -> Result<(), String> {
    let mut serial_port = SERIAL_PORT.lock()
        .map_err(|_| "无法获取串口锁".to_string())?;
    
    let port = serial_port.as_mut()
        .ok_or("串口未打开".to_string())?;
    
    port.write_data_terminal_ready(dtr)
        .map_err(|e| format!("设置DTR失败: {}", e))?;
    port.write_request_to_send(rts)
        .map_err(|e| format!("设置RTS失败: {}", e))?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            run_python_code,
            run_python_script,
            run_python_function,
            check_python_version,
            close_app,
            list_serial_ports,
            open_serial_port,
            close_serial_port,
            is_serial_port_open,
            send_serial_data,
            read_serial_data,
            set_dtr_rts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
