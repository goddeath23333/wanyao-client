const fs = require('fs');
const path = require('path');

const distDir = path.join(__dirname, '..', 'dist');
const rootDir = path.join(__dirname, '..');

const filesToCopy = [
    'index.html',
    'index.js',
    'server.js'
];

const dirsToCopy = [
    'css',
    'js'
];

function ensureDir(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

function copyFile(src, dest) {
    fs.copyFileSync(src, dest);
    console.log('Copied: ' + src + ' -> ' + dest);
}

function copyDir(src, dest) {
    ensureDir(dest);
    const entries = fs.readdirSync(src, { withFileTypes: true });
    
    for (const entry of entries) {
        const srcPath = path.join(src, entry.name);
        const destPath = path.join(dest, entry.name);
        
        if (entry.isDirectory()) {
            copyDir(srcPath, destPath);
        } else {
            copyFile(srcPath, destPath);
        }
    }
}

function main() {
    console.log('Building dist directory...');
    
    ensureDir(distDir);
    
    for (const file of filesToCopy) {
        const src = path.join(rootDir, file);
        const dest = path.join(distDir, file);
        
        if (fs.existsSync(src)) {
            copyFile(src, dest);
        } else {
            console.warn('Warning: ' + file + ' not found, skipping');
        }
    }
    
    for (const dir of dirsToCopy) {
        const src = path.join(rootDir, dir);
        const dest = path.join(distDir, dir);
        
        if (fs.existsSync(src)) {
            copyDir(src, dest);
        } else {
            console.warn('Warning: ' + dir + ' directory not found, skipping');
        }
    }
    
    console.log('Build complete!');
}

main();
