use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, migrate::MigrateDatabase};
use tauri::{AppHandle, Manager};

pub async fn establish_connection(app: &AppHandle) -> Result<SqlitePool, sqlx::Error> {
    let app_dir = app.path().app_data_dir().expect("Failed to get app directory");
    std::fs::create_dir_all(&app_dir).expect("Failed to create directory");

    let db_path = app_dir.join("passwords.db");
    let db_url = format!("sqlite:{}", db_path.display());

    if !sqlx::Sqlite::database_exists(&db_url).await? {
        sqlx::Sqlite::create_database(&db_url).await?
    }

    let pool = SqlitePoolOptions::new().max_connections(5).connect(&db_url).await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;

    Ok(())
}