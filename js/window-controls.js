import { getIsTauriEnvironment, getCurrentWindowInstance, invoke } from './common.js';

let isMaximized = false;

async function closeWindow() {
    try {
        if (getIsTauriEnvironment()) {
            await invoke('close_app');
        } else {
            if (confirm('确定要关闭窗口吗？')) {
                window.close();
            }
        }
    } catch (error) {
        console.error('关闭窗口失败:', error);
        if (confirm('确定要关闭窗口吗？')) {
            window.close();
        }
    }
}

async function minimizeWindow() {
    try {
        const currentWindow = getCurrentWindowInstance();
        if (getIsTauriEnvironment() && currentWindow) {
            await currentWindow.minimize();
        } else {
            console.log('最小化功能仅在桌面应用中可用');
        }
    } catch (error) {
        console.error('最小化窗口失败:', error);
    }
}

async function toggleMaximizeWindow() {
    try {
        const currentWindow = getCurrentWindowInstance();
        if (getIsTauriEnvironment() && currentWindow) {
            if (isMaximized) {
                await currentWindow.unmaximize();
                isMaximized = false;
            } else {
                await currentWindow.maximize();
                isMaximized = true;
            }
        } else {
            if (document.fullscreenElement) {
                await document.exitFullscreen();
                isMaximized = false;
            } else {
                await document.documentElement.requestFullscreen();
                isMaximized = true;
            }
        }
    } catch (error) {
        console.error('切换最大化失败:', error);
    }
}

export {
    closeWindow,
    minimizeWindow,
    toggleMaximizeWindow
};
