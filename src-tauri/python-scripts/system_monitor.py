import json
import sys
import platform
import subprocess
import shutil

def get_cpu_info():
    try:
        import psutil
        cpu_percent = psutil.cpu_percent(interval=0.1)
        cpu_count = psutil.cpu_count(logical=True)
        cpu_count_physical = psutil.cpu_count(logical=False)
        
        cpu_name = "未知CPU"
        if platform.system() == "Windows":
            import wmi
            try:
                c = wmi.WMI()
                for cpu in c.Win32_Processor():
                    cpu_name = cpu.Name.strip()
                    break
            except:
                cpu_name = platform.processor()
        else:
            cpu_name = platform.processor()
        
        return {
            "name": cpu_name,
            "usage": round(cpu_percent, 1),
            "cores": cpu_count,
            "physical_cores": cpu_count_physical
        }
    except Exception as e:
        return {
            "name": "检测失败",
            "usage": 0.0,
            "cores": 0,
            "physical_cores": 0,
            "error": str(e)
        }

def get_memory_info():
    try:
        import psutil
        mem = psutil.virtual_memory()
        return {
            "total": mem.total,
            "used": mem.used,
            "available": mem.available,
            "usage": round(mem.percent, 1)
        }
    except Exception as e:
        return {
            "total": 0,
            "used": 0,
            "available": 0,
            "usage": 0.0,
            "error": str(e)
        }

def get_nvidia_gpus():
    gpus = []
    try:
        import GPUtil
        gpu_list = GPUtil.getGPUs()
        for gpu in gpu_list:
            gpus.append({
                "name": gpu.name,
                "usage": round(gpu.load * 100, 1),
                "memory_total": gpu.memoryTotal,
                "memory_used": gpu.memoryUsed,
                "memory_usage": round((gpu.memoryUsed / gpu.memoryTotal) * 100, 1) if gpu.memoryTotal > 0 else 0,
                "temperature": gpu.temperature,
                "vendor": "NVIDIA"
            })
    except ImportError:
        pass
    except Exception as e:
        print(f"GPUtil error: {e}", file=sys.stderr)
    return gpus

def get_amd_gpus():
    gpus = []
    try:
        result = subprocess.run(
            ["rocm-smi", "--showuse", "--showmeminfo", "vram", "--json"],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0:
            data = json.loads(result.stdout)
            if "card" in data:
                for card_id, card_info in data["card"].items():
                    name = card_info.get("Card series", card_info.get("Card model", f"AMD GPU {card_id}"))
                    gpu_usage = float(card_info.get("GPU use (%)", "0").replace("%", ""))
                    vram_total = int(card_info.get("VRAM Total Memory (B)", "0"))
                    vram_used = int(card_info.get("VRAM Total Used Memory (B)", "0"))
                    
                    gpus.append({
                        "name": name.strip(),
                        "usage": round(gpu_usage, 1),
                        "memory_total": vram_total,
                        "memory_used": vram_used,
                        "memory_usage": round((vram_used / vram_total) * 100, 1) if vram_total > 0 else 0,
                        "temperature": card_info.get("Temperature (Sensor edge) (C)", "N/A"),
                        "vendor": "AMD"
                    })
    except FileNotFoundError:
        pass
    except subprocess.TimeoutExpired:
        pass
    except Exception as e:
        print(f"rocm-smi error: {e}", file=sys.stderr)
    return gpus

def get_windows_gpus():
    gpus = []
    try:
        result = subprocess.run(
            ["powershell", "-Command", 
             "Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM | ConvertTo-Json"],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            data = json.loads(result.stdout)
            if not isinstance(data, list):
                data = [data]
            
            for gpu_data in data:
                name = gpu_data.get("Name", "未知显卡")
                memory = gpu_data.get("AdapterRAM", 0) or 0
                
                gpus.append({
                    "name": name.strip(),
                    "usage": 0.0,
                    "memory_total": memory,
                    "memory_used": 0,
                    "memory_usage": 0.0,
                    "temperature": "N/A",
                    "vendor": "Unknown"
                })
    except Exception as e:
        print(f"Windows GPU detection error: {e}", file=sys.stderr)
    return gpus

def get_all_gpus():
    all_gpus = []
    
    nvidia_gpus = get_nvidia_gpus()
    all_gpus.extend(nvidia_gpus)
    
    amd_gpus = get_amd_gpus()
    all_gpus.extend(amd_gpus)
    
    if not all_gpus:
        if platform.system() == "Windows":
            all_gpus = get_windows_gpus()
        else:
            all_gpus.append({
                "name": "未检测到显卡",
                "usage": 0.0,
                "memory_total": 0,
                "memory_used": 0,
                "memory_usage": 0.0,
                "temperature": "N/A",
                "vendor": "Unknown"
            })
    
    return all_gpus

def get_os_info():
    return {
        "name": platform.system(),
        "version": platform.version(),
        "release": platform.release(),
        "arch": platform.machine()
    }

def get_system_info():
    cpu = get_cpu_info()
    memory = get_memory_info()
    gpus = get_all_gpus()
    os_info = get_os_info()
    
    return {
        "cpu": cpu,
        "memory": memory,
        "gpus": gpus,
        "os": os_info
    }

if __name__ == "__main__":
    try:
        info = get_system_info()
        print(json.dumps(info, ensure_ascii=False))
    except Exception as e:
        error_result = {
            "error": str(e),
            "cpu": {"name": "错误", "usage": 0, "cores": 0},
            "memory": {"total": 0, "used": 0, "usage": 0},
            "gpus": [{"name": "检测失败", "usage": 0, "vendor": "Unknown"}],
            "os": {"name": platform.system(), "version": "", "release": ""}
        }
        print(json.dumps(error_result, ensure_ascii=False))
        sys.exit(1)
