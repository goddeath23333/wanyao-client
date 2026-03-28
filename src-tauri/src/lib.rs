mod modules;

use modules::{
    system,
    python,
    serial,
    network,
    flasher,
    visualization,
    tester,
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            system::get_system_info,
            python::check_python_version,
            python::run_python_code,
            python::run_python_script,
            serial::list_serial_ports,
            serial::open_serial_port,
            serial::close_serial_port,
            serial::send_serial_data,
            serial::read_serial_data,
            network::create_tcp_client,
            network::create_tcp_server,
            network::create_udp_socket,
            network::send_network_data,
            network::close_network_connection,
            flasher::detect_chip,
            flasher::load_firmware,
            flasher::flash_firmware,
            flasher::verify_firmware,
            flasher::erase_chip,
            visualization::create_channel,
            visualization::add_data_point,
            visualization::get_channel_data,
            visualization::clear_channel_data,
            visualization::export_data,
            tester::create_test_case,
            tester::run_test_case,
            tester::run_all_tests,
            tester::load_test_script,
            tester::generate_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
