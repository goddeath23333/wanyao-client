var vizIsTauri = false;
var vizInvoke = null;
var vizChart = null;
var vizChannels = [];
var vizUpdateInterval = null;
var vizSimInterval = null;
var vizMaxDisplayPoints = 200;

async function initVisualization() {
    vizInvoke = window.getInvoke();
    vizIsTauri = window.getIsTauriEnvironment();

    if (vizIsTauri && vizInvoke) {
        console.log('数据可视化模块初始化完成 (Tauri 模式)');
    } else {
        console.log('数据可视化模块初始化完成 (Web 模拟模式)');
    }

    initVizChart();
    refreshChannelList();
}

function initVizChart() {
    var ctx = document.getElementById('vizChart');
    if (!ctx) return;

    vizChart = new Chart(ctx.getContext('2d'), {
        type: 'line',
        data: {
            datasets: []
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: false,
            scales: {
                x: {
                    type: 'linear',
                    display: true,
                    title: {
                        display: true,
                        text: '时间 (ms)'
                    },
                    ticks: {
                        callback: function(value) {
                            return (value / 1000).toFixed(1) + 's';
                        }
                    }
                },
                y: {
                    display: true,
                    title: {
                        display: true,
                        text: '数值'
                    }
                }
            },
            plugins: {
                legend: {
                    display: true,
                    position: 'top'
                }
            },
            elements: {
                point: {
                    radius: 0
                },
                line: {
                    borderWidth: 2
                }
            }
        }
    });
}

async function refreshChannelList() {
    if (vizIsTauri && vizInvoke) {
        try {
            vizChannels = await vizInvoke('list_channels');
        } catch (e) {
            console.error('获取通道列表失败:', e);
            vizChannels = [];
        }
    }

    renderChannelList();
    updateChartDatasets();
}

function renderChannelList() {
    var container = document.getElementById('vizChannelList');
    if (!container) return;

    container.innerHTML = '';

    if (vizChannels.length === 0) {
        container.innerHTML = '<div class="viz-empty">暂无数据通道，请创建新通道</div>';
        return;
    }

    vizChannels.forEach(function(channel) {
        var item = document.createElement('div');
        item.className = 'viz-channel-item';
        item.innerHTML =
            '<div class="viz-channel-color" style="background:' + channel.color + '"></div>' +
            '<div class="viz-channel-info">' +
                '<div class="viz-channel-name">' + channel.name + '</div>' +
                '<div class="viz-channel-detail">' + channel.point_count + ' 个数据点</div>' +
            '</div>' +
            '<div class="viz-channel-actions">' +
                '<button class="btn btn--ghost btn--sm" onclick="toggleChannelEnabled(\'' + channel.id + '\', ' + !channel.enabled + ')">' +
                    (channel.enabled ? '禁用' : '启用') +
                '</button>' +
                '<button class="btn btn--ghost btn--sm" onclick="removeChannel(\'' + channel.id + '\')">删除</button>' +
            '</div>';
        container.appendChild(item);
    });
}

function updateChartDatasets() {
    if (!vizChart) return;

    var datasets = vizChannels
        .filter(function(ch) { return ch.enabled; })
        .map(function(ch) {
            return {
                label: ch.name,
                data: [],
                borderColor: ch.color,
                backgroundColor: ch.color + '20',
                fill: false,
                tension: 0.1
            };
        });

    vizChart.data.datasets = datasets;
    vizChart.update();
}

async function createNewChannel() {
    var name = document.getElementById('vizNewChannelName').value;
    var color = document.getElementById('vizNewChannelColor').value;

    if (!name) {
        alert('请输入通道名称');
        return;
    }

    if (vizIsTauri && vizInvoke) {
        try {
            var channel = await vizInvoke('create_channel', { name: name, color: color });
            vizChannels.push(channel);
        } catch (e) {
            alert('创建通道失败: ' + e);
            return;
        }
    } else {
        var channel = {
            id: 'sim_' + Date.now(),
            name: name,
            color: color,
            enabled: true,
            point_count: 0
        };
        vizChannels.push(channel);
    }

    document.getElementById('vizNewChannelName').value = '';
    renderChannelList();
    updateChartDatasets();
}

async function toggleChannelEnabled(channelId, enabled) {
    if (vizIsTauri && vizInvoke) {
        try {
            await vizInvoke('update_channel', {
                channelId: channelId,
                enabled: enabled,
                name: null,
                color: null
            });
        } catch (e) {
            console.error('更新通道失败:', e);
        }
    }

    vizChannels.forEach(function(ch) {
        if (ch.id === channelId) {
            ch.enabled = enabled;
        }
    });

    renderChannelList();
    updateChartDatasets();
}

async function removeChannel(channelId) {
    if (!confirm('确定删除此通道？')) return;

    if (vizIsTauri && vizInvoke) {
        try {
            await vizInvoke('remove_channel', { channelId: channelId });
        } catch (e) {
            console.error('删除通道失败:', e);
        }
    }

    vizChannels = vizChannels.filter(function(ch) { return ch.id !== channelId; });
    renderChannelList();
    updateChartDatasets();
}

async function addManualDataPoint() {
    var channelId = document.getElementById('vizDataChannelSelect').value;
    var value = parseFloat(document.getElementById('vizDataValue').value);

    if (!channelId) {
        alert('请选择数据通道');
        return;
    }
    if (isNaN(value)) {
        alert('请输入有效数值');
        return;
    }

    if (vizIsTauri && vizInvoke) {
        try {
            await vizInvoke('add_data_point', { channelId: channelId, value: value });
        } catch (e) {
            alert('添加数据失败: ' + e);
            return;
        }
    }

    document.getElementById('vizDataValue').value = '';
}

