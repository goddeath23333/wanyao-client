const http = require('http');
const fs = require('fs');
const path = require('path');

const port = 3001;

const mimeTypes = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpg',
  '.gif': 'image/gif',
  '.ico': 'image/x-icon'
};

const server = http.createServer((req, res) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  
  // 处理 @tauri-apps/api 模块请求
  if (req.url.startsWith('/@tauri-apps/')) {
    // 移除开头的斜杠
    const relativePath = req.url.substring(1);
    const modulePath = path.join(__dirname, 'node_modules', relativePath);
    
    fs.readFile(modulePath, (error, content) => {
      if (error) {
        console.error('模块未找到:', modulePath);
        res.writeHead(404);
        res.end('模块未找到: ' + req.url);
      } else {
        res.writeHead(200, { 'Content-Type': 'text/javascript' });
        res.end(content, 'utf-8');
      }
    });
    return;
  }
  
  // 处理根路径
  let filePath = req.url === '/' ? '/index.html' : req.url;
  
  // 处理查询参数
  if (filePath.includes('?')) {
    filePath = filePath.split('?')[0];
  }
  
  filePath = path.join(__dirname, 'src', filePath);
  
  const extname = path.extname(filePath);
  const contentType = mimeTypes[extname] || 'text/plain';
  
  fs.readFile(filePath, (error, content) => {
    if (error) {
      if (error.code === 'ENOENT') {
        res.writeHead(404);
        res.end('文件未找到');
      } else {
        res.writeHead(500);
        res.end('服务器错误: ' + error.code);
      }
    } else {
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(content, 'utf-8');
    }
  });
});

server.listen(port, () => {
  console.log(`🚀 本地服务器运行在 http://localhost:${port}`);
  console.log(`📁 服务目录: ${__dirname}`);
  console.log(`⏰ 启动时间: ${new Date().toLocaleString()}`);
});

// 优雅关闭
process.on('SIGINT', () => {
  console.log('\n🛑 正在关闭服务器...');
  server.close(() => {
    console.log('✅ 服务器已关闭');
    process.exit(0);
  });
});