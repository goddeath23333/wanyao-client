let isTauri = false;
let invoke = null;
let isConnected = false;
let readInterval = null;
let totalReceivedBytes = 0;
let totalSentBytes = 0;

async function initSerialAssistant() {
    try {
        const module = await import('@tauri-apps/api/core');
        invoke = module.invoke;
        isTauri = true;
        console.log('串口助手初始化完成 (Tauri 模式)');
    } catch (e) {
        isTauri = false;
        console.log('串口助手初始化完成 (Web 模拟模式)');
    }
    
    await refreshSerialPorts();
}

async function refreshSerialPorts() {
    const select = document.getElementById('serialPortSelect');
    if (!select) return;
    
    select.innerHTML = '<option value="">扫描中...</option>';
    
    if (isTauri && invoke) {
        try {
            const ports = await invoke('list_serial_ports');
            select.innerHTML = '<option value="">选择串口...</option>';
            
            ports.forEach(port => {
                const option = document.createElement('option');
                option.value = port.name;
                let label = port.name;
                if (port.manufacturer) {
                    label += ` (${port.manufacturer})`;
                }
                if (port.product) {
                    label += ` - ${port.product}`;
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
        select.innerHTML = `
            <option value="">选择串口...</option>
            <option value="COM1">COM1 (模拟)</option>
            <option value="COM3">COM3 (模拟)</option>
            <option value="COM5">COM5 (模拟)</option>
        `;
    }
}

async function toggleSerialConnection() {
    const btn = document.getElementById('serialConnectBtn');
    const status = document.getElementById('serialStatus');
    const portSelect = document.getElementById('serialPortSelect');
    
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
        const portName = portSelect.value;
        if (!portName) {
            alert('请选择串口');
            return;
        }
        
        const baudRate = parseInt(document.getElementById('baudRateSelect').value);
        const dataBits = document.getElementById('dataBitsSelect').value;
        const stopBits = document.getElementById('stopBitsSelect').value;
        const parity = document.getElementById('paritySelect').value;
        const flowControl = document.getElementById('flowControlSelect').value;
        
        if (isTauri && invoke) {
            try {
                await invoke('open_serial_port', {
                    portName,
                    baudRate,
                    dataBits,
                    stopBits,
                    parity,
                    flowControl
                });
                
                btn.textContent = '关闭串口';
                btn.classList.remove('btn--primary');
                btn.classList.add('btn--danger');
                status.textContent = `已连接 ${portName}`;
                status.classList.remove('status-indicator--disconnected');
                status.classList.add('status-indicator--connected');
                isConnected = true;
                
                startReading();
            } catch (e) {
                alert(`打开串口失败: ${e}`);
                console.error('打开串口失败:', e);
            }
        } else {
            btn.textContent = '关闭串口';
            btn.classList.remove('btn--primary');
            btn.classList.add('btn--danger');
            status.textContent = `已连接 ${portName} (模拟)`;
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
    
    if (isTauri && invoke) {
        try {
            await invoke('close_serial_port');
        } catch (e) {
            console.error('关闭串口失败:', e);
        }
    }
}

function startReading() {
    if (readInterval) {
        clearInterval(readInterval);
    }
    
    const hexDisplay = document.getElementById('hexDisplayCheck').checked;
    
    readInterval = setInterval(async () => {
        if (!isConnected) {
            clearInterval(readInterval);
            return;
        }
        
        try {
            const result = await invoke('read_serial_data', { isHex: hexDisplay });
            if (result) {
                appendMessage(result);
            }
        } catch (e) {
            console.error('读取数据失败:', e);
        }
    }, 50);
}

function simulateReceive() {
    const messages = [
        'OK\r\n',
        'AT version:1.7.4.0\r\n',
        'SDK version:3.0.4\r\n',
        'ready\r\n',
        'WIFI CONNECTED\r\n',
        'WIFI GOT IP\r\n'
    ];
    
    readInterval = setInterval(() => {
        if (!isConnected) {
            clearInterval(readInterval);
            return;
        }
        
        if (Math.random() > 0.7) {
            const msg = messages[Math.floor(Math.random() * messages.length)];
            const hexDisplay = document.getElementById('hexDisplayCheck').checked;
            const showTimestamp = document.getElementById('timestampCheck').checked;
            
            const timestamp = new Date().toLocaleTimeString('zh-CN', { 
                hour12: false, 
                hour: '2-digit', 
                minute: '2-digit', 
                second: '2-digit',
                fractionalSecondDigits: 3
            });
            
            const message = {
                timestamp,
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
        .map(b => b.toString(16).toUpperCase().padStart(2, '0'))
        .join(' ');
}

async function sendSerialData() {
    const sendArea = document.getElementById('sendArea');
    const data = sendArea.value;
    
    if (!data) {
        return;
    }
    
    if (!isConnected) {
        alert('请先打开串口');
        return;
    }
    
    const hexSend = document.getElementById('hexSendCheck').checked;
    const appendNewline = document.getElementById('appendNewlineCheck').checked;
    
    let sendData = data;
    if (appendNewline && !hexSend) {
        sendData += '\r\n';
    }
    
    if (isTauri && invoke) {
        try {
            const result = await invoke('send_serial_data', { 
                data: sendData, 
                isHex: hexSend 
            });
            
            appendMessage(result);
            totalSentBytes += hexSend ? sendData.replace(/\s/g, '').length / 2 : sendData.length;
            updateSendCount();
        } catch (e) {
            alert(`发送失败: ${e}`);
            console.error('发送失败:', e);
        }
    } else {
        const timestamp = new Date().toLocaleTimeString('zh-CN', { 
            hour12: false, 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit',
            fractionalSecondDigits: 3
        });
        
        const message = {
            timestamp,
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
    const receiveArea = document.getElementById('receiveArea');
    const showTimestamp = document.getElementById('timestampCheck').checked;
    
    const line = document.createElement('div');
    line.className = `message-line message-line--${message.direction.toLowerCase()}`;
    
    let content = '';
    if (showTimestamp) {
        content += `<span class="message-timestamp">[${message.timestamp}]</span>`;
    }
    content += `<span class="message-direction">[${message.direction}]</span>`;
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
    const receiveArea = document.getElementById('receiveArea');
    receiveArea.innerHTML = '';
    totalReceivedBytes = 0;
    totalSentBytes = 0;
    updateReceiveCount();
    updateSendCount();
}

function updateReceiveCount() {
    const el = document.getElementById('receiveCount');
    if (el) {
        el.textContent = `接收: ${totalReceivedBytes} 字节`;
    }
}

function updateSendCount() {
    const el = document.getElementById('sendCount');
    if (el) {
        el.textContent = `发送: ${totalSentBytes} 字节`;
    }
}

function quickSend(text) {
    const sendArea = document.getElementById('sendArea');
    
    if (text === '\\r\\n') {
        sendArea.value += '\r\n';
    } else {
        sendArea.value = text;
        sendSerialData();
    }
}

export {
    initSerialAssistant,
    refreshSerialPorts,
    toggleSerialConnection,
    sendSerialData,
    clearReceiveArea,
    quickSend
};
