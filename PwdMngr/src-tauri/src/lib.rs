pub mod commands;
pub mod crypto;
pub mod db;
pub mod models;
pub mod user_state;

use commands::{get_passwords, login_user, logout_user, new_password, register_user};

use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tauri::{async_runtime, generate_handler, Manager};

#[derive(Clone)]
pub struct DatabasePool(Arc<SqlitePool>);

#[derive(Default, Clone)]
pub struct UserState(pub Arc<Mutex<Option<String>>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();

            let pool = async_runtime::block_on(async {
                db::establish_connection(&app_handle)
                    .await
                    .expect("Failed to establish database connection")
            });

            app.manage(DatabasePool(Arc::new(pool)));
            app.manage(UserState::default());

            Ok(())
        })
        .invoke_handler(generate_handler![
            register_user,
            login_user,
            logout_user,
            new_password,
            get_passwords
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
