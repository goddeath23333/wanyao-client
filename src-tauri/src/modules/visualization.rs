use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChannel {
    pub id: String,
    pub name: String,
    pub color: String,
    pub enabled: bool,
    pub point_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: f64,
    pub value: f64,
    pub channel_id: String,
}

const MAX_POINTS_PER_CHANNEL: usize = 10000;

struct ChannelData {
    channel: DataChannel,
    points: Vec<DataPoint>,
}

static CHANNELS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, ChannelData>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

fn now_epoch_ms() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs_f64() * 1000.0
}

#[tauri::command]
pub fn create_channel(name: String, color: String) -> Result<DataChannel, String> {
    let id = Uuid::new_v4().to_string();
    let channel = DataChannel {
        id: id.clone(),
        name,
        color,
        enabled: true,
        point_count: 0,
    };

    let channel_data = ChannelData {
        channel: channel.clone(),
        points: Vec::new(),
    };

    CHANNELS.lock().unwrap().insert(id, channel_data);

    Ok(channel)
}

#[tauri::command]
pub fn add_data_point(channel_id: String, value: f64) -> Result<DataPoint, String> {
    let mut channels = CHANNELS.lock().unwrap();
    let cd = channels
        .get_mut(&channel_id)
        .ok_or("通道不存在".to_string())?;

    let timestamp = now_epoch_ms();
    let point = DataPoint {
        timestamp,
        value,
        channel_id: channel_id.clone(),
    };

    if cd.points.len() >= MAX_POINTS_PER_CHANNEL {
        cd.points.drain(0..cd.points.len() - MAX_POINTS_PER_CHANNEL + 1);
    }

    cd.points.push(point.clone());
    cd.channel.point_count = cd.points.len();

    Ok(point)
}

#[tauri::command]
pub fn add_data_points_batch(channel_id: String, values: Vec<f64>) -> Result<Vec<DataPoint>, String> {
    let mut channels = CHANNELS.lock().unwrap();
    let cd = channels
        .get_mut(&channel_id)
        .ok_or("通道不存在".to_string())?;

    let mut points = Vec::with_capacity(values.len());
    let base_time = now_epoch_ms();
    let step = 10.0;

    for (i, value) in values.iter().enumerate() {
        let timestamp = base_time - (values.len() - 1 - i) as f64 * step;
        let point = DataPoint {
            timestamp,
            value: *value,
            channel_id: channel_id.clone(),
        };
        points.push(point.clone());
        cd.points.push(point);
    }

    if cd.points.len() > MAX_POINTS_PER_CHANNEL {
        cd.points.drain(0..cd.points.len() - MAX_POINTS_PER_CHANNEL);
    }

    cd.channel.point_count = cd.points.len();

    Ok(points)
}

#[tauri::command]
pub fn get_channel_data(
    channel_id: String,
    start_time: f64,
    end_time: f64,
) -> Result<Vec<DataPoint>, String> {
    let channels = CHANNELS.lock().unwrap();
    let cd = channels
        .get(&channel_id)
        .ok_or("通道不存在".to_string())?;

    let filtered: Vec<DataPoint> = cd
        .points
        .iter()
        .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
        .cloned()
        .collect();

    Ok(filtered)
}

#[tauri::command]
pub fn get_latest_channel_data(channel_id: String, count: usize) -> Result<Vec<DataPoint>, String> {
    let channels = CHANNELS.lock().unwrap();
    let cd = channels
        .get(&channel_id)
        .ok_or("通道不存在".to_string())?;

    let start = if cd.points.len() > count {
        cd.points.len() - count
    } else {
        0
    };

    Ok(cd.points[start..].to_vec())
}

#[tauri::command]
pub fn clear_channel_data(channel_id: String) -> Result<String, String> {
    let mut channels = CHANNELS.lock().unwrap();
    let cd = channels
        .get_mut(&channel_id)
        .ok_or("通道不存在".to_string())?;

    cd.points.clear();
    cd.channel.point_count = 0;

    Ok("数据已清空".to_string())
}

#[tauri::command]
pub fn export_data(format: String, channel_ids: Vec<String>) -> Result<String, String> {
    let channels = CHANNELS.lock().unwrap();

    match format.as_str() {
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            wtr.write_record(&["channel_id", "channel_name", "timestamp", "value"])
                .map_err(|e| format!("CSV写入失败: {}", e))?;

            for channel_id in &channel_ids {
                if let Some(cd) = channels.get(channel_id) {
                    for point in &cd.points {
                        wtr.write_record(&[
                            &cd.channel.id,
                            &cd.channel.name,
                            &point.timestamp.to_string(),
                            &point.value.to_string(),
                        ])
                        .map_err(|e| format!("CSV写入失败: {}", e))?;
                    }
                }
            }

            let bytes = wtr.into_inner().map_err(|e| format!("CSV序列化失败: {}", e))?;
            Ok(String::from_utf8_lossy(&bytes).to_string())
        }
        "json" => {
            let mut export_data = Vec::new();
            for channel_id in &channel_ids {
                if let Some(cd) = channels.get(channel_id) {
                    export_data.push(serde_json::json!({
                        "channel": cd.channel,
                        "points": cd.points,
                    }));
                }
            }
            serde_json::to_string_pretty(&export_data)
                .map_err(|e| format!("JSON序列化失败: {}", e))
        }
        _ => Err("不支持的导出格式，请使用 csv 或 json".to_string()),
    }
}

#[tauri::command]
pub fn list_channels() -> Result<Vec<DataChannel>, String> {
    let channels = CHANNELS.lock().unwrap();
    Ok(channels.values().map(|cd| cd.channel.clone()).collect())
}

#[tauri::command]
pub fn remove_channel(channel_id: String) -> Result<String, String> {
    CHANNELS
        .lock()
        .unwrap()
        .remove(&channel_id)
        .ok_or("通道不存在".to_string())?;
    Ok("通道已删除".to_string())
}

#[tauri::command]
pub fn update_channel(
    channel_id: String,
    name: Option<String>,
    color: Option<String>,
    enabled: Option<bool>,
) -> Result<DataChannel, String> {
    let mut channels = CHANNELS.lock().unwrap();
    let cd = channels
        .get_mut(&channel_id)
        .ok_or("通道不存在".to_string())?;

    if let Some(n) = name {
        cd.channel.name = n;
    }
    if let Some(c) = color {
        cd.channel.color = c;
    }
    if let Some(e) = enabled {
        cd.channel.enabled = e;
    }

    Ok(cd.channel.clone())
}
