use std::collections::HashMap;
use wmi::{WMIConnection, COMLibrary};

// 简化的硬件信息获取函数
fn get_cpu_info() -> String {
    match WMIConnection::new(COMLibrary::new().unwrap()) {
        Ok(wmi_con) => {
            let cpus: Vec<HashMap<String, wmi::Variant>> = wmi_con
                .raw_query("SELECT Name FROM Win32_Processor")
                .unwrap_or_default();
            
            if let Some(cpu) = cpus.first() {
                cpu.get("Name")
                    .and_then(|v| match v {
                        wmi::Variant::String(s) => Some(s.clone()),
                        _ => None
                    })
                    .unwrap_or_else(|| "未知CPU".to_string())
            } else {
                "未知CPU".to_string()
            }
        }
        Err(_) => "WMI连接失败".to_string()
    }
}

fn get_memory_info() -> String {
    match WMIConnection::new(COMLibrary::new().unwrap()) {
        Ok(wmi_con) => {
            let memory: Vec<HashMap<String, wmi::Variant>> = wmi_con
                .raw_query("SELECT TotalPhysicalMemory FROM Win32_ComputerSystem")
                .unwrap_or_default();
            
            if let Some(mem) = memory.first() {
                let total_bytes = mem.get("TotalPhysicalMemory")
                    .and_then(|v| match v {
                        wmi::Variant::UI8(n) => Some(*n),
                        wmi::Variant::String(s) => s.parse::<u64>().ok(),
                        _ => None
                    })
                    .unwrap_or(0);
                
                let total_gb = total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                format!("{:.1} GB", total_gb)
            } else {
                "未知内存".to_string()
            }
        }
        Err(_) => "WMI连接失败".to_string()
    }
}

fn main() {
    println!("=== Rust硬件信息测试程序 ===");
    println!();
    
    // 获取并显示CPU信息
    println!("正在获取CPU信息...");
    let cpu_info = get_cpu_info();
    println!("CPU型号: {}", cpu_info);
    println!();
    
    // 获取并显示内存信息
    println!("正在获取内存信息...");
    let memory_info = get_memory_info();
    println!("总内存: {}", memory_info);
    println!();
    
    // 显示系统信息
    println!("系统信息:");
    println!("- 操作系统: Windows");
    println!("- 架构: x64");
    println!();
    
    println!("=== 硬件信息测试完成 ===");
    
    // 等待用户按键退出
    println!("按Enter键退出...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}