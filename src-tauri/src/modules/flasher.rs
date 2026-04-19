use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipType {
    pub id: String,
    pub name: String,
    pub flash_cmd: String,
    pub verify_cmd: String,
    pub erase_cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub path: String,
    pub file_name: String,
    pub file_size: u64,
    pub chip_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashOutput {
    pub line: String,
    pub timestamp: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashStatus {
    pub running: bool,
    pub exit_code: Option<i32>,
    pub output_count: usize,
}

static CHIP_TYPES: once_cell::sync::Lazy<Vec<ChipType>> = once_cell::sync::Lazy::new(|| {
    vec![
        ChipType {
            id: "esp32".to_string(),
            name: "ESP32 系列".to_string(),
            flash_cmd: "esptool.py --chip esp32 --port {port} --baud {baud} write_flash -z 0x10000 {firmware}".to_string(),
            verify_cmd: "esptool.py --chip esp32 --port {port} verify_flash 0x10000 {firmware}".to_string(),
            erase_cmd: "esptool.py --chip esp32 --port {port} erase_flash".to_string(),
        },
        ChipType {
            id: "esp8266".to_string(),
            name: "ESP8266 系列".to_string(),
            flash_cmd: "esptool.py --chip esp8266 --port {port} --baud {baud} write_flash -z 0x00000 {firmware}".to_string(),
            verify_cmd: "esptool.py --chip esp8266 --port {port} verify_flash 0x00000 {firmware}".to_string(),
            erase_cmd: "esptool.py --chip esp8266 --port {port} erase_flash".to_string(),
        },
        ChipType {
            id: "stm32".to_string(),
            name: "STM32 系列".to_string(),
            flash_cmd: "stm32flash -w {firmware} -v -g 0x0 {port}".to_string(),
            verify_cmd: "stm32flash -v {firmware} {port}".to_string(),
            erase_cmd: "stm32flash -e 0 {port}".to_string(),
        },
        ChipType {
            id: "stc".to_string(),
            name: "STC 系列".to_string(),
            flash_cmd: "stcgal -p {port} {firmware}".to_string(),
            verify_cmd: "".to_string(),
            erase_cmd: "".to_string(),
        },
    ]
});

static FLASH_OUTPUT_BUFFER: once_cell::sync::Lazy<Arc<Mutex<Vec<FlashOutput>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

static FLASH_STATUS: once_cell::sync::Lazy<Arc<Mutex<FlashStatus>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(FlashStatus {
        running: false,
        exit_code: None,
        output_count: 0,
    })));

static FLASH_CANCEL: once_cell::sync::Lazy<Arc<Mutex<Option<mpsc::Sender<()>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

fn now_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

fn push_output(line: &str, is_error: bool) {
    let output = FlashOutput {
        line: line.to_string(),
        timestamp: now_timestamp(),
        is_error,
    };
    let mut buf = FLASH_OUTPUT_BUFFER.lock().unwrap();
    if buf.len() >= 5000 {
        let keep_from = buf.len() - 4999;
        *buf = buf.split_off(keep_from);
    }
    buf.push(output);
    let count = buf.len();
    drop(buf);
    let mut status = FLASH_STATUS.lock().unwrap();
    status.output_count = count;
}

fn build_command(template: &str, port: &str, baud: &str, firmware: &str) -> String {
    template
        .replace("{port}", port)
        .replace("{baud}", baud)
        .replace("{firmware}", firmware)
}

