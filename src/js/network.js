var networkIsTauri = false;
var networkInvoke = null;
var isNetworkConnected = false;
var networkConnectionId = null;
var networkReadInterval = null;
var totalNetworkReceivedBytes = 0;
var totalNetworkSentBytes = 0;

async function initNetworkAssistant() {
    networkInvoke = window.getInvoke();
    networkIsTauri = window.getIsTauriEnvironment();

    if (networkIsTauri && networkInvoke) {
        console.log('网络助手初始化完成 (Tauri 模式)');
    } else {
        console.log('网络助手初始化完成 (Web 模拟模式)');
    }

    onProtocolChange();
}

function onProtocolChange() {
    var protocol = document.getElementById('networkProtocol').value;
    var remoteHostItem = document.getElementById('remoteHostItem');
    var remotePortItem = document.getElementById('remotePortItem');
    var localPortItem = document.getElementById('localPortItem');

    if (protocol === 'tcp-client') {
        remoteHostItem.style.display = 'flex';
        remotePortItem.style.display = 'flex';
        localPortItem.style.display = 'none';
    } else if (protocol === 'tcp-server') {
        remoteHostItem.style.display = 'none';
        remotePortItem.style.display = 'none';
        localPortItem.style.display = 'flex';
    } else if (protocol === 'udp') {
        remoteHostItem.style.display = 'flex';
        remotePortItem.style.display = 'flex';
        localPortItem.style.display = 'flex';
    }
}

async function toggleNetworkConnection() {
    var btn = document.getElementById('networkConnectBtn');
    var status = document.getElementById('networkStatus');
    var protocol = document.getElementById('networkProtocol').value;

    if (isNetworkConnected) {
        await closeNetworkConnection();
        btn.textContent = '连接';
        btn.classList.remove('btn--danger');
        btn.classList.add('btn--primary');
        status.textContent = '未连接';
        status.classList.remove('status-indicator--connected');
        status.classList.add('status-indicator--disconnected');
        isNetworkConnected = false;
        networkConnectionId = null;
    } else {
        if (networkIsTauri && networkInvoke) {
            try {
                var connection;

                if (protocol === 'tcp-client') {
                    var host = document.getElementById('remoteHost').value;
                    var port = parseInt(document.getElementById('remotePort').value);

                    if (!host || !port) {
                        alert('请输入远程主机和端口');
                        return;
                    }

                    connection = await networkInvoke('create_tcp_client', { host: host, port: port });

                } else if (protocol === 'tcp-server') {
                    var port = parseInt(document.getElementById('localPort').value);

                    if (!port) {
                        alert('请输入本地端口');
                        return;
                    }

                    connection = await networkInvoke('create_tcp_server', { port: port });

                } else if (protocol === 'udp') {
                    var localPort = parseInt(document.getElementById('localPort').value);
                    var remoteHost = document.getElementById('remoteHost').value;
                    var remotePort = parseInt(document.getElementById('remotePort').value);

                    if (!localPort) {
                        alert('请输入本地端口');
                        return;
                    }

                    connection = await networkInvoke('create_udp_socket', {
                        localPort: localPort,
                        remoteHost: remoteHost || null,
                        remotePort: remotePort || null
                    });
                }

                networkConnectionId = connection.id;

                btn.textContent = '断开';
                btn.classList.remove('btn--primary');
                btn.classList.add('btn--danger');
                status.textContent = connection.status + ' - ' + connection.host + ':' + connection.port;
                status.classList.remove('status-indicator--disconnected');
                status.classList.add('status-indicator--connected');
                isNetworkConnected = true;

                startNetworkReading();

            } catch (e) {
                alert('连接失败: ' + e);
                console.error('连接失败:', e);
            }
        } else {
            btn.textContent = '断开';
            btn.classList.remove('btn--primary');
            btn.classList.add('btn--danger');
            status.textContent = '已连接 (模拟)';
            status.classList.remove('status-indicator--disconnected');
            status.classList.add('status-indicator--connected');
            isNetworkConnected = true;

            simulateNetworkReceive();
        }
    }
}

async function closeNetworkConnection() {
    if (networkReadInterval) {
        clearInterval(networkReadInterval);
        networkReadInterval = null;
    }

    if (networkIsTauri && networkInvoke && networkConnectionId) {
        try {
            await networkInvoke('close_network_connection', { connectionId: networkConnectionId });
        } catch (e) {
            console.error('关闭连接失败:', e);
        }
    }
}

