use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub address: String,
    pub connected_at: String,
}

type ConnectionMap = Arc<Mutex<HashMap<String, NetworkConnection>>>;

static CONNECTIONS: once_cell::sync::Lazy<ConnectionMap> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static TCP_CLIENT_WRITERS: once_cell::sync::Lazy<
    Arc<Mutex<HashMap<String, mpsc::Sender<Vec<u8>>>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static TCP_SERVER_CLIENT_WRITERS: once_cell::sync::Lazy<
    Arc<Mutex<HashMap<String, HashMap<String, mpsc::Sender<Vec<u8>>>>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static UDP_WRITERS: once_cell::sync::Lazy<
    Arc<Mutex<HashMap<String, mpsc::Sender<(Vec<u8>, Option<String>)>>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static READ_BUFFERS: once_cell::sync::Lazy<
    Arc<Mutex<HashMap<String, Vec<NetworkMessage>>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static TCP_SERVER_CLIENTS: once_cell::sync::Lazy<
    Arc<Mutex<HashMap<String, Vec<ClientInfo>>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

const MAX_BUFFER_SIZE: usize = 2000;

fn push_to_read_buffer(conn_id: &str, msg: NetworkMessage) {
    let mut buffers = READ_BUFFERS.lock().unwrap();
    if let Some(buf) = buffers.get_mut(conn_id) {
        if buf.len() >= MAX_BUFFER_SIZE {
            buf.drain(0..buf.len() - MAX_BUFFER_SIZE + 1);
        }
        buf.push(msg);
    }
}

fn now_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

#[tauri::command]
pub async fn create_tcp_client(host: String, port: u16) -> Result<NetworkConnection, String> {
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

    let stream = TcpStream::connect((host.as_str(), port))
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let local_port = stream.local_addr().map(|a| a.port()).ok();

    let (write_tx, mut write_rx) = mpsc::channel::<Vec<u8>>(100);

    TCP_CLIENT_WRITERS
        .lock()
        .unwrap()
        .insert(connection_id.clone(), write_tx);

    READ_BUFFERS
        .lock()
        .unwrap()
        .insert(connection_id.clone(), Vec::new());

    let conn_id = connection_id.clone();
    tokio::spawn(async move {
        let (mut reader, mut writer) = stream.into_split();
        let read_conn_id = conn_id.clone();
        let write_conn_id = conn_id.clone();

        let read_task = tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => {
                        push_to_read_buffer(
                            &read_conn_id,
                            NetworkMessage {
                                timestamp: now_timestamp(),
                                data: "[连接已关闭]".to_string(),
                                direction: "SYS".to_string(),
                                hex: false,
                                from_addr: None,
                            },
                        );
                        break;
                    }
                    Ok(n) => {
                        let data = &buf[..n];
                        let hex_data = bytes_to_hex(data);
                        let text_data = String::from_utf8_lossy(data).to_string();
                        push_to_read_buffer(
                            &read_conn_id,
                            NetworkMessage {
                                timestamp: now_timestamp(),
                                data: hex_data,
                                direction: "RX".to_string(),
                                hex: true,
                                from_addr: None,
                            },
                        );
                        push_to_read_buffer(
                            &read_conn_id,
                            NetworkMessage {
                                timestamp: now_timestamp(),
                                data: text_data,
                                direction: "RX".to_string(),
                                hex: false,
                                from_addr: None,
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
        });

        while let Some(data) = write_rx.recv().await {
            if writer.write_all(&data).await.is_err() {
                break;
            }
        }

        read_task.abort();
        TCP_CLIENT_WRITERS.lock().unwrap().remove(&write_conn_id);
    });

    let mut conn = connection.clone();
    conn.status = "Connected".to_string();
    conn.local_port = local_port;
    CONNECTIONS
        .lock()
        .unwrap()
        .insert(connection_id, conn.clone());
    Ok(conn)
}

#[tauri::command]
pub async fn create_tcp_server(port: u16) -> Result<NetworkConnection, String> {
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

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| format!("启动服务器失败: {}", e))?;

    TCP_SERVER_CLIENT_WRITERS
        .lock()
        .unwrap()
        .insert(connection_id.clone(), HashMap::new());

    TCP_SERVER_CLIENTS
        .lock()
        .unwrap()
        .insert(connection_id.clone(), Vec::new());

    READ_BUFFERS
        .lock()
        .unwrap()
        .insert(connection_id.clone(), Vec::new());

    let conn_id = connection_id.clone();
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let client_addr = addr.to_string();
                    let connected_at = now_timestamp();

                    {
                        let mut clients = TCP_SERVER_CLIENTS.lock().unwrap();
                        if let Some(list) = clients.get_mut(&conn_id) {
                            list.push(ClientInfo {
                                address: client_addr.clone(),
                                connected_at: connected_at.clone(),
                            });
                        }
                    }

                    push_to_read_buffer(
                        &conn_id,
                        NetworkMessage {
                            timestamp: now_timestamp(),
                            data: format!("[客户端已连接: {}]", client_addr),
                            direction: "SYS".to_string(),
                            hex: false,
                            from_addr: Some(client_addr.clone()),
                        },
                    );

                    let (mut reader, mut writer) = stream.into_split();

                    let (client_tx, mut client_rx) = mpsc::channel::<Vec<u8>>(100);

                    {
                        let mut server_clients = TCP_SERVER_CLIENT_WRITERS.lock().unwrap();
                        if let Some(map) = server_clients.get_mut(&conn_id) {
                            map.insert(client_addr.clone(), client_tx);
                        }
                    }

                    let server_id = conn_id.clone();
                    let read_addr = client_addr.clone();
                    let write_addr = client_addr.clone();

                    let read_task = tokio::spawn(async move {
                        let mut buf = [0u8; 4096];
                        loop {
                            match reader.read(&mut buf).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    let data = &buf[..n];
                                    let hex_data = bytes_to_hex(data);
                                    let text_data = String::from_utf8_lossy(data).to_string();
                                    push_to_read_buffer(
                                        &server_id,
                                        NetworkMessage {
                                            timestamp: now_timestamp(),
                                            data: hex_data,
                                            direction: "RX".to_string(),
                                            hex: true,
                                            from_addr: Some(read_addr.clone()),
                                        },
                                    );
                                    push_to_read_buffer(
                                        &server_id,
                                        NetworkMessage {
                                            timestamp: now_timestamp(),
                                            data: text_data,
                                            direction: "RX".to_string(),
                                            hex: false,
                                            from_addr: Some(read_addr.clone()),
                                        },
                                    );
                                }
                                Err(_) => break,
                            }
                        }
                    });

                    let write_server_id = conn_id.clone();
                    tokio::spawn(async move {
                        while let Some(data) = client_rx.recv().await {
                            if writer.write_all(&data).await.is_err() {
                                break;
                            }
                        }
                        read_task.abort();
                        let mut server_clients =
                            TCP_SERVER_CLIENT_WRITERS.lock().unwrap();
                        if let Some(map) = server_clients.get_mut(&write_server_id) {
                            map.remove(&write_addr);
                        }
                    });
                }
                Err(_) => continue,
            }
        }
    });

    let mut conn = connection.clone();
    conn.status = "Listening".to_string();
    CONNECTIONS
        .lock()
        .unwrap()
        .insert(connection_id, conn.clone());
    Ok(conn)
}

