use serde::{Deserialize, Serialize};
use serialport::{SerialPort, SerialPortType, DataBits, StopBits, Parity, FlowControl};
use std::io::{Read, Write};
use std::sync::Mutex;

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

static SERIAL_PORT: Mutex<Option<Box<dyn SerialPort>>> = Mutex::new(None);

#[tauri::command]
pub fn list_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
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
pub fn open_serial_port(
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
pub fn close_serial_port() -> Result<String, String> {
    let mut serial_guard = SERIAL_PORT.lock().map_err(|e| e.to_string())?;
    if serial_guard.is_some() {
        *serial_guard = None;
        Ok("Serial port closed successfully".to_string())
    } else {
        Ok("No serial port is open".to_string())
    }
}

#[tauri::command]
pub fn send_serial_data(data: String, is_hex: bool) -> Result<SerialMessage, String> {
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
pub fn read_serial_data(is_hex: bool) -> Result<Option<SerialMessage>, String> {
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
