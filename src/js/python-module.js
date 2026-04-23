var pythonAvailable = null;

async function checkPythonVersion() {
    var output = document.getElementById('pythonOutput');
    if (!output) return;

    output.textContent = '正在检查 Python 版本...';

    try {
        if (!window.getIsTauriEnvironment()) {
            output.innerHTML = '<span style="color: var(--warning);">⚠️ 此功能需要在桌面应用中使用</span><br><br>' +
                '<span style="color: var(--text-tertiary);">请在 Tauri 桌面模式下使用 Python 功能。</span>';
            pythonAvailable = false;
            return;
        }

        var invoke = window.getInvoke();
        var version = await invoke('check_python_version');
        pythonAvailable = true;
        output.innerHTML = '<span style="color: var(--success);">✅ Python 版本: ' + version + '</span><br><br>' +
            '<span style="color: var(--text-tertiary);">Python 环境已就绪，可以使用 Python 嵌入式功能。</span>';
    } catch (error) {
        pythonAvailable = false;
        var errorMsg = (error.message || error || '').toString();
        var hint = '';

        if (errorMsg.indexOf('python') !== -1 || errorMsg.indexOf('not found') !== -1 || errorMsg.indexOf('No such') !== -1) {
            hint = '<br><br>' +
                '<span style="color: var(--warning);">⚠️ 未检测到 Python 环境</span><br><br>' +
                '<span style="color: var(--text-tertiary);">Python 嵌入式功能需要 Python 3.x 环境。请按以下步骤安装：</span><br>' +
                '<span style="color: var(--text-secondary);">1. 访问 <a href="https://www.python.org/downloads/" target="_blank" style="color: var(--accent-primary);">python.org/downloads</a> 下载 Python</span><br>' +
                '<span style="color: var(--text-secondary);">2. 安装时勾选 "Add Python to PATH"</span><br>' +
                '<span style="color: var(--text-secondary);">3. 安装完成后重启应用</span><br><br>' +
                '<span style="color: var(--text-tertiary);">💡 不安装 Python 不影响其他功能（串口、网络、烧录等）的正常使用。</span>';
        } else {
            hint = '<br><br><span style="color: var(--error);">❌ 检查失败: ' + errorMsg + '</span>';
        }

        output.innerHTML = hint;
    }
}

async function runPythonTest() {
    var output = document.getElementById('pythonOutput');
    if (!output) return;

    if (pythonAvailable === false) {
        output.innerHTML = '<span style="color: var(--warning);">⚠️ Python 环境不可用</span><br><br>' +
            '<span style="color: var(--text-tertiary);">请先安装 Python 3.x 并确保已添加到系统 PATH。</span><br>' +
            '<span style="color: var(--text-tertiary);">访问 <a href="https://www.python.org/downloads/" target="_blank" style="color: var(--accent-primary);">python.org/downloads</a> 下载安装。</span>';
        return;
    }

    output.textContent = '正在执行测试代码...';

    try {
        if (!window.getIsTauriEnvironment()) {
            output.innerHTML = '<span style="color: var(--warning);">⚠️ 此功能需要在桌面应用中使用</span>';
            return;
        }

        var invoke = window.getInvoke();
        var testCode = 'import sys\n' +
            'import platform\n' +
            'print(f"Python: {sys.version}")\n' +
            'print(f"Platform: {platform.system()}")\n';

        var result = await invoke('run_python_code', { code: testCode });

        if (result.success) {
            output.innerHTML = '<span style="color: var(--success);">✅ 测试通过</span><br><br>' +
                '<pre style="color: var(--text-secondary); white-space: pre-wrap;">' + result.result + '</pre>';
        } else {
            pythonAvailable = false;
            output.innerHTML = '<span style="color: var(--error);">❌ 运行失败</span><br><br>' +
                '<pre style="color: var(--text-secondary); white-space: pre-wrap;">' + result.error + '</pre>';
        }
    } catch (error) {
        pythonAvailable = false;
        output.innerHTML = '<span style="color: var(--error);">❌ 调用失败: ' + (error.message || error) + '</span><br><br>' +
            '<span style="color: var(--text-tertiary);">可能原因：Python 未安装或未添加到 PATH。</span>';
    }
}

window.checkPythonVersion = checkPythonVersion;
window.runPythonTest = runPythonTest;
