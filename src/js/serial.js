var serialIsTauri = false;
var serialInvoke = null;
var isConnected = false;
var readInterval = null;
var totalReceivedBytes = 0;
var totalSentBytes = 0;

async function initSerialAssistant() {
    serialInvoke = window.getInvoke();
    serialIsTauri = window.getIsTauriEnvironment();
    
    if (serialIsTauri && serialInvoke) {
        console.log('串口助手初始化完成 (Tauri 模式)');
    } else {
        console.log('串口助手初始化完成 (Web 模拟模式)');
    }
    
    await refreshSerialPorts();
}

async function refreshSerialPorts() {
    var select = document.getElementById('serialPortSelect');
    if (!select) return;
    
    select.innerHTML = '<option value="">扫描中...</option>';
    
    if (serialIsTauri && serialInvoke) {
        try {
            var ports = await serialInvoke('list_serial_ports');
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
            console.error('获取串口列表失败:', e);
        }
    } else {
        select.innerHTML = '<option value="">选择串口...</option>' +
            '<option value="COM1">COM1 (模拟)</option>' +
            '<option value="COM3">COM3 (模拟)</option>' +
            '<option value="COM5">COM5 (模拟)</option>';
    }
}

async function toggleSerialConnection() {
    var btn = document.getElementById('serialConnectBtn');
    var status = document.getElementById('serialStatus');
    var portSelect = document.getElementById('serialPortSelect');
    
    if (isConnected) {
        await closeSerialPort();
        btn.textContent = '打开串口';
        btn.classList.remove('btn--danger');
        btn.classList.add('btn--primary');
        status.textContent = '未连接';
        status.classList.remove('status-indicator--connected');
        status.classList.add('status-indicator--disconnected');
        isConnected = false;
    } else {
        var portName = portSelect.value;
        if (!portName) {
            alert('请选择串口');
            return;
        }
        
        var baudRate = parseInt(document.getElementById('baudRateSelect').value);
        var dataBits = document.getElementById('dataBitsSelect').value;
        var stopBits = document.getElementById('stopBitsSelect').value;
        var parity = document.getElementById('paritySelect').value;
        var flowControl = document.getElementById('flowControlSelect').value;
        
        if (serialIsTauri && serialInvoke) {
            try {
                await serialInvoke('open_serial_port', {
                    portName: portName,
                    baudRate: baudRate,
                    dataBits: dataBits,
                    stopBits: stopBits,
                    parity: parity,
                    flowControl: flowControl
                });
                
                btn.textContent = '关闭串口';
                btn.classList.remove('btn--primary');
                btn.classList.add('btn--danger');
                status.textContent = '已连接 ' + portName;
                status.classList.remove('status-indicator--disconnected');
                status.classList.add('status-indicator--connected');
                isConnected = true;
                
                startReading();
            } catch (e) {
                alert('打开串口失败: ' + e);
                console.error('打开串口失败:', e);
            }
        } else {
            btn.textContent = '关闭串口';
            btn.classList.remove('btn--primary');
            btn.classList.add('btn--danger');
            status.textContent = '已连接 ' + portName + ' (模拟)';
            status.classList.remove('status-indicator--disconnected');
            status.classList.add('status-indicator--connected');
            isConnected = true;
            
            simulateReceive();
        }
    }
}

async function closeSerialPort() {
    if (readInterval) {
        clearInterval(readInterval);
        readInterval = null;
    }
    
    if (serialIsTauri && serialInvoke) {
        try {
            await serialInvoke('close_serial_port');
        } catch (e) {
            console.error('关闭串口失败:', e);
        }
    }
}

function startReading() {
    if (readInterval) {
        clearInterval(readInterval);
    }
    
    var hexDisplay = document.getElementById('hexDisplayCheck').checked;
    
    readInterval = setInterval(async function() {
        if (!isConnected) {
            clearInterval(readInterval);
            return;
        }
        
        try {
            var result = await serialInvoke('read_serial_data', { isHex: hexDisplay });
            if (result) {
                appendMessage(result);
            }
        } catch (e) {
            console.error('读取数据失败:', e);
        }
    }, 50);
}

