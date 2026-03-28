let isTauriEnvironment = false;
let currentWindow = null;
let currentTheme = 'dark';
let currentPage = 'dashboard';
let invoke = null;
let getCurrentWindow = null;

const THEME_KEY = 'wanyao-theme';

function debugLog(message, type = 'log') {
    if (window.debugLog && window.debugLog !== debugLog) {
        window.debugLog(message, type);
    }
    console.log('[' + type + '] ' + message);
}

async function detectTauriEnvironment() {
    debugLog('开始检测 Tauri 环境...', 'info');
    debugLog('window.__TAURI__ 存在: ' + !!window.__TAURI__, 'info');
    
    try {
        if (window.__TAURI__) {
            debugLog('检测到全局 Tauri API', 'success');
            var tauriCore = window.__TAURI__.core;
            var tauriWindow = window.__TAURI__.window;
            
            debugLog('tauriCore 存在: ' + !!tauriCore, 'info');
            debugLog('tauriWindow 存在: ' + !!tauriWindow, 'info');
            debugLog('tauriCore.invoke 存在: ' + !!(tauriCore && tauriCore.invoke), 'info');
            
            invoke = tauriCore.invoke;
            getCurrentWindow = tauriWindow.getCurrentWindow;
            currentWindow = getCurrentWindow();
            isTauriEnvironment = true;
            debugLog('Tauri 环境初始化成功 (全局 API)', 'success');
        } else {
            debugLog('未检测到 Tauri 环境，运行在 Web 模式', 'warn');
            isTauriEnvironment = false;
        }
    } catch (error) {
        isTauriEnvironment = false;
        invoke = null;
        getCurrentWindow = null;
        currentWindow = null;
        debugLog('Tauri 环境检测失败: ' + error.message, 'error');
        debugLog('错误堆栈: ' + error.stack, 'error');
    }
    
    debugLog('最终 isTauriEnvironment: ' + isTauriEnvironment, 'info');
    debugLog('最终 invoke 存在: ' + !!invoke, 'info');
    return isTauriEnvironment;
}

function getIsTauriEnvironment() {
    return isTauriEnvironment;
}

function getCurrentWindowInstance() {
    return currentWindow;
}

function getInvoke() {
    debugLog('getInvoke 被调用, invoke 存在: ' + !!invoke, 'info');
    return invoke;
}

function showMessage(message, type) {
    type = type || 'info';
    debugLog(message, type);
}

function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    var k = 1024;
    var sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    var i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function initTheme() {
    var savedTheme = localStorage.getItem(THEME_KEY);
    if (savedTheme) {
        currentTheme = savedTheme;
    } else {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
            currentTheme = 'light';
        }
    }
    applyTheme(currentTheme);
    debugLog('主题初始化完成: ' + currentTheme, 'info');
}

function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    currentTheme = theme;
    localStorage.setItem(THEME_KEY, theme);
}

function setTheme(theme) {
    if (theme === 'system') {
        localStorage.removeItem(THEME_KEY);
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
            applyTheme('light');
        } else {
            applyTheme('dark');
        }
    } else {
        applyTheme(theme);
    }
}

function getTheme() {
    return currentTheme;
}

function toggleTheme() {
    var newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    debugLog('切换主题: ' + newTheme, 'info');
}

function navigateTo(page) {
    debugLog('导航到页面: ' + page, 'info');
    currentPage = page;
    
    document.querySelectorAll('.nav-item').forEach(function(item) {
        item.classList.remove('active');
        if (item.dataset.page === page) {
            item.classList.add('active');
        }
    });
    
    document.querySelectorAll('.page').forEach(function(p) {
        p.classList.remove('active');
    });
    
    var targetPage = document.getElementById('page-' + page);
    if (targetPage) {
        targetPage.classList.add('active');
    }
    
    var titleEl = document.querySelector('.content__title');
    if (titleEl) {
        var titles = {
            dashboard: '仪表盘',
            serial: '串口助手',
            network: '网络助手',
            flasher: '固件烧录',
            visualization: '数据可视化',
            tester: '自动化测试',
            settings: '设置'
        };
        titleEl.textContent = titles[page] || page;
    }
}

function toggleSidebar() {
    var sidebar = document.querySelector('.sidebar');
    if (sidebar) {
        sidebar.classList.toggle('collapsed');
    }
}

window.detectTauriEnvironment = detectTauriEnvironment;
window.getIsTauriEnvironment = getIsTauriEnvironment;
window.getCurrentWindowInstance = getCurrentWindowInstance;
window.getInvoke = getInvoke;
window.showMessage = showMessage;
window.formatBytes = formatBytes;
window.initTheme = initTheme;
window.setTheme = setTheme;
window.getTheme = getTheme;
window.toggleTheme = toggleTheme;
window.navigateTo = navigateTo;
window.toggleSidebar = toggleSidebar;
window.THEME_KEY = THEME_KEY;
