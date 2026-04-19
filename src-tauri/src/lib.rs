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
            network::receive_network_data,
            network::close_network_connection,
            network::list_network_connections,
            network::list_tcp_server_clients,
            flasher::list_supported_chips,
            flasher::detect_chip,
            flasher::load_firmware,
            flasher::flash_firmware,
            flasher::verify_firmware,
            flasher::erase_chip,
            flasher::get_flash_output,
            flasher::clear_flash_output,
            flasher::cancel_flash,
            flasher::list_serial_ports_for_flasher,
            visualization::create_channel,
            visualization::add_data_point,
            visualization::add_data_points_batch,
            visualization::get_channel_data,
            visualization::get_latest_channel_data,
            visualization::clear_channel_data,
            visualization::export_data,
            visualization::list_channels,
            visualization::remove_channel,
            visualization::update_channel,
            tester::create_test_case,
            tester::update_test_case,
            tester::delete_test_case,
            tester::list_test_cases,
            tester::run_test_case,
            tester::run_all_tests,
            tester::load_test_script,
            tester::generate_report,
            tester::get_last_report,
            tester::clear_test_cases
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