function simulateReceive() {
    var messages = [
        'OK\r\n',
        'AT version:1.7.4.0\r\n',
        'SDK version:3.0.4\r\n',
        'ready\r\n',
        'WIFI CONNECTED\r\n',
        'WIFI GOT IP\r\n'
    ];
    
    readInterval = setInterval(function() {
        if (!isConnected) {
            clearInterval(readInterval);
            return;
        }
        
        if (Math.random() > 0.7) {
            var msg = messages[Math.floor(Math.random() * messages.length)];
            var hexDisplay = document.getElementById('hexDisplayCheck').checked;
            
            var timestamp = new Date().toLocaleTimeString('zh-CN', { 
                hour12: false, 
                hour: '2-digit', 
                minute: '2-digit', 
                second: '2-digit',
                fractionalSecondDigits: 3
            });
            
            var message = {
                timestamp: timestamp,
                data: hexDisplay ? stringToHex(msg) : msg,
                direction: 'RX',
                hex: hexDisplay
            };
            
            appendMessage(message);
            totalReceivedBytes += msg.length;
            updateReceiveCount();
        }
    }, 1000);
}

function stringToHex(str) {
    return Array.from(new TextEncoder().encode(str))
        .map(function(b) { return b.toString(16).toUpperCase().padStart(2, '0'); })
        .join(' ');
}

async function sendSerialData() {
    var sendArea = document.getElementById('sendArea');
    var data = sendArea.value;
    
    if (!data) {
        return;
    }
    
    if (!isConnected) {
        alert('请先打开串口');
        return;
    }
    
    var hexSend = document.getElementById('hexSendCheck').checked;
    var appendNewline = document.getElementById('appendNewlineCheck').checked;
    
    var sendData = data;
    if (appendNewline && !hexSend) {
        sendData += '\r\n';
    }
    
    if (serialIsTauri && serialInvoke) {
        try {
            var result = await serialInvoke('send_serial_data', { 
                data: sendData, 
                isHex: hexSend 
            });
            
            appendMessage(result);
            totalSentBytes += hexSend ? sendData.replace(/\s/g, '').length / 2 : sendData.length;
            updateSendCount();
        } catch (e) {
            alert('发送失败: ' + e);
            console.error('发送失败:', e);
        }
    } else {
        var timestamp = new Date().toLocaleTimeString('zh-CN', { 
            hour12: false, 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit',
            fractionalSecondDigits: 3
        });
        
        var message = {
            timestamp: timestamp,
            data: hexSend ? stringToHex(sendData) : sendData,
            direction: 'TX',
            hex: hexSend
        };
        
        appendMessage(message);
        totalSentBytes += sendData.length;
        updateSendCount();
    }
}

function appendMessage(message) {
    var receiveArea = document.getElementById('receiveArea');
    var showTimestamp = document.getElementById('timestampCheck').checked;
    
    var line = document.createElement('div');
    line.className = 'message-line message-line--' + message.direction.toLowerCase();
    
    var content = '';
    if (showTimestamp) {
        content += '<span class="message-timestamp">[' + message.timestamp + ']</span>';
    }
    content += '<span class="message-direction">[' + message.direction + ']</span>';
    content += message.data;
    
    line.innerHTML = content;
    receiveArea.appendChild(line);
    receiveArea.scrollTop = receiveArea.scrollHeight;
    
    if (message.direction === 'RX') {
        totalReceivedBytes += message.data.length;
        updateReceiveCount();
    }
}

function clearReceiveArea() {
    var receiveArea = document.getElementById('receiveArea');
    receiveArea.innerHTML = '';
    totalReceivedBytes = 0;
    totalSentBytes = 0;
    updateReceiveCount();
    updateSendCount();
}

function updateReceiveCount() {
    var el = document.getElementById('receiveCount');
    if (el) {
        el.textContent = '接收: ' + totalReceivedBytes + ' 字节';
    }
}

function updateSendCount() {
    var el = document.getElementById('sendCount');
    if (el) {
        el.textContent = '发送: ' + totalSentBytes + ' 字节';
    }
}

function quickSend(text) {
    var sendArea = document.getElementById('sendArea');
    
    if (text === '\\r\\n') {
        sendArea.value += '\r\n';
    } else {
        sendArea.value = text;
        sendSerialData();
    }
}

window.initSerialAssistant = initSerialAssistant;
window.refreshSerialPorts = refreshSerialPorts;
window.toggleSerialConnection = toggleSerialConnection;
window.sendSerialData = sendSerialData;
window.clearReceiveArea = clearReceiveArea;
window.quickSend = quickSend;
