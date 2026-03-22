import json
import platform
import os

def get_info():
    info = {
        "platform": platform.system(),
        "platform_version": platform.version(),
        "python_version": platform.python_version(),
        "architecture": platform.architecture()[0],
        "processor": platform.processor(),
        "hostname": platform.node(),
        "current_dir": os.getcwd(),
        "environment_vars": dict(os.environ)
    }
    return json.dumps(info, ensure_ascii=False, indent=2)

def main():
    print("=== 系统信息 ===")
    print(get_info())

if __name__ == "__main__":
    main()
