use serde::{Deserialize, Serialize};
use sysinfo::System;
use wmi::{WMIConnection, COMLibrary};
use std::collections::HashMap;
use log::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
struct SystemInfo {
    cpu_usage: f32,
    cpu_name: String,
    cpu_cores: usize,
    memory_total: u64,
    memory_used: u64,
    memory_usage: f32,
    gpu_name: String,
    gpu_usage: f32,
    os_name: String,
    os_version: String,
}

fn get_cpu_info() -> (String, usize) {
    info!("开始获取CPU信息...");
    
    let com_con = match COMLibrary::new() {
        Ok(com) => com,
        Err(_) => {
            warn!("COM库初始化失败");
            return ("未知CPU".to_string(), 1);
        }
    };

    let wmi_con = match WMIConnection::new(com_con) {
        Ok(wmi) => wmi,
        Err(_) => {
            warn!("WMI连接失败");
            return ("未知CPU".to_string(), 1);
        }
    };

    let cpus: Vec<HashMap<String, wmi::Variant>> = match wmi_con.raw_query("SELECT Name, NumberOfCores FROM Win32_Processor") {
        Ok(cpus) => cpus,
        Err(_) => {
            warn!("WMI查询CPU信息失败");
            return ("未知CPU".to_string(), 1);
        }
    };

    if let Some(cpu) = cpus.first() {
        let name = match cpu.get(&"Name".to_string()) {
            Some(wmi::Variant::String(s)) => s.clone(),
            _ => "未知CPU".to_string(),
        };
        
        let cores = match cpu.get(&"NumberOfCores".to_string()) {
            Some(wmi::Variant::I4(n)) => *n as usize,
            Some(wmi::Variant::UI4(n)) => *n as usize,
            _ => 1,
        };
        
        info!("获取CPU信息成功: {} ({}核心)", name, cores);
        println!("CPU名称: {}", name);
        println!("CPU核心数: {}", cores);
        (name, cores)
    } else {
        warn!("未找到CPU信息");
        ("未知CPU".to_string(), 1)
    }
}

fn get_gpu_info() -> String {
    info!("开始获取GPU信息...");
    
    let com_con = match COMLibrary::new() {
        Ok(com) => com,
        Err(_) => {
            warn!("COM库初始化失败");
            return "集成显卡".to_string();
        }
    };

    let wmi_con = match WMIConnection::new(com_con) {
        Ok(wmi) => wmi,
        Err(_) => {
            warn!("WMI连接失败");
            return "集成显卡".to_string();
        }
    };

    let gpus: Vec<HashMap<String, wmi::Variant>> = match wmi_con.raw_query("SELECT Name FROM Win32_VideoController WHERE Name IS NOT NULL") {
        Ok(gpus) => gpus,
        Err(_) => {
            warn!("WMI查询GPU信息失败");
            return "集成显卡".to_string();
        }
    };

    if let Some(gpu) = gpus.first() {
        match gpu.get(&"Name".to_string()) {
            Some(wmi::Variant::String(name)) => {
                info!("获取GPU信息成功: {}", name);
                println!("GPU名称: {}", name);
                name.clone()
            },
            _ => "集成显卡".to_string(),
        }
    } else {
        warn!("未找到GPU信息");
        "集成显卡".to_string()
    }
}

fn get_memory_info() -> (u64, u64) {
    info!("开始获取内存信息...");
    
    let com_con = match COMLibrary::new() {
        Ok(com) => com,
        Err(_) => {
            warn!("COM库初始化失败");
            return (0, 0);
        }
    };

    let wmi_con = match WMIConnection::new(com_con) {
        Ok(wmi) => wmi,
        Err(_) => {
            warn!("WMI连接失败");
            return (0, 0);
        }
    };

    let memory: Vec<HashMap<String, wmi::Variant>> = match wmi_con.raw_query("SELECT TotalPhysicalMemory, FreePhysicalMemory FROM Win32_ComputerSystem") {
        Ok(memory) => memory,
        Err(_) => {
            warn!("WMI查询内存信息失败");
            return (0, 0);
        }
    };

    if let Some(mem) = memory.first() {
        let total = match mem.get(&"TotalPhysicalMemory".to_string()) {
            Some(wmi::Variant::String(s)) => s.parse::<u64>().unwrap_or(0),
            _ => 0,
        };
        
        let free_kb = match mem.get(&"FreePhysicalMemory".to_string()) {
            Some(wmi::Variant::String(s)) => s.parse::<u64>().unwrap_or(0),
            Some(wmi::Variant::UI4(n)) => *n as u64,
            _ => 0,
        };
        
        let free_bytes = free_kb * 1024;
        let used = total.saturating_sub(free_bytes);
        
        info!("获取内存信息成功: 总内存={}MB, 已用={}MB", total / 1024 / 1024, used / 1024 / 1024);
        println!("总内存: {}MB, 已用内存: {}MB", total / 1024 / 1024, used / 1024 / 1024);
        (total, used)
    } else {
        warn!("未找到内存信息");
        (0, 0)
    }
}

