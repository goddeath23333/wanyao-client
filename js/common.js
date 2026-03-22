import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

let isTauriEnvironment = false;
let currentWindow = null;
let currentTheme = 'dark';
let currentPage = 'dashboard';

const THEME_KEY = 'wanyao-theme';

async function detectTauriEnvironment() {
    try {
        currentWindow = getCurrentWindow();
        isTauriEnvironment = true;
        console.log('Tauri 环境检测: 是');
    } catch (error) {
        isTauriEnvironment = false;
        console.log('浏览器环境');
    }
    return isTauriEnvironment;
}

function getIsTauriEnvironment() {
    return isTauriEnvironment;
}

function getCurrentWindowInstance() {
    return currentWindow;
}

function showMessage(message, type = 'info') {
    console.log(`[${type}] ${message}`);
}

function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function initTheme() {
    const savedTheme = localStorage.getItem(THEME_KEY);
    if (savedTheme) {
        currentTheme = savedTheme;
    } else {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
            currentTheme = 'light';
        }
    }
    applyTheme(currentTheme);
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
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
}

function navigateTo(page) {
    currentPage = page;
    
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.remove('active');
        if (item.dataset.page === page) {
            item.classList.add('active');
        }
    });
    
    document.querySelectorAll('.page').forEach(p => {
        p.classList.remove('active');
    });
    
    const targetPage = document.getElementById(`page-${page}`);
    if (targetPage) {
        targetPage.classList.add('active');
    }
    
    const titleEl = document.querySelector('.content__title');
    if (titleEl) {
        const titles = {
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
    const sidebar = document.querySelector('.sidebar');
    if (sidebar) {
        sidebar.classList.toggle('collapsed');
    }
}

export {
    invoke,
    detectTauriEnvironment,
    getIsTauriEnvironment,
    getCurrentWindowInstance,
    showMessage,
    formatBytes,
    initTheme,
    setTheme,
    getTheme,
    toggleTheme,
    navigateTo,
    toggleSidebar,
    THEME_KEY
};
