pub mod commands;
pub mod db;
pub mod crypto;
pub mod models;
pub mod user_state;

use commands::{register_user, login_user, logout_user};

use std::sync::{Arc, Mutex};
use sqlx::SqlitePool;
use tauri::{async_runtime, Manager, generate_handler};

#[derive(Clone)]
pub struct DatabasePool(Arc<SqlitePool>);

#[derive(Default, Clone)]
pub struct UserState(pub Arc<Mutex<Option<String>>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();
            
            let pool = async_runtime::block_on(async {
                db::establish_connection(&app_handle).await.expect("Failed to establish database connection")
            });
            
            app.manage(DatabasePool(Arc::new(pool)));
            app.manage(UserState::default());
            
            Ok(())
        })
        .invoke_handler(generate_handler![
            register_user,
            login_user,
            logout_user
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}