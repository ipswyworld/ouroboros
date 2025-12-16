#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod wallet;
mod commands;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize wallet storage on startup
            let app_dir = app.path_resolver()
                .app_data_dir()
                .expect("Failed to get app data dir");

            std::fs::create_dir_all(&app_dir)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_wallet,
            commands::import_wallet,
            commands::import_from_key,
            commands::get_wallet_info,
            commands::export_mnemonic,
            commands::get_balance,
            commands::get_microchain_balance,
            commands::send_transaction,
            commands::send_microchain_transaction,
            commands::list_microchains,
            commands::link_to_node,
            commands::get_transaction_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