#[tauri::command]
pub async fn create_udp_socket(
    local_port: u16,
    remote_host: Option<String>,
    remote_port: Option<u16>,
) -> Result<NetworkConnection, String> {
    let connection_id = Uuid::new_v4().to_string();
    let connection = NetworkConnection {
        id: connection_id.clone(),
        host: remote_host.clone().unwrap_or_default(),
        port: remote_port.unwrap_or(0),
        protocol: "UDP".to_string(),
        mode: if remote_host.is_some() {
            "Client".to_string()
        } else {
            "Server".to_string()
        },
        status: "Creating".to_string(),
        local_port: Some(local_port),
    };

    let socket = UdpSocket::bind(("0.0.0.0", local_port))
        .await
        .map_err(|e| format!("创建UDP套接字失败: {}", e))?;

    if let (Some(host), Some(port)) = (remote_host, remote_port) {
        if let Ok(addr) = SocketAddr::from_str(&format!("{}:{}", host, port)) {
            let _ = socket.connect(addr).await;
        }
    }

    let (write_tx, mut write_rx) = mpsc::channel::<(Vec<u8>, Option<String>)>(100);

    UDP_WRITERS
        .lock()
        .unwrap()
        .insert(connection_id.clone(), write_tx);

    READ_BUFFERS
        .lock()
        .unwrap()
        .insert(connection_id.clone(), Vec::new());

    let conn_id = connection_id.clone();
    let socket = Arc::new(socket);
    let read_socket = socket.clone();
    let write_socket = socket.clone();

    tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match read_socket.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    let data = &buf[..n];
                    let hex_data = bytes_to_hex(data);
                    let text_data = String::from_utf8_lossy(data).to_string();
                    push_to_read_buffer(
                        &conn_id,
                        NetworkMessage {
                            timestamp: now_timestamp(),
                            data: hex_data,
                            direction: "RX".to_string(),
                            hex: true,
                            from_addr: Some(addr.to_string()),
                        },
                    );
                    push_to_read_buffer(
                        &conn_id,
                        NetworkMessage {
                            timestamp: now_timestamp(),
                            data: text_data,
                            direction: "RX".to_string(),
                            hex: false,
                            from_addr: Some(addr.to_string()),
                        },
                    );
                }
                Err(_) => continue,
            }
        }
    });

    let write_conn_id = connection_id.clone();
    tokio::spawn(async move {
        while let Some((data, remote_addr)) = write_rx.recv().await {
            let result: Result<usize, std::io::Error> = if let Some(addr_str) = remote_addr {
                if let Ok(addr) = SocketAddr::from_str(&addr_str) {
                    write_socket.send_to(&data, addr).await
                } else {
                    continue;
                }
            } else {
                write_socket.send(&data).await
            };
            if result.is_err() {
                break;
            }
        }
        UDP_WRITERS.lock().unwrap().remove(&write_conn_id);
    });

    let mut conn = connection.clone();
    conn.status = "Ready".to_string();
    CONNECTIONS
        .lock()
        .unwrap()
        .insert(connection_id, conn.clone());
    Ok(conn)
}

