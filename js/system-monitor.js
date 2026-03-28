import { invoke, getIsTauriEnvironment, formatBytes } from './common.js';

let systemInfoInterval = null;
let isUpdating = false;
let isResizing = false;
let resizeTimeout = null;
let usageChart = null;
const MAX_DATA_POINTS = 60;
const chartData = {
    cpu: [],
    memory: [],
    gpu: []
};

function getFallbackSystemInfo() {
    return {
        cpu: {
            name: '检测中...',
            usage: 0,
            cores: 0
        },
        memory: {
            total: 0,
            used: 0,
            usage: 0
        },
        gpus: [{ name: '检测中...', usage: 0, vendor: 'Unknown' }],
        os: {
            name: '检测中...',
            version: ''
        },
        is_simulated: true
    };
}

async function getSystemInfo() {
    if (isResizing) return null;
    
    try {
        if (getIsTauriEnvironment()) {
            const rawInfo = await invoke('get_system_info');
            if (rawInfo && rawInfo.memory_total > 0) {
                const systemInfo = {
                    cpu: {
                        name: rawInfo.cpu_name || 'Unknown CPU',
                        usage: rawInfo.cpu_usage || 0,
                        cores: rawInfo.cpu_cores || 0
                    },
                    memory: {
                        total: rawInfo.memory_total || 0,
                        used: rawInfo.memory_used || 0,
                        usage: rawInfo.memory_usage || 0
                    },
                    gpus: [{ 
                        name: rawInfo.gpu_name || 'Unknown GPU', 
                        usage: rawInfo.gpu_usage || 0, 
                        vendor: 'Unknown' 
                    }],
                    os: {
                        name: rawInfo.os_name || 'Unknown',
                        version: rawInfo.os_version || ''
                    },
                    is_simulated: false
                };
                return systemInfo;
            }
        }
        return getFallbackSystemInfo();
    } catch (error) {
        console.error('获取系统信息失败:', error);
        return getFallbackSystemInfo();
    }
}

function initChart() {
    const ctx = document.getElementById('usageChart');
    if (!ctx) return;
    
    const isDark = document.documentElement.getAttribute('data-theme') !== 'light';
    const gridColor = isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)';
    const textColor = isDark ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)';
    
    usageChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: Array(MAX_DATA_POINTS).fill(''),
            datasets: [
                {
                    label: 'CPU',
                    data: [],
                    borderColor: '#60a5fa',
                    backgroundColor: 'rgba(96, 165, 250, 0.1)',
                    borderWidth: 2,
                    fill: true,
                    tension: 0.4,
                    pointRadius: 0
                },
                {
                    label: '内存',
                    data: [],
                    borderColor: '#a78bfa',
                    backgroundColor: 'rgba(167, 139, 250, 0.1)',
                    borderWidth: 2,
                    fill: true,
                    tension: 0.4,
                    pointRadius: 0
                },
                {
                    label: 'GPU',
                    data: [],
                    borderColor: '#fbbf24',
                    backgroundColor: 'rgba(251, 191, 36, 0.1)',
                    borderWidth: 2,
                    fill: true,
                    tension: 0.4,
                    pointRadius: 0
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: {
                duration: 0
            },
            interaction: {
                intersect: false,
                mode: 'index'
            },
            plugins: {
                legend: {
                    display: false
                }
            },
            scales: {
                x: {
                    display: false
                },
                y: {
                    min: 0,
                    max: 100,
                    grid: {
                        color: gridColor
                    },
                    ticks: {
                        color: textColor,
                        callback: value => value + '%'
                    }
                }
            }
        }
    });
}

function updateChart(cpuUsage, memoryUsage, gpuUsage) {
    if (!usageChart) return;
    
    chartData.cpu.push(cpuUsage);
    chartData.memory.push(memoryUsage);
    chartData.gpu.push(gpuUsage);
    
    if (chartData.cpu.length > MAX_DATA_POINTS) {
        chartData.cpu.shift();
        chartData.memory.shift();
        chartData.gpu.shift();
    }
    
    usageChart.data.datasets[0].data = chartData.cpu;
    usageChart.data.datasets[1].data = chartData.memory;
    usageChart.data.datasets[2].data = chartData.gpu;
    usageChart.update('none');
}