async fn run_flash_command(cmd_str: String) -> Result<(), String> {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err("空命令".to_string());
    }

    let program = parts[0];
    let args: Vec<&str> = parts[1..].to_vec();

    let mut child = Command::new(program)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动命令失败: {} - {}", program, e))?;

    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
    {
        let mut cancel = FLASH_CANCEL.lock().unwrap();
        *cancel = Some(cancel_tx);
    }

    {
        let mut status = FLASH_STATUS.lock().unwrap();
        status.running = true;
        status.exit_code = None;
    }

    FLASH_OUTPUT_BUFFER.lock().unwrap().clear();

    let stdout = child.stdout.take().ok_or("无法获取标准输出")?;
    let stderr = child.stderr.take().ok_or("无法获取标准错误")?;

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            push_output(&line, false);
        }
    });

    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            push_output(&line, true);
        }
    });

    let child_task = tokio::spawn(async move {
        let result = child.wait().await;
        match result {
            Ok(status) => {
                let code = status.code();
                let mut flash_status = FLASH_STATUS.lock().unwrap();
                flash_status.running = false;
                flash_status.exit_code = code;
                if status.success() {
                    push_output(&format!("命令执行成功 (退出码: {})", code.unwrap_or(0)), false);
                } else {
                    push_output(&format!("命令执行失败 (退出码: {})", code.unwrap_or(-1)), true);
                }
            }
            Err(e) => {
                let mut flash_status = FLASH_STATUS.lock().unwrap();
                flash_status.running = false;
                flash_status.exit_code = None;
                push_output(&format!("命令执行异常: {}", e), true);
            }
        }
    });

    let child_handle = child_task;
    tokio::pin!(child_handle);

    tokio::select! {
        _ = cancel_rx.recv() => {
            push_output("烧录已取消", true);
            let mut flash_status = FLASH_STATUS.lock().unwrap();
            flash_status.running = false;
            flash_status.exit_code = None;
            child_handle.abort();
        }
        _ = &mut child_handle => {}
    }

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    Ok(())
}

#[tauri::command]
pub fn list_supported_chips() -> Result<Vec<ChipType>, String> {
    Ok(CHIP_TYPES.clone())
}

#[tauri::command]
pub fn detect_chip(port_name: String) -> Result<String, String> {
    let ports = serialport::available_ports()
        .map_err(|e| format!("扫描串口失败: {}", e))?;

    for port in ports {
        if port.port_name == port_name {
            match port.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => {
                    let vid = usb_info.vid;
                    let pid = usb_info.pid;
                    if vid == 0x10c4 {
                        return Ok("cp210x".to_string());
                    }
                    if vid == 0x1a86 {
                        return Ok("ch340".to_string());
                    }
                    if vid == 0x0403 {
                        return Ok("ftdi".to_string());
                    }
                    if vid == 0x303a {
                        return Ok("esp32".to_string());
                    }
                    return Ok(format!("usb_{:04x}_{:04x}", vid, pid));
                }
                serialport::SerialPortType::PciPort => {
                    return Ok("pci".to_string());
                }
                _ => {
                    return Ok("unknown".to_string());
                }
            }
        }
    }

    Err(format!("串口 {} 未找到", port_name))
}

#[tauri::command]
pub fn load_firmware(path: String) -> Result<FirmwareInfo, String> {
    let metadata = std::fs::metadata(&path).map_err(|e| format!("无法读取固件文件: {}", e))?;

    let file_name = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let extension = std::path::Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let chip_type = match extension.as_str() {
        "bin" => "esp32",
        "hex" => "stm32",
        "ihex" => "stm32",
        _ => "unknown",
    };

    Ok(FirmwareInfo {
        path: path.clone(),
        file_name,
        file_size: metadata.len(),
        chip_type: chip_type.to_string(),
    })
}

#[tauri::command]
pub async fn flash_firmware(
    chip_type: String,
    firmware_path: String,
    port_name: String,
    baud_rate: u32,
) -> Result<String, String> {
    {
        let status = FLASH_STATUS.lock().unwrap();
        if status.running {
            return Err("已有烧录任务正在运行".to_string());
        }
    }

    let chip = CHIP_TYPES
        .iter()
        .find(|c| c.id == chip_type)
        .ok_or(format!("不支持的芯片类型: {}", chip_type))?;

    if chip.flash_cmd.is_empty() {
        return Err("该芯片类型不支持烧录".to_string());
    }

    let cmd_str = build_command(
        &chip.flash_cmd,
        &port_name,
        &baud_rate.to_string(),
        &firmware_path,
    );

    push_output(&format!("执行烧录命令: {}", cmd_str), false);

    run_flash_command(cmd_str).await?;

    Ok("烧录命令已执行".to_string())
}

