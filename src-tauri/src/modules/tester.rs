use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub expected_response: String,
    pub timeout_ms: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub passed: bool,
    pub message: String,
    pub actual_response: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
    pub results: Vec<TestResult>,
    pub timestamp: String,
}

static TEST_CASES: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, TestCase>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static LAST_REPORT: once_cell::sync::Lazy<Arc<Mutex<Option<TestReport>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

fn now_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[tauri::command]
pub fn create_test_case(
    name: String,
    description: String,
    command: String,
    expected_response: String,
    timeout_ms: u64,
) -> Result<TestCase, String> {
    let id = Uuid::new_v4().to_string();
    let test_case = TestCase {
        id: id.clone(),
        name,
        description,
        command,
        expected_response,
        timeout_ms,
        status: "idle".to_string(),
    };

    TEST_CASES
        .lock()
        .unwrap()
        .insert(id, test_case.clone());

    Ok(test_case)
}

#[tauri::command]
pub fn update_test_case(
    test_id: String,
    name: Option<String>,
    description: Option<String>,
    command: Option<String>,
    expected_response: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<TestCase, String> {
    let mut cases = TEST_CASES.lock().unwrap();
    let tc = cases
        .get_mut(&test_id)
        .ok_or("测试用例不存在".to_string())?;

    if let Some(n) = name {
        tc.name = n;
    }
    if let Some(d) = description {
        tc.description = d;
    }
    if let Some(c) = command {
        tc.command = c;
    }
    if let Some(e) = expected_response {
        tc.expected_response = e;
    }
    if let Some(t) = timeout_ms {
        tc.timeout_ms = t;
    }
    tc.status = "idle".to_string();

    Ok(tc.clone())
}

#[tauri::command]
pub fn delete_test_case(test_id: String) -> Result<String, String> {
    TEST_CASES
        .lock()
        .unwrap()
        .remove(&test_id)
        .ok_or("测试用例不存在".to_string())?;
    Ok("测试用例已删除".to_string())
}

#[tauri::command]
pub fn list_test_cases() -> Result<Vec<TestCase>, String> {
    let cases = TEST_CASES.lock().unwrap();
    Ok(cases.values().cloned().collect())
}

fn execute_single_test(test_case: &TestCase, port_name: &str, baud_rate: u32) -> TestResult {
    let start = Instant::now();

    let port_result = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(test_case.timeout_ms))
        .open();

    let mut port = match port_result {
        Ok(p) => p,
        Err(e) => {
            return TestResult {
                test_id: test_case.id.clone(),
                test_name: test_case.name.clone(),
                passed: false,
                message: format!("打开串口失败: {}", e),
                actual_response: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    };

    if let Err(e) = port.write_all(test_case.command.as_bytes()) {
        return TestResult {
            test_id: test_case.id.clone(),
            test_name: test_case.name.clone(),
            passed: false,
            message: format!("发送命令失败: {}", e),
            actual_response: String::new(),
            duration_ms: start.elapsed().as_millis() as u64,
        };
    }

    let _ = port.flush();

    let mut response = Vec::new();
    let mut buf = [0u8; 1024];
    let deadline = Instant::now() + Duration::from_millis(test_case.timeout_ms);

    while Instant::now() < deadline {
        match port.read(&mut buf) {
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                let response_str = String::from_utf8_lossy(&response).to_string();
                if response_str.contains(&test_case.expected_response) {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if !response.is_empty() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let actual_response = String::from_utf8_lossy(&response).to_string();
    let duration_ms = start.elapsed().as_millis() as u64;

    let passed = actual_response.contains(&test_case.expected_response);
    let message = if passed {
        "测试通过".to_string()
    } else if actual_response.is_empty() {
        "超时: 未收到响应".to_string()
    } else {
        "响应不匹配".to_string()
    };

    TestResult {
        test_id: test_case.id.clone(),
        test_name: test_case.name.clone(),
        passed,
        message,
        actual_response,
        duration_ms,
    }
}

#[tauri::command]
pub fn run_test_case(
    test_id: String,
    port_name: String,
    baud_rate: u32,
) -> Result<TestResult, String> {
    let cases = TEST_CASES.lock().unwrap();
    let test_case = cases
        .get(&test_id)
        .ok_or("测试用例不存在".to_string())?
        .clone();
    drop(cases);

    {
        let mut cases = TEST_CASES.lock().unwrap();
        if let Some(tc) = cases.get_mut(&test_id) {
            tc.status = "running".to_string();
        }
    }

    let result = execute_single_test(&test_case, &port_name, baud_rate);

    {
        let mut cases = TEST_CASES.lock().unwrap();
        if let Some(tc) = cases.get_mut(&test_id) {
            tc.status = if result.passed {
                "passed".to_string()
            } else {
                "failed".to_string()
            };
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn run_all_tests(
    port_name: String,
    baud_rate: u32,
) -> Result<TestReport, String> {
    let cases = TEST_CASES.lock().unwrap();
    let test_cases: Vec<TestCase> = cases.values().cloned().collect();
    drop(cases);

    let start = Instant::now();
    let mut results = Vec::new();
    let mut passed = 0u32;
    let mut failed = 0u32;

    for test_case in &test_cases {
        {
            let mut cases = TEST_CASES.lock().unwrap();
            if let Some(tc) = cases.get_mut(&test_case.id) {
                tc.status = "running".to_string();
            }
        }

        let result = execute_single_test(test_case, &port_name, baud_rate);

        {
            let mut cases = TEST_CASES.lock().unwrap();
            if let Some(tc) = cases.get_mut(&test_case.id) {
                tc.status = if result.passed {
                    "passed".to_string()
                } else {
                    "failed".to_string()
                };
            }
        }

        if result.passed {
            passed += 1;
        } else {
            failed += 1;
        }

        results.push(result);
    }

    let total_tests = test_cases.len() as u32;
    let duration_ms = start.elapsed().as_millis() as u64;

    let report = TestReport {
        total_tests,
        passed,
        failed,
        skipped: 0,
        duration_ms,
        results,
        timestamp: now_timestamp(),
    };

    *LAST_REPORT.lock().unwrap() = Some(report.clone());

    Ok(report)
}

#[tauri::command]
pub fn load_test_script(script_path: String) -> Result<Vec<TestCase>, String> {
    let content =
        std::fs::read_to_string(&script_path).map_err(|e| format!("读取脚本失败: {}", e))?;

    let script_tests: Vec<ScriptTestCase> =
        serde_json::from_str(&content).map_err(|e| format!("解析脚本失败: {}", e))?;

    let mut created = Vec::new();
    let mut cases = TEST_CASES.lock().unwrap();

    for st in script_tests {
        let id = Uuid::new_v4().to_string();
        let test_case = TestCase {
            id: id.clone(),
            name: st.name,
            description: st.description.unwrap_or_default(),
            command: st.command,
            expected_response: st.expected_response,
            timeout_ms: st.timeout_ms.unwrap_or(5000),
            status: "idle".to_string(),
        };
        cases.insert(id, test_case.clone());
        created.push(test_case);
    }

    Ok(created)
}

#[derive(Debug, Deserialize)]
struct ScriptTestCase {
    name: String,
    description: Option<String>,
    command: String,
    expected_response: String,
    timeout_ms: Option<u64>,
}

#[tauri::command]
pub fn generate_report(format: String) -> Result<String, String> {
    let report = LAST_REPORT
        .lock()
        .unwrap()
        .clone()
        .ok_or("没有可用的测试报告，请先运行测试".to_string())?;

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&report)
            .map_err(|e| format!("JSON序列化失败: {}", e)),
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            wtr.write_record(&["test_id", "test_name", "passed", "message", "actual_response", "duration_ms"])
                .map_err(|e| format!("CSV写入失败: {}", e))?;

            for result in &report.results {
                wtr.write_record(&[
                    &result.test_id,
                    &result.test_name,
                    &result.passed.to_string(),
                    &result.message,
                    &result.actual_response,
                    &result.duration_ms.to_string(),
                ])
                .map_err(|e| format!("CSV写入失败: {}", e))?;
            }

            let bytes = wtr.into_inner().map_err(|e| format!("CSV序列化失败: {}", e))?;
            Ok(String::from_utf8_lossy(&bytes).to_string())
        }
        "text" => {
            let mut text = String::new();
            text.push_str(&format!("测试报告 - {}\n", report.timestamp));
            text.push_str(&format!(
                "总计: {} | 通过: {} | 失败: {} | 耗时: {}ms\n",
                report.total_tests, report.passed, report.failed, report.duration_ms
            ));
            text.push_str(&"=".repeat(60));
            text.push('\n');

            for result in &report.results {
                let status = if result.passed { "✓ PASS" } else { "✗ FAIL" };
                text.push_str(&format!(
                    "[{}] {} - {} ({}ms)\n",
                    status, result.test_name, result.message, result.duration_ms
                ));
                if !result.passed && !result.actual_response.is_empty() {
                    text.push_str(&format!("  实际响应: {}\n", result.actual_response));
                }
            }

            Ok(text)
        }
        _ => Err("不支持的报告格式，请使用 json、csv 或 text".to_string()),
    }
}

#[tauri::command]
pub fn get_last_report() -> Result<Option<TestReport>, String> {
    Ok(LAST_REPORT.lock().unwrap().clone())
}

#[tauri::command]
pub fn clear_test_cases() -> Result<String, String> {
    TEST_CASES.lock().unwrap().clear();
    Ok("测试用例已清空".to_string())
}
