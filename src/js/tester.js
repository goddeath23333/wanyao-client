var testerIsTauri = false;
var testerInvoke = null;
var testerCases = [];
var testerEditingId = null;

async function initTester() {
    testerInvoke = window.getInvoke();
    testerIsTauri = window.getIsTauriEnvironment();

    if (testerIsTauri && testerInvoke) {
        console.log('自动化测试模块初始化完成 (Tauri 模式)');
        await refreshTestCases();
        await refreshTesterPorts();
    } else {
        console.log('自动化测试模块初始化完成 (Web 模拟模式)');
    }

    renderTestCaseList();
}

async function refreshTesterPorts() {
    var select = document.getElementById('testerPortSelect');
    if (!select) return;

    select.innerHTML = '<option value="">扫描中...</option>';

    if (testerIsTauri && testerInvoke) {
        try {
            var ports = await testerInvoke('list_serial_ports');
            select.innerHTML = '<option value="">选择串口...</option>';
            ports.forEach(function(port) {
                var option = document.createElement('option');
                option.value = port.name;
                var label = port.name;
                if (port.manufacturer) {
                    label += ' (' + port.manufacturer + ')';
                }
                option.textContent = label;
                select.appendChild(option);
            });
        } catch (e) {
            select.innerHTML = '<option value="">获取串口失败</option>';
        }
    } else {
        select.innerHTML = '<option value="">选择串口...</option>' +
            '<option value="COM1">COM1 (模拟)</option>' +
            '<option value="COM3">COM3 (模拟)</option>';
    }
}

async function refreshTestCases() {
    if (testerIsTauri && testerInvoke) {
        try {
            testerCases = await testerInvoke('list_test_cases');
        } catch (e) {
            console.error('获取测试用例失败:', e);
            testerCases = [];
        }
    }
    renderTestCaseList();
}

function renderTestCaseList() {
    var container = document.getElementById('testCaseList');
    if (!container) return;

    container.innerHTML = '';

    if (testerCases.length === 0) {
        container.innerHTML = '<div class="tester-empty">暂无测试用例，请添加新用例</div>';
        return;
    }

    testerCases.forEach(function(tc) {
        var statusClass = 'tester-status--' + tc.status;
        var statusText = {
            'idle': '待执行',
            'running': '执行中',
            'passed': '通过',
            'failed': '失败'
        }[tc.status] || tc.status;

        var item = document.createElement('div');
        item.className = 'tester-case-item';
        item.innerHTML =
            '<div class="tester-case-header">' +
                '<div class="tester-case-name">' + tc.name + '</div>' +
                '<span class="tester-status ' + statusClass + '">' + statusText + '</span>' +
            '</div>' +
            '<div class="tester-case-detail">' + tc.description + '</div>' +
            '<div class="tester-case-cmd">命令: ' + tc.command + '</div>' +
            '<div class="tester-case-expected">期望: ' + tc.expected_response + '</div>' +
            '<div class="tester-case-actions">' +
                '<button class="btn btn--ghost btn--sm" onclick="editTestCase(\'' + tc.id + '\')">编辑</button>' +
                '<button class="btn btn--ghost btn--sm" onclick="runSingleTest(\'' + tc.id + '\')">执行</button>' +
                '<button class="btn btn--ghost btn--sm btn--danger-text" onclick="deleteTestCase(\'' + tc.id + '\')">删除</button>' +
            '</div>';
        container.appendChild(item);
    });
}

function showAddTestCaseForm() {
    testerEditingId = null;
    document.getElementById('testerFormTitle').textContent = '添加测试用例';
    document.getElementById('testCaseName').value = '';
    document.getElementById('testCaseDesc').value = '';
    document.getElementById('testCaseCmd').value = '';
    document.getElementById('testCaseExpected').value = '';
    document.getElementById('testCaseTimeout').value = '5000';
    document.getElementById('testCaseForm').style.display = 'block';
}

function hideTestCaseForm() {
    document.getElementById('testCaseForm').style.display = 'none';
    testerEditingId = null;
}

function editTestCase(testId) {
    var tc = testerCases.find(function(c) { return c.id === testId; });
    if (!tc) return;

    testerEditingId = testId;
    document.getElementById('testerFormTitle').textContent = '编辑测试用例';
    document.getElementById('testCaseName').value = tc.name;
    document.getElementById('testCaseDesc').value = tc.description;
    document.getElementById('testCaseCmd').value = tc.command;
    document.getElementById('testCaseExpected').value = tc.expected_response;
    document.getElementById('testCaseTimeout').value = tc.timeout_ms;
    document.getElementById('testCaseForm').style.display = 'block';
}