#[tauri::command]
pub async fn verify_firmware(
    chip_type: String,
    firmware_path: String,
    port_name: String,
) -> Result<String, String> {
    {
        let status = FLASH_STATUS.lock().unwrap();
        if status.running {
            return Err("已有任务正在运行".to_string());
        }
    }

    let chip = CHIP_TYPES
        .iter()
        .find(|c| c.id == chip_type)
        .ok_or(format!("不支持的芯片类型: {}", chip_type))?;

    if chip.verify_cmd.is_empty() {
        return Err("该芯片类型不支持验证".to_string());
    }

    let cmd_str = build_command(&chip.verify_cmd, &port_name, "115200", &firmware_path);

    push_output(&format!("执行验证命令: {}", cmd_str), false);

    run_flash_command(cmd_str).await?;

    Ok("验证命令已执行".to_string())
}

#[tauri::command]
pub async fn erase_chip(chip_type: String, port_name: String) -> Result<String, String> {
    {
        let status = FLASH_STATUS.lock().unwrap();
        if status.running {
            return Err("已有任务正在运行".to_string());
        }
    }

    let chip = CHIP_TYPES
        .iter()
        .find(|c| c.id == chip_type)
        .ok_or(format!("不支持的芯片类型: {}", chip_type))?;

    if chip.erase_cmd.is_empty() {
        return Err("该芯片类型不支持擦除".to_string());
    }

    let cmd_str = build_command(&chip.erase_cmd, &port_name, "115200", "");

    push_output(&format!("执行擦除命令: {}", cmd_str), false);

    run_flash_command(cmd_str).await?;

    Ok("擦除命令已执行".to_string())
}

#[tauri::command]
pub fn get_flash_output() -> Result<(Vec<FlashOutput>, FlashStatus), String> {
    let buf = FLASH_OUTPUT_BUFFER.lock().unwrap();
    let status = FLASH_STATUS.lock().unwrap();
    Ok((buf.clone(), status.clone()))
}

#[tauri::command]
pub fn clear_flash_output() -> Result<(), String> {
    FLASH_OUTPUT_BUFFER.lock().unwrap().clear();
    let mut status = FLASH_STATUS.lock().unwrap();
    status.running = false;
    status.exit_code = None;
    status.output_count = 0;
    Ok(())
}

#[tauri::command]
pub fn cancel_flash() -> Result<(), String> {
    let mut cancel = FLASH_CANCEL.lock().unwrap();
    if let Some(tx) = cancel.take() {
        let _ = tx.blocking_send(());
    }
    Ok(())
}

#[tauri::command]
pub fn list_serial_ports_for_flasher() -> Result<Vec<serde_json::Value>, String> {
    let ports = serialport::available_ports()
        .map_err(|e| format!("扫描串口失败: {}", e))?;

    Ok(ports
        .iter()
        .map(|p| {
            let mut obj = serde_json::Map::new();
            obj.insert("name".to_string(), serde_json::Value::String(p.port_name.clone()));
            match &p.port_type {
                serialport::SerialPortType::UsbPort(usb) => {
                    if let Some(manufacturer) = &usb.manufacturer {
                        obj.insert("manufacturer".to_string(), serde_json::Value::String(manufacturer.clone()));
                    }
                    if let Some(product) = &usb.product {
                        obj.insert("product".to_string(), serde_json::Value::String(product.clone()));
                    }
                }
                _ => {}
            }
            serde_json::Value::Object(obj)
        })
        .collect())
}
