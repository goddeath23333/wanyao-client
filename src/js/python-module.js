async function checkPythonVersion() {
    var output = document.getElementById('pythonOutput');
    if (!output) return;
    
    output.textContent = '正在检查 Python 版本...';
    
    try {
        if (!window.getIsTauriEnvironment()) {
            output.textContent = '此功能需要在桌面应用中使用';
            return;
        }
        
        var invoke = window.getInvoke();
        var version = await invoke('check_python_version');
        output.textContent = 'Python 版本: ' + version;
    } catch (error) {
        output.textContent = '检查失败: ' + (error.message || error);
    }
}

async function runPythonTest() {
    var output = document.getElementById('pythonOutput');
    if (!output) return;
    
    output.textContent = '正在执行测试代码...';
    
    try {
        if (!window.getIsTauriEnvironment()) {
            output.textContent = '此功能需要在桌面应用中使用';
            return;
        }
        
        var invoke = window.getInvoke();
        var testCode = 'import sys\n' +
            'import platform\n' +
            'print(f"Python: {sys.version}")\n' +
            'print(f"Platform: {platform.system()}")\n';
        
        var result = await invoke('run_python_code', { code: testCode });
        
        if (result.success) {
            output.textContent = result.result;
        } else {
            output.textContent = '错误: ' + result.error;
        }
    } catch (error) {
        output.textContent = '调用失败: ' + (error.message || error);
    }
}

window.checkPythonVersion = checkPythonVersion;
window.runPythonTest = runPythonTest;