fn get_os_info() -> (String, String) {
    info!("开始获取操作系统信息...");
    
    let com_con = match COMLibrary::new() {
        Ok(com) => com,
        Err(_) => {
            warn!("COM库初始化失败");
            return ("未知系统".to_string(), "未知版本".to_string());
        }
    };

    let wmi_con = match WMIConnection::new(com_con) {
        Ok(wmi) => wmi,
        Err(_) => {
            warn!("WMI连接失败");
            return ("未知系统".to_string(), "未知版本".to_string());
        }
    };

    let os: Vec<HashMap<String, wmi::Variant>> = match wmi_con.raw_query("SELECT Caption, Version FROM Win32_OperatingSystem") {
        Ok(os) => os,
        Err(_) => {
            warn!("WMI查询操作系统信息失败");
            return ("未知系统".to_string(), "未知版本".to_string());
        }
    };

    if let Some(os_info) = os.first() {
        let name = match os_info.get(&"Caption".to_string()) {
            Some(wmi::Variant::String(s)) => s.clone(),
            _ => "未知系统".to_string(),
        };
        
        let version = match os_info.get(&"Version".to_string()) {
            Some(wmi::Variant::String(s)) => s.clone(),
            _ => "未知版本".to_string(),
        };
        
        info!("获取操作系统信息成功: {} {}", name, version);
        (name, version)
    } else {
        warn!("未找到操作系统信息");
        ("未知系统".to_string(), "未知版本".to_string())
    }
}

#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String> {
    info!("开始获取系统信息...");
    println!("=== 开始执行get_system_info函数 ===");
    
    let mut system = System::new_all();
    println!("1. 创建System对象成功");
    
    system.refresh_all();
    println!("2. 刷新系统信息完成");
    
    let cpu_usage = system.global_cpu_usage();
    println!("3. 获取CPU使用率: {}%", cpu_usage);
    
    let (cpu_name, cpu_cores) = get_cpu_info();
    println!("4. 获取CPU信息完成: {} ({}核心)", cpu_name, cpu_cores);
    
    let (memory_total, memory_used) = get_memory_info();
    println!("5. 获取内存信息完成: 总内存={}MB, 已用={}MB", memory_total / 1024 / 1024, memory_used / 1024 / 1024);
    
    let memory_usage = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        let total = system.total_memory();
        let used = system.used_memory();
        println!("6. 使用sysinfo获取内存信息: 总内存={}MB, 已用={}MB", total / 1024 / 1024, used / 1024 / 1024);
        (used as f32 / total as f32) * 100.0
    };
    
    println!("7. 计算内存使用率: {}%", memory_usage);
    
    let gpu_name = get_gpu_info();
    println!("8. 获取GPU信息完成: {}", gpu_name);
    let gpu_usage = 0.0;
    
    let (os_name, os_version) = get_os_info();
    println!("9. 获取操作系统信息完成: {} {}", os_name, os_version);
    
    let final_memory_total = if memory_total > 0 { memory_total } else { system.total_memory() };
    let final_memory_used = if memory_used > 0 { memory_used } else { system.used_memory() };
    
    println!("10. 最终内存信息: 总内存={}MB, 已用={}MB", final_memory_total / 1024 / 1024, final_memory_used / 1024 / 1024);
    
    let result = SystemInfo {
        cpu_usage,
        cpu_name,
        cpu_cores,
        memory_total: final_memory_total,
        memory_used: final_memory_used,
        memory_usage,
        gpu_name,
        gpu_usage,
        os_name,
        os_version,
    };
    
    println!("11. 构建SystemInfo对象成功");
    println!("12. 准备返回结果给前端");
    
    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_system_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}