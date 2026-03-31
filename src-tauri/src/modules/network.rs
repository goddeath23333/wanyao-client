use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use uuid::Uuid;

type ConnectionMap = Arc<Mutex<HashMap<String, NetworkConnection>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub mode: String,
    pub status: String,
    pub local_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub timestamp: String,
    pub data: String,
    pub direction: String,
    pub hex: bool,
    pub from_addr: Option<String>,
}

static CONNECTIONS: once_cell::sync::Lazy<ConnectionMap> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static TCP_CLIENTS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, mpsc::Sender<Vec<u8>>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static TCP_SERVERS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, mpsc::Sender<(Vec<u8>, String)>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static UDP_SOCKETS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, mpsc::Sender<(Vec<u8>, String)>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[tauri::command]
pub fn create_tcp_client(host: String, port: u16) -> Result<NetworkConnection, String> {
    let connection_id = Uuid::new_v4().to_string();
    let connection = NetworkConnection {
        id: connection_id.clone(),
        host: host.clone(),
        port,
        protocol: "TCP".to_string(),
        mode: "Client".to_string(),
        status: "Connecting".to_string(),
        local_port: None,
    };
    
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    
    rt.block_on(async {
        match TcpStream::connect((host.as_str(), port)).await {
            Ok(stream) => {
                let local_port = stream.local_addr().map(|a| a.port()).ok();
                
                let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
                let (shutdown_tx, _shutdown_rx) = mpsc::channel::<()>(1);
                
                TCP_CLIENTS.lock().unwrap().insert(connection_id.clone(), tx);
                
                let conn_id = connection_id.clone();
                tokio::spawn(async move {
                    let (mut reader, mut writer) = stream.into_split();
                    let conn_id_clone = conn_id.clone();
                    
                    let read_task = tokio::spawn(async move {
                        let mut buf = [0u8; 4096];
                        loop {
                            match reader.read(&mut buf).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    let sender = {
                                        let clients = TCP_CLIENTS.lock().unwrap();
                                        clients.get(&conn_id_clone).cloned()
                                    };
                                    if let Some(sender) = sender {
                                        let _ = sender.send(buf[..n].to_vec()).await;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                    
                    while let Some(data) = rx.recv().await {
                        if writer.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                    
                    read_task.abort();
                    let _ = shutdown_tx.send(()).await;
                });
                
                let mut conn = connection.clone();
                conn.status = "Connected".to_string();
                conn.local_port = local_port;
                CONNECTIONS.lock().unwrap().insert(connection_id, conn.clone());
                Ok(conn)
            }
            Err(e) => Err(format!("连接失败: {}", e))
        }
    })
}

#[tauri::command]
pub fn create_tcp_server(port: u16) -> Result<NetworkConnection, String> {
    let connection_id = Uuid::new_v4().to_string();
    let connection = NetworkConnection {
        id: connection_id.clone(),
        host: "0.0.0.0".to_string(),
        port,
        protocol: "TCP".to_string(),
        mode: "Server".to_string(),
        status: "Starting".to_string(),
        local_port: Some(port),
    };
    
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    
    rt.block_on(async {
        match TcpListener::bind(("0.0.0.0", port)).await {
            Ok(listener) => {
                let (tx, _) = mpsc::channel::<(Vec<u8>, String)>(100);
                TCP_SERVERS.lock().unwrap().insert(connection_id.clone(), tx);
                
                let conn_id = connection_id.clone();
                tokio::spawn(async move {
                    loop {
                        match listener.accept().await {
                            Ok((stream, addr)) => {
                                let client_addr = addr.to_string();
                                let (mut reader, _writer) = stream.into_split();
                                
                                let servers = TCP_SERVERS.lock().unwrap();
                                if let Some(sender) = servers.get(&conn_id) {
                                    let sender = sender.clone();
                                    drop(servers);
                                    
                                    tokio::spawn(async move {
                                        let mut buf = [0u8; 4096];
                                        loop {
                                            match reader.read(&mut buf).await {
                                                Ok(0) => break,
                                                Ok(n) => {
                                                    let _ = sender.send((buf[..n].to_vec(), client_addr.clone())).await;
                                                }
                                                Err(_) => break,
                                            }
                                        }
                                    });
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                });
                
                let mut conn = connection.clone();
                conn.status = "Listening".to_string();
                CONNECTIONS.lock().unwrap().insert(connection_id, conn.clone());
                Ok(conn)
            }
            Err(e) => Err(format!("启动服务器失败: {}", e))
        }
    })
}

#[tauri::command]
pub fn create_udp_socket(local_port: u16, remote_host: Option<String>, remote_port: Option<u16>) -> Result<NetworkConnection, String> {
    let connection_id = Uuid::new_v4().to_string();
    let connection = NetworkConnection {
        id: connection_id.clone(),
        host: remote_host.clone().unwrap_or_default(),
        port: remote_port.unwrap_or(0),
        protocol: "UDP".to_string(),
        mode: if remote_host.is_some() { "Client".to_string() } else { "Server".to_string() },
        status: "Creating".to_string(),
        local_port: Some(local_port),
    };
    
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    
    rt.block_on(async {
        match UdpSocket::bind(("0.0.0.0", local_port)).await {
            Ok(socket) => {
                if let (Some(host), Some(port)) = (remote_host, remote_port) {
                    if let Ok(addr) = SocketAddr::from_str(&format!("{}:{}", host, port)) {
                        let _ = socket.connect(addr).await;
                    }
                }
                
                let (tx, _) = mpsc::channel::<(Vec<u8>, String)>(100);
                UDP_SOCKETS.lock().unwrap().insert(connection_id.clone(), tx);
                
                let conn_id = connection_id.clone();
                let sock = socket;
                
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    loop {
                        match sock.recv_from(&mut buf).await {
                            Ok((n, addr)) => {
                                let sender = {
                                    let sockets = UDP_SOCKETS.lock().unwrap();
                                    sockets.get(&conn_id).cloned()
                                };
                                if let Some(sender) = sender {
                                    let _ = sender.send((buf[..n].to_vec(), addr.to_string())).await;
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                });
                
                let mut conn = connection.clone();
                conn.status = "Ready".to_string();
                CONNECTIONS.lock().unwrap().insert(connection_id, conn.clone());
                Ok(conn)
            }
            Err(e) => Err(format!("创建UDP套接字失败: {}", e))
        }
    })
}

#[tauri::command]
pub fn send_network_data(connection_id: String, data: String, is_hex: bool, remote_addr: Option<String>) -> Result<NetworkMessage, String> {
    let connections = CONNECTIONS.lock().unwrap();
    let conn = connections.get(&connection_id)
        .ok_or("连接不存在")?
        .clone();
    drop(connections);
    
    let bytes = if is_hex {
        hex_to_bytes(&data)?
    } else {
        data.as_bytes().to_vec()
    };
    
    let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
    
    let hex_data = if is_hex { data.clone() } else { bytes_to_hex(&bytes) };
    
    match conn.protocol.as_str() {
        "TCP" => {
            if conn.mode == "Client" {
                let clients = TCP_CLIENTS.lock().unwrap();
                if let Some(sender) = clients.get(&connection_id) {
                    let _ = sender.blocking_send(bytes);
                }
            } else {
                return Err("TCP服务器发送需要指定客户端地址".to_string());
            }
        }
        "UDP" => {
            // UDP 发送逻辑待实现
        }
        _ => return Err("未知协议".to_string()),
    }
    
    Ok(NetworkMessage {
        timestamp,
        data: hex_data,
        direction: "TX".to_string(),
        hex: is_hex,
        from_addr: remote_addr,
    })
}

#[tauri::command]
pub fn receive_network_data(connection_id: String, _is_hex: bool) -> Result<Option<NetworkMessage>, String> {
    let connections = CONNECTIONS.lock().unwrap();
    let conn = connections.get(&connection_id)
        .ok_or("连接不存在")?
        .clone();
    drop(connections);
    
    match conn.protocol.as_str() {
        "TCP" => {
            if conn.mode == "Client" {
                let clients = TCP_CLIENTS.lock().unwrap();
                if clients.contains_key(&connection_id) {
                    drop(clients);
                    Ok(None)
                } else {
                    Err("连接已断开".to_string())
                }
            } else {
                Ok(None)
            }
        }
        "UDP" => {
            Ok(None)
        }
        _ => Err("未知协议".to_string())
    }
}

#[tauri::command]
pub fn close_network_connection(connection_id: String) -> Result<(), String> {
    let mut connections = CONNECTIONS.lock().unwrap();
    if let Some(conn) = connections.remove(&connection_id) {
        match conn.protocol.as_str() {
            "TCP" => {
                if conn.mode == "Client" {
                    TCP_CLIENTS.lock().unwrap().remove(&connection_id);
                } else {
                    TCP_SERVERS.lock().unwrap().remove(&connection_id);
                }
            }
            "UDP" => {
                UDP_SOCKETS.lock().unwrap().remove(&connection_id);
            }
            _ => {}
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_network_connections() -> Result<Vec<NetworkConnection>, String> {
    let connections = CONNECTIONS.lock().unwrap();
    Ok(connections.values().cloned().collect())
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    if hex.len() % 2 != 0 {
        return Err("十六进制字符串长度必须为偶数".to_string());
    }
    
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2], 16)
            .map_err(|e| format!("无效的十六进制: {}", e)))
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02X} ", b))
        .collect::<String>()
        .trim()
        .to_string()
}
