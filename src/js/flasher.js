var flasherIsTauri = false;
var flasherInvoke = null;
var flasherOutputInterval = null;
var lastOutputCount = 0;

async function initFlasher() {
    flasherInvoke = window.getInvoke();
    flasherIsTauri = window.getIsTauriEnvironment();

    if (flasherIsTauri && flasherInvoke) {
        console.log('固件烧录模块初始化完成 (Tauri 模式)');
        await loadChipTypes();
        await refreshFlasherPorts();
    } else {
        console.log('固件烧录模块初始化完成 (Web 模拟模式)');
        loadSimulatedChipTypes();
    }
}

async function loadChipTypes() {
    try {
        var chips = await flasherInvoke('list_supported_chips');
        var select = document.getElementById('flasherChipType');
        select.innerHTML = '<option value="">选择芯片类型...</option>';
        chips.forEach(function(chip) {
            var option = document.createElement('option');
            option.value = chip.id;
            option.textContent = chip.name;
            select.appendChild(option);
        });
    } catch (e) {
        console.error('加载芯片类型失败:', e);
    }
}

function loadSimulatedChipTypes() {
    var select = document.getElementById('flasherChipType');
    select.innerHTML = '<option value="">选择芯片类型...</option>' +
        '<option value="esp32">ESP32 系列</option>' +
        '<option value="esp8266">ESP8266 系列</option>' +
        '<option value="stm32">STM32 系列</option>' +
        '<option value="stc">STC 系列</option>';
}

async function refreshFlasherPorts() {
    var select = document.getElementById('flasherPortSelect');
    if (!select) return;

    select.innerHTML = '<option value="">扫描中...</option>';

    if (flasherIsTauri && flasherInvoke) {
        try {
            var ports = await flasherInvoke('list_serial_ports_for_flasher');
            select.innerHTML = '<option value="">选择串口...</option>';
            ports.forEach(function(port) {
                var option = document.createElement('option');
                option.value = port.name;
                var label = port.name;
                if (port.manufacturer) {
                    label += ' (' + port.manufacturer + ')';
                }
                if (port.product) {
                    label += ' - ' + port.product;
                }
                option.textContent = label;
                select.appendChild(option);
            });
            if (ports.length === 0) {
                select.innerHTML = '<option value="">未发现串口</option>';
            }
        } catch (e) {
            select.innerHTML = '<option value="">获取串口失败</option>';
        }
    } else {
        select.innerHTML = '<option value="">选择串口...</option>' +
            '<option value="COM1">COM1 (模拟)</option>' +
            '<option value="COM3">COM3 (模拟)</option>';
    }
}

async function onChipTypeChange() {
    var chipId = document.getElementById('flasherChipType').value;
    var detectBtn = document.getElementById('flasherDetectBtn');
    if (chipId) {
        detectBtn.style.display = 'inline-flex';
    } else {
        detectBtn.style.display = 'none';
    }
}

async function detectChip() {
    var portName = document.getElementById('flasherPortSelect').value;
    if (!portName) {
        alert('请先选择串口');
        return;
    }

    if (flasherIsTauri && flasherInvoke) {
        try {
            var chipType = await flasherInvoke('detect_chip', { portName: portName });
            document.getElementById('flasherDetectedChip').textContent = '检测到芯片: ' + chipType;
            document.getElementById('flasherDetectedChip').style.display = 'block';
        } catch (e) {
            document.getElementById('flasherDetectedChip').textContent = '检测失败: ' + e;
            document.getElementById('flasherDetectedChip').style.display = 'block';
        }
    } else {
        document.getElementById('flasherDetectedChip').textContent = '检测到芯片: esp32 (模拟)';
        document.getElementById('flasherDetectedChip').style.display = 'block';
    }
}

async function loadFirmwareFile() {
    var path = document.getElementById('flasherFirmwarePath').value;
    if (!path) {
        alert('请输入固件文件路径');
        return;
    }

    if (flasherIsTauri && flasherInvoke) {
        try {
            var info = await flasherInvoke('load_firmware', { path: path });
            document.getElementById('flasherFirmwareInfo').innerHTML =
                '文件: ' + info.file_name + '<br>' +
                '大小: ' + formatFlashSize(info.file_size) + '<br>' +
                '推断芯片: ' + info.chip_type;
            document.getElementById('flasherFirmwareInfo').style.display = 'block';
        } catch (e) {
            document.getElementById('flasherFirmwareInfo').textContent = '加载失败: ' + e;
            document.getElementById('flasherFirmwareInfo').style.display = 'block';
        }
    } else {
        document.getElementById('flasherFirmwareInfo').innerHTML =
            '文件: firmware.bin (模拟)<br>大小: 256 KB<br>推断芯片: esp32';
        document.getElementById('flasherFirmwareInfo').style.display = 'block';
    }
}