async function saveTestCase() {
    var name = document.getElementById('testCaseName').value;
    var description = document.getElementById('testCaseDesc').value;
    var command = document.getElementById('testCaseCmd').value;
    var expected = document.getElementById('testCaseExpected').value;
    var timeout = parseInt(document.getElementById('testCaseTimeout').value) || 5000;

    if (!name || !command || !expected) {
        alert('请填写用例名称、命令和期望响应');
        return;
    }

    if (testerEditingId) {
        if (testerIsTauri && testerInvoke) {
            try {
                await testerInvoke('update_test_case', {
                    testId: testerEditingId,
                    name: name,
                    description: description,
                    command: command,
                    expectedResponse: expected,
                    timeoutMs: timeout
                });
            } catch (e) {
                alert('更新失败: ' + e);
                return;
            }
        }
    } else {
        if (testerIsTauri && testerInvoke) {
            try {
                var tc = await testerInvoke('create_test_case', {
                    name: name,
                    description: description,
                    command: command,
                    expectedResponse: expected,
                    timeoutMs: timeout
                });
                testerCases.push(tc);
            } catch (e) {
                alert('创建失败: ' + e);
                return;
            }
        } else {
            testerCases.push({
                id: 'sim_' + Date.now(),
                name: name,
                description: description,
                command: command,
                expected_response: expected,
                timeout_ms: timeout,
                status: 'idle'
            });
        }
    }

    hideTestCaseForm();
    await refreshTestCases();
}

async function deleteTestCase(testId) {
    if (!confirm('确定删除此测试用例？')) return;

    if (testerIsTauri && testerInvoke) {
        try {
            await testerInvoke('delete_test_case', { testId: testId });
        } catch (e) {
            console.error('删除失败:', e);
        }
    }

    testerCases = testerCases.filter(function(tc) { return tc.id !== testId; });
    renderTestCaseList();
}

async function runSingleTest(testId) {
    var portName = document.getElementById('testerPortSelect').value;
    var baudRate = parseInt(document.getElementById('testerBaudRate').value);

    if (!portName) {
        alert('请选择测试串口');
        return;
    }

    if (testerIsTauri && testerInvoke) {
        try {
            var result = await testerInvoke('run_test_case', {
                testId: testId,
                portName: portName,
                baudRate: baudRate
            });
            appendTestResult(result);
        } catch (e) {
            alert('执行失败: ' + e);
        }
    } else {
        var tc = testerCases.find(function(c) { return c.id === testId; });
        appendTestResult({
            test_id: testId,
            test_name: tc ? tc.name : '未知',
            passed: Math.random() > 0.3,
            message: '模拟测试结果',
            actual_response: 'OK\r\n',
            duration_ms: Math.floor(Math.random() * 500) + 100
        });
    }

    await refreshTestCases();
}

async function runAllTests() {
    var portName = document.getElementById('testerPortSelect').value;
    var baudRate = parseInt(document.getElementById('testerBaudRate').value);

    if (!portName) {
        alert('请选择测试串口');
        return;
    }

    if (testerCases.length === 0) {
        alert('没有测试用例可执行');
        return;
    }

    var reportArea = document.getElementById('testReportArea');
    reportArea.innerHTML = '<div class="tester-running">正在执行所有测试...</div>';

    if (testerIsTauri && testerInvoke) {
        try {
            var report = await testerInvoke('run_all_tests', {
                portName: portName,
                baudRate: baudRate
            });
            renderTestReport(report);
        } catch (e) {
            reportArea.innerHTML = '<div class="tester-error">执行失败: ' + e + '</div>';
        }
    } else {
        var report = {
            total_tests: testerCases.length,
            passed: Math.floor(testerCases.length * 0.7),
            failed: testerCases.length - Math.floor(testerCases.length * 0.7),
            skipped: 0,
            duration_ms: testerCases.length * 300,
            results: testerCases.map(function(tc) {
                return {
                    test_id: tc.id,
                    test_name: tc.name,
                    passed: Math.random() > 0.3,
                    message: '模拟测试结果',
                    actual_response: 'OK\r\n',
                    duration_ms: Math.floor(Math.random() * 500) + 100
                };
            }),
            timestamp: new Date().toLocaleString('zh-CN')
        };
        renderTestReport(report);
    }

    await refreshTestCases();
}