function startAutoUpdate() {
    if (vizUpdateInterval) {
        clearInterval(vizUpdateInterval);
    }

    vizUpdateInterval = setInterval(async function() {
        if (!vizIsTauri || !vizInvoke) return;

        for (var i = 0; i < vizChannels.length; i++) {
            var ch = vizChannels[i];
            if (!ch.enabled) continue;

            try {
                var points = await vizInvoke('get_latest_channel_data', {
                    channelId: ch.id,
                    count: vizMaxDisplayPoints
                });

                if (vizChart && vizChart.data.datasets[i]) {
                    vizChart.data.datasets[i].data = points.map(function(p) {
                        return { x: p.timestamp, y: p.value };
                    });
                }
            } catch (e) {
                console.error('获取数据失败:', e);
            }
        }

        if (vizChart) {
            vizChart.update();
        }
    }, 100);
}

function stopAutoUpdate() {
    if (vizUpdateInterval) {
        clearInterval(vizUpdateInterval);
        vizUpdateInterval = null;
    }
}

function toggleAutoUpdate() {
    var btn = document.getElementById('vizAutoUpdateBtn');
    if (vizUpdateInterval) {
        stopAutoUpdate();
        btn.textContent = '开始实时更新';
        btn.classList.remove('btn--danger');
        btn.classList.add('btn--primary');
    } else {
        startAutoUpdate();
        btn.textContent = '停止更新';
        btn.classList.remove('btn--primary');
        btn.classList.add('btn--danger');
    }
}

function startSimulation() {
    if (vizSimInterval) {
        clearInterval(vizSimInterval);
    }

    var simTime = Date.now();
    var simStep = 0;

    vizSimInterval = setInterval(function() {
        simStep++;
        var now = Date.now();

        vizChannels.forEach(function(ch, idx) {
            if (!ch.enabled) return;

            var value;
            if (idx === 0) {
                value = Math.sin(simStep * 0.1) * 50 + 50;
            } else if (idx === 1) {
                value = Math.cos(simStep * 0.05) * 30 + 60;
            } else {
                value = Math.random() * 100;
            }

            if (vizIsTauri && vizInvoke) {
                vizInvoke('add_data_point', { channelId: ch.id, value: value });
            }

            if (vizChart && vizChart.data.datasets[idx]) {
                vizChart.data.datasets[idx].data.push({ x: now, y: value });
                if (vizChart.data.datasets[idx].data.length > vizMaxDisplayPoints) {
                    vizChart.data.datasets[idx].data.shift();
                }
            }
        });

        if (vizChart) {
            vizChart.update();
        }
    }, 50);
}

function stopSimulation() {
    if (vizSimInterval) {
        clearInterval(vizSimInterval);
        vizSimInterval = null;
    }
}

function toggleSimulation() {
    var btn = document.getElementById('vizSimBtn');
    if (vizSimInterval) {
        stopSimulation();
        btn.textContent = '开始模拟';
        btn.classList.remove('btn--danger');
        btn.classList.add('btn--primary');
    } else {
        startSimulation();
        btn.textContent = '停止模拟';
        btn.classList.remove('btn--primary');
        btn.classList.add('btn--danger');
    }
}

async function exportChannelData() {
    var format = document.getElementById('vizExportFormat').value;
    var enabledChannels = vizChannels.filter(function(ch) { return ch.enabled; });

    if (enabledChannels.length === 0) {
        alert('没有启用的通道可导出');
        return;
    }

    var channelIds = enabledChannels.map(function(ch) { return ch.id; });

    if (vizIsTauri && vizInvoke) {
        try {
            var data = await vizInvoke('export_data', { format: format, channelIds: channelIds });
            var outputArea = document.getElementById('vizExportOutput');
            outputArea.value = data;
            outputArea.style.display = 'block';
        } catch (e) {
            alert('导出失败: ' + e);
        }
    } else {
        document.getElementById('vizExportOutput').value = '[模拟导出数据] 格式: ' + format;
        document.getElementById('vizExportOutput').style.display = 'block';
    }
}

async function clearAllChannelData() {
    if (!confirm('确定清空所有通道数据？')) return;

    for (var i = 0; i < vizChannels.length; i++) {
        if (vizIsTauri && vizInvoke) {
            try {
                await vizInvoke('clear_channel_data', { channelId: vizChannels[i].id });
            } catch (e) {
                console.error('清空数据失败:', e);
            }
        }
        vizChannels[i].point_count = 0;
    }

    if (vizChart) {
        vizChart.data.datasets.forEach(function(ds) {
            ds.data = [];
        });
        vizChart.update();
    }

    renderChannelList();
}

function updateDataChannelSelect() {
    var select = document.getElementById('vizDataChannelSelect');
    if (!select) return;

    select.innerHTML = '<option value="">选择通道...</option>';
    vizChannels.forEach(function(ch) {
        var option = document.createElement('option');
        option.value = ch.id;
        option.textContent = ch.name;
        select.appendChild(option);
    });
}

window.initVisualization = initVisualization;
window.refreshChannelList = refreshChannelList;
window.createNewChannel = createNewChannel;
window.toggleChannelEnabled = toggleChannelEnabled;
window.removeChannel = removeChannel;
window.addManualDataPoint = addManualDataPoint;
window.toggleAutoUpdate = toggleAutoUpdate;
window.toggleSimulation = toggleSimulation;
window.exportChannelData = exportChannelData;
window.clearAllChannelData = clearAllChannelData;
window.updateDataChannelSelect = updateDataChannelSelect;