function startNetworkReading() {
    if (networkReadInterval) {
        clearInterval(networkReadInterval);
    }

    var hexDisplay = document.getElementById('networkHexDisplay').checked;

    networkReadInterval = setInterval(async function() {
        if (!isNetworkConnected || !networkConnectionId) {
            clearInterval(networkReadInterval);
            return;
        }

        try {
            var result = await networkInvoke('receive_network_data', {
                connectionId: networkConnectionId,
                isHex: hexDisplay
            });

            if (result) {
                if (result.direction === 'SYS') {
                    appendNetworkMessage(result);
                    if (result.data.indexOf('连接已关闭') >= 0) {
                        var btn = document.getElementById('networkConnectBtn');
                        var status = document.getElementById('networkStatus');
                        btn.textContent = '连接';
                        btn.classList.remove('btn--danger');
                        btn.classList.add('btn--primary');
                        status.textContent = '连接已断开';
                        status.classList.remove('status-indicator--connected');
                        status.classList.add('status-indicator--disconnected');
                        isNetworkConnected = false;
                        networkConnectionId = null;
                        clearInterval(networkReadInterval);
                    }
                } else {
                    appendNetworkMessage(result);
                }
            }
        } catch (e) {
            console.error('读取数据失败:', e);
        }
    }, 50);
}

function simulateNetworkReceive() {
    var messages = [
        'OK\r\n',
        'WIFI CONNECTED\r\n',
        'WIFI GOT IP\r\n',
        '+IPD,5:Hello\r\n',
        'ready\r\n'
    ];

    networkReadInterval = setInterval(function() {
        if (!isNetworkConnected) {
            clearInterval(networkReadInterval);
            return;
        }

        if (Math.random() > 0.7) {
            var msg = messages[Math.floor(Math.random() * messages.length)];
            var hexDisplay = document.getElementById('networkHexDisplay').checked;

            var timestamp = new Date().toLocaleTimeString('zh-CN', {
                hour12: false,
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                fractionalSecondDigits: 3
            });

            var message = {
                timestamp: timestamp,
                data: hexDisplay ? stringToHexNetwork(msg) : msg,
                direction: 'RX',
                hex: hexDisplay
            };

            appendNetworkMessage(message);
            totalNetworkReceivedBytes += msg.length;
            updateNetworkReceiveCount();
        }
    }, 1500);
}

function stringToHexNetwork(str) {
    return Array.from(new TextEncoder().encode(str))
        .map(function(b) { return b.toString(16).toUpperCase().padStart(2, '0'); })
        .join(' ');
}

async function sendNetworkData() {
    var sendArea = document.getElementById('networkSendArea');
    var data = sendArea.value;

    if (!data) {
        return;
    }

    if (!isNetworkConnected) {
        alert('请先建立连接');
        return;
    }

    var hexSend = document.getElementById('networkHexSend').checked;
    var appendNewline = document.getElementById('networkAppendNewlineCheck').checked;

    var sendData = data;
    if (appendNewline && !hexSend) {
        sendData += '\r\n';
    }

    if (networkIsTauri && networkInvoke) {
        try {
            var result = await networkInvoke('send_network_data', {
                connectionId: networkConnectionId,
                data: sendData,
                isHex: hexSend,
                remoteAddr: null
            });

            appendNetworkMessage(result);
            totalNetworkSentBytes += hexSend ? sendData.replace(/\s/g, '').length / 2 : sendData.length;
            updateNetworkSendCount();
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
            data: hexSend ? stringToHexNetwork(sendData) : sendData,
            direction: 'TX',
            hex: hexSend
        };

        appendNetworkMessage(message);
        totalNetworkSentBytes += sendData.length;
        updateNetworkSendCount();
    }
}

function appendNetworkMessage(message) {
    var receiveArea = document.getElementById('networkReceiveArea');
    var showTimestamp = document.getElementById('networkTimestampCheck').checked;

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
        totalNetworkReceivedBytes += message.data.length;
        updateNetworkReceiveCount();
    }
}

function clearNetworkReceiveArea() {
    var receiveArea = document.getElementById('networkReceiveArea');
    receiveArea.innerHTML = '';
    totalNetworkReceivedBytes = 0;
    totalNetworkSentBytes = 0;
    updateNetworkReceiveCount();
    updateNetworkSendCount();
}

function updateNetworkReceiveCount() {
    var el = document.getElementById('networkReceiveCount');
    if (el) {
        el.textContent = '接收: ' + totalNetworkReceivedBytes + ' 字节';
    }
}

function updateNetworkSendCount() {
    var el = document.getElementById('networkSendCount');
    if (el) {
        el.textContent = '发送: ' + totalNetworkSentBytes + ' 字节';
    }
}

function networkQuickSend(text) {
    var sendArea = document.getElementById('networkSendArea');
    sendArea.value = text;
    sendNetworkData();
}

window.initNetworkAssistant = initNetworkAssistant;
window.onProtocolChange = onProtocolChange;
window.toggleNetworkConnection = toggleNetworkConnection;
window.sendNetworkData = sendNetworkData;
window.clearNetworkReceiveArea = clearNetworkReceiveArea;
window.networkQuickSend = networkQuickSend;