function appendTestResult(result) {
    var reportArea = document.getElementById('testReportArea');
    var div = document.createElement('div');
    div.className = 'tester-result-item' + (result.passed ? ' tester-result-item--pass' : ' tester-result-item--fail');
    div.innerHTML =
        '<span class="tester-result-icon">' + (result.passed ? '✓' : '✗') + '</span>' +
        '<span class="tester-result-name">' + result.test_name + '</span>' +
        '<span class="tester-result-msg">' + result.message + '</span>' +
        '<span class="tester-result-time">' + result.duration_ms + 'ms</span>';
    reportArea.insertBefore(div, reportArea.firstChild);
}

function renderTestReport(report) {
    var reportArea = document.getElementById('testReportArea');
    var html =
        '<div class="tester-report-summary">' +
            '<div class="tester-report-stat tester-report-stat--total">总计: ' + report.total_tests + '</div>' +
            '<div class="tester-report-stat tester-report-stat--pass">通过: ' + report.passed + '</div>' +
            '<div class="tester-report-stat tester-report-stat--fail">失败: ' + report.failed + '</div>' +
            '<div class="tester-report-stat">耗时: ' + report.duration_ms + 'ms</div>' +
        '</div>';

    report.results.forEach(function(result) {
        html +=
            '<div class="tester-result-item' + (result.passed ? ' tester-result-item--pass' : ' tester-result-item--fail') + '">' +
                '<span class="tester-result-icon">' + (result.passed ? '✓' : '✗') + '</span>' +
                '<span class="tester-result-name">' + result.test_name + '</span>' +
                '<span class="tester-result-msg">' + result.message + '</span>' +
                '<span class="tester-result-time">' + result.duration_ms + 'ms</span>' +
            '</div>';
        if (!result.passed && result.actual_response) {
            html += '<div class="tester-result-detail">实际响应: ' + result.actual_response + '</div>';
        }
    });

    reportArea.innerHTML = html;
}

async function loadTestScript() {
    var path = document.getElementById('testerScriptPath').value;
    if (!path) {
        alert('请输入脚本文件路径');
        return;
    }

    if (testerIsTauri && testerInvoke) {
        try {
            var cases = await testerInvoke('load_test_script', { scriptPath: path });
            alert('成功加载 ' + cases.length + ' 个测试用例');
        } catch (e) {
            alert('加载脚本失败: ' + e);
        }
    } else {
        alert('模拟: 成功加载 3 个测试用例');
    }

    await refreshTestCases();
}

async function generateTestReport() {
    var format = document.getElementById('testerReportFormat').value;

    if (testerIsTauri && testerInvoke) {
        try {
            var reportText = await testerInvoke('generate_report', { format: format });
            var outputArea = document.getElementById('testerReportOutput');
            outputArea.value = reportText;
            outputArea.style.display = 'block';
        } catch (e) {
            alert('生成报告失败: ' + e);
        }
    } else {
        document.getElementById('testerReportOutput').value = '[模拟测试报告] 格式: ' + format;
        document.getElementById('testerReportOutput').style.display = 'block';
    }
}

async function clearAllTestCases() {
    if (!confirm('确定清空所有测试用例？')) return;

    if (testerIsTauri && testerInvoke) {
        try {
            await testerInvoke('clear_test_cases');
        } catch (e) {
            console.error('清空失败:', e);
        }
    }

    testerCases = [];
    renderTestCaseList();
    document.getElementById('testReportArea').innerHTML = '';
}

window.initTester = initTester;
window.refreshTesterPorts = refreshTesterPorts;
window.refreshTestCases = refreshTestCases;
window.showAddTestCaseForm = showAddTestCaseForm;
window.hideTestCaseForm = hideTestCaseForm;
window.editTestCase = editTestCase;
window.saveTestCase = saveTestCase;
window.deleteTestCase = deleteTestCase;
window.runSingleTest = runSingleTest;
window.runAllTests = runAllTests;
window.loadTestScript = loadTestScript;
window.generateTestReport = generateTestReport;
window.clearAllTestCases = clearAllTestCases;
