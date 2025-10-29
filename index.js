// 万遥客户端主程序
console.log('万遥客户端启动中...');

// 简单的示例功能
function greetUser() {
    console.log('欢迎使用万遥客户端！');
    console.log('这是一个开源项目，欢迎贡献代码！');
}

// 主函数
function main() {
    greetUser();
    console.log('客户端已准备就绪');
}

// 启动应用
if (require.main === module) {
    main();
}

module.exports = {
    greetUser,
    main
};