function updateDashboardStats(systemInfo) {
    if (!systemInfo || isUpdating || isResizing) return;
    
    isUpdating = true;
    
    requestAnimationFrame(() => {
        const cpu = systemInfo.cpu || {};
        const memory = systemInfo.memory || {};
        const gpus = systemInfo.gpus || [];
        
        const hwCpuName = document.getElementById('hwCpuName');
        const hwMemorySize = document.getElementById('hwMemorySize');
        
        if (hwCpuName && cpu.name) {
            hwCpuName.textContent = cpu.name;
        }
        if (hwMemorySize && memory.total) {
            hwMemorySize.textContent = formatBytes(memory.total);
        }
        
        const cpuValue = document.getElementById('statCpuValue');
        const cpuBar = document.getElementById('statCpuBar');
        const cpuCores = document.getElementById('statCpuCores');
        
        if (cpuValue && cpuBar) {
            const cpuUsage = Math.round(cpu.usage || 0);
            cpuValue.textContent = `${cpuUsage}`;
            cpuBar.style.width = `${cpuUsage}%`;
        }
        if (cpuCores && cpu.cores) {
            cpuCores.textContent = `${cpu.cores} 核心`;
        }
        
        const memoryValue = document.getElementById('statMemoryValue');
        const memoryBar = document.getElementById('statMemoryBar');
        const memoryDetail = document.getElementById('statMemoryDetail');
        
        if (memoryValue && memoryBar) {
            const memUsage = Math.round(memory.usage || 0);
            memoryValue.textContent = `${memUsage}`;
            memoryBar.style.width = `${memUsage}%`;
        }
        if (memoryDetail) {
            const memUsed = formatBytes(memory.used || 0);
            const memTotal = formatBytes(memory.total || 0);
            memoryDetail.textContent = `${memUsed} / ${memTotal}`;
        }
        
        const gpuContainer = document.getElementById('gpuContainer');
        const gpuCard = document.getElementById('gpuCard');
        
        if (gpuContainer && gpuCard && gpus.length > 0) {
            if (gpus.length === 1) {
                gpuCard.style.display = 'flex';
                gpuContainer.style.display = 'none';
                
                const hwGpuName = document.getElementById('hwGpuName');
                const statGpuValue = document.getElementById('statGpuValue');
                const statGpuBar = document.getElementById('statGpuBar');
                
                if (hwGpuName) hwGpuName.textContent = gpus[0].name;
                if (statGpuValue) statGpuValue.textContent = Math.round(gpus[0].usage || 0);
                if (statGpuBar) statGpuBar.style.width = `${gpus[0].usage || 0}%`;
            } else if (gpus.length > 1) {
                gpuCard.style.display = 'none';
                gpuContainer.style.display = 'grid';
                
                let gpuHtml = '';
                gpus.forEach((gpu, index) => {
                    const vendorIcon = gpu.vendor === 'NVIDIA' ? '🟢' : gpu.vendor === 'AMD' ? '🔴' : '⚪';
                    gpuHtml += `
                        <div class="monitor-card">
                            <div class="monitor-card__header">
                                <div class="monitor-card__icon">🎮</div>
                                <div class="monitor-card__info">
                                    <div class="monitor-card__name">${gpu.name}</div>
                                    <div class="monitor-card__detail">${vendorIcon} ${gpu.vendor} · GPU ${index + 1}</div>
                                </div>
                            </div>
                            <div class="monitor-card__usage">
                                <span class="monitor-card__value">${Math.round(gpu.usage || 0)}</span>
                                <span class="monitor-card__unit">%</span>
                            </div>
                            <div class="monitor-card__bar">
                                <div class="monitor-card__bar-fill gpu" style="width: ${gpu.usage || 0}%"></div>
                            </div>
                        </div>
                    `;
                });
                gpuContainer.innerHTML = gpuHtml;
            }
        }
        
        const firstGpuUsage = gpus && gpus[0] ? (gpus[0].usage || 0) : 0;
        updateChart(
            Math.round(cpu.usage || 0),
            Math.round(memory.usage || 0),
            Math.round(firstGpuUsage)
        );
        
        isUpdating = false;
    });
}

async function refreshStats() {
    if (isResizing) return;
    const info = await getSystemInfo();
    if (info) {
        updateDashboardStats(info);
    }
}

function handleResizeStart() {
    isResizing = true;
    if (resizeTimeout) {
        clearTimeout(resizeTimeout);
    }
}

function handleResizeEnd() {
    if (resizeTimeout) {
        clearTimeout(resizeTimeout);
    }
    resizeTimeout = setTimeout(() => {
        isResizing = false;
        refreshStats();
    }, 200);
}

function startMonitoring() {
    if (systemInfoInterval) {
        clearInterval(systemInfoInterval);
    }
    
    window.addEventListener('resize', handleResizeStart);
    window.addEventListener('resize', handleResizeEnd);
    
    refreshStats();
    
    systemInfoInterval = setInterval(refreshStats, 3000);
}

function stopMonitoring() {
    if (systemInfoInterval) {
        clearInterval(systemInfoInterval);
        systemInfoInterval = null;
    }
    
    window.removeEventListener('resize', handleResizeStart);
    window.removeEventListener('resize', handleResizeEnd);
    
    if (resizeTimeout) {
        clearTimeout(resizeTimeout);
        resizeTimeout = null;
    }
}

export {
    getSystemInfo,
    updateDashboardStats,
    startMonitoring,
    stopMonitoring,
    initChart,
    updateChart
};