function formatFlashSize(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

async function startFlash() {
    var chipType = document.getElementById('flasherChipType').value;
    var portName = document.getElementById('flasherPortSelect').value;
    var firmwarePath = document.getElementById('flasherFirmwarePath').value;
    var baudRate = parseInt(document.getElementById('flasherBaudRate').value);

    if (!chipType) {
        alert('请选择芯片类型');
        return;
    }
    if (!portName) {
        alert('请选择串口');
        return;
    }
    if (!firmwarePath) {
        alert('请输入固件文件路径');
        return;
    }

    if (flasherIsTauri && flasherInvoke) {
        try {
            await flasherInvoke('flash_firmware', {
                chipType: chipType,
                firmwarePath: firmwarePath,
                portName: portName,
                baudRate: baudRate
            });
            startFlashOutputPolling();
        } catch (e) {
            alert('烧录失败: ' + e);
        }
    } else {
        appendFlashOutput('[模拟] 开始烧录 ' + chipType + ' 固件到 ' + portName, false);
        appendFlashOutput('[模拟] 烧录完成', false);
    }
}

async function startVerify() {
    var chipType = document.getElementById('flasherChipType').value;
    var portName = document.getElementById('flasherPortSelect').value;
    var firmwarePath = document.getElementById('flasherFirmwarePath').value;

    if (!chipType || !portName || !firmwarePath) {
        alert('请填写完整参数');
        return;
    }

    if (flasherIsTauri && flasherInvoke) {
        try {
            await flasherInvoke('verify_firmware', {
                chipType: chipType,
                firmwarePath: firmwarePath,
                portName: portName
            });
            startFlashOutputPolling();
        } catch (e) {
            alert('验证失败: ' + e);
        }
    } else {
        appendFlashOutput('[模拟] 验证完成', false);
    }
}

async function startErase() {
    var chipType = document.getElementById('flasherChipType').value;
    var portName = document.getElementById('flasherPortSelect').value;

    if (!chipType || !portName) {
        alert('请选择芯片类型和串口');
        return;
    }

    if (!confirm('确定要擦除芯片闪存？此操作不可恢复！')) {
        return;
    }

    if (flasherIsTauri && flasherInvoke) {
        try {
            await flasherInvoke('erase_chip', {
                chipType: chipType,
                portName: portName
            });
            startFlashOutputPolling();
        } catch (e) {
            alert('擦除失败: ' + e);
        }
    } else {
        appendFlashOutput('[模拟] 擦除完成', false);
    }
}

function startFlashOutputPolling() {
    if (flasherOutputInterval) {
        clearInterval(flasherOutputInterval);
    }
    lastOutputCount = 0;

    flasherOutputInterval = setInterval(async function() {
        if (!flasherIsTauri || !flasherInvoke) {
            clearInterval(flasherOutputInterval);
            return;
        }

        try {
            var result = await flasherInvoke('get_flash_output');
            var outputs = result[0];
            var status = result[1];

            for (var i = lastOutputCount; i < outputs.length; i++) {
                appendFlashOutput(outputs[i].line, outputs[i].is_error);
            }
            lastOutputCount = outputs.length;

            if (!status.running) {
                clearInterval(flasherOutputInterval);
                flasherOutputInterval = null;
            }
        } catch (e) {
            console.error('获取输出失败:', e);
        }
    }, 200);
}

function appendFlashOutput(text, isError) {
    var outputArea = document.getElementById('flasherOutputArea');
    var line = document.createElement('div');
    line.className = 'flasher-output-line' + (isError ? ' flasher-output-line--error' : '');
    line.textContent = text;
    outputArea.appendChild(line);
    outputArea.scrollTop = outputArea.scrollHeight;
}

async function cancelFlash() {
    if (flasherIsTauri && flasherInvoke) {
        try {
            await flasherInvoke('cancel_flash');
        } catch (e) {
            console.error('取消失败:', e);
        }
    }
}

function clearFlashOutput() {
    document.getElementById('flasherOutputArea').innerHTML = '';
    lastOutputCount = 0;
    if (flasherIsTauri && flasherInvoke) {
        flasherInvoke('clear_flash_output');
    }
}

window.initFlasher = initFlasher;
window.refreshFlasherPorts = refreshFlasherPorts;
window.onChipTypeChange = onChipTypeChange;
window.detectChip = detectChip;
window.loadFirmwareFile = loadFirmwareFile;
window.startFlash = startFlash;
window.startVerify = startVerify;
window.startErase = startErase;
window.cancelFlash = cancelFlash;
window.clearFlashOutput = clearFlashOutput;