#[tauri::command]
pub async fn send_network_data(
    connection_id: String,
    data: String,
    is_hex: bool,
    remote_addr: Option<String>,
) -> Result<NetworkMessage, String> {
    let conn = {
        let connections = CONNECTIONS.lock().unwrap();
        connections
            .get(&connection_id)
            .ok_or("连接不存在")?
            .clone()
    };

    let bytes = if is_hex {
        hex_to_bytes(&data)?
    } else {
        data.as_bytes().to_vec()
    };

    let timestamp = now_timestamp();
    let hex_data = if is_hex {
        data.clone()
    } else {
        bytes_to_hex(&bytes)
    };

    match conn.protocol.as_str() {
        "TCP" => {
            if conn.mode == "Client" {
                let sender = {
                    let writers = TCP_CLIENT_WRITERS.lock().unwrap();
                    writers.get(&connection_id).cloned()
                };
                if let Some(sender) = sender {
                    sender
                        .send(bytes)
                        .await
                        .map_err(|_| "发送失败: 连接已断开".to_string())?;
                } else {
                    return Err("连接已断开".to_string());
                }
            } else {
                let sender = {
                    let server_clients = TCP_SERVER_CLIENT_WRITERS.lock().unwrap();
                    if let Some(client_map) = server_clients.get(&connection_id) {
                        if let Some(addr) = &remote_addr {
                            client_map.get(addr).cloned()
                        } else {
                            client_map.iter().next().map(|(_, s)| s.clone())
                        }
                    } else {
                        None
                    }
                };
                if let Some(sender) = sender {
                    sender
                        .send(bytes)
                        .await
                        .map_err(|_| "发送失败: 客户端已断开".to_string())?;
                } else if remote_addr.is_some() {
                    return Err(format!("客户端 {} 不存在", remote_addr.unwrap()));
                } else {
                    return Err("没有已连接的客户端".to_string());
                }
            }
        }
        "UDP" => {
            let sender = {
                let writers = UDP_WRITERS.lock().unwrap();
                writers.get(&connection_id).cloned()
            };
            if let Some(sender) = sender {
                sender
                    .send((bytes, remote_addr.clone()))
                    .await
                    .map_err(|_| "发送失败: 套接字已关闭".to_string())?;
            } else {
                return Err("UDP套接字已关闭".to_string());
            }
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
pub fn receive_network_data(
    connection_id: String,
    is_hex: bool,
) -> Result<Option<NetworkMessage>, String> {
    let connections = CONNECTIONS.lock().unwrap();
    if !connections.contains_key(&connection_id) {
        return Err("连接不存在".to_string());
    }
    drop(connections);

    let mut buffers = READ_BUFFERS.lock().unwrap();
    if let Some(buf) = buffers.get_mut(&connection_id) {
        while let Some(msg) = buf.first() {
            if msg.hex != is_hex {
                buf.remove(0);
                continue;
            }
            return Ok(Some(buf.remove(0)));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn close_network_connection(connection_id: String) -> Result<(), String> {
    let mut connections = CONNECTIONS.lock().unwrap();
    if let Some(conn) = connections.remove(&connection_id) {
        match conn.protocol.as_str() {
            "TCP" => {
                if conn.mode == "Client" {
                    let mut writers = TCP_CLIENT_WRITERS.lock().unwrap();
                    if let Some(sender) = writers.remove(&connection_id) {
                        drop(sender);
                    }
                } else {
                    let mut server_clients = TCP_SERVER_CLIENT_WRITERS.lock().unwrap();
                    server_clients.remove(&connection_id);
                    TCP_SERVER_CLIENTS.lock().unwrap().remove(&connection_id);
                }
            }
            "UDP" => {
                let mut writers = UDP_WRITERS.lock().unwrap();
                if let Some(sender) = writers.remove(&connection_id) {
                    drop(sender);
                }
            }
            _ => {}
        }
    }
    READ_BUFFERS.lock().unwrap().remove(&connection_id);
    Ok(())
}

#[tauri::command]
pub fn list_network_connections() -> Result<Vec<NetworkConnection>, String> {
    let connections = CONNECTIONS.lock().unwrap();
    Ok(connections.values().cloned().collect())
}

#[tauri::command]
pub fn list_tcp_server_clients(connection_id: String) -> Result<Vec<ClientInfo>, String> {
    let clients = TCP_SERVER_CLIENTS.lock().unwrap();
    clients
        .get(&connection_id)
        .cloned()
        .ok_or("服务器不存在".to_string())
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    if hex.len() % 2 != 0 {
        return Err("十六进制字符串长度必须为偶数".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("无效的十六进制: {}", e))
        })
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X} ", b))
        .collect::<String>()
        .trim()
        .to_string()
}
