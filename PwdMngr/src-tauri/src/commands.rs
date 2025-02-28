use crate::{models::User, crypto, DatabasePool};
use uuid::Uuid;
use chrono::Utc;
use serde_json::{json, Value as JsonValue};

#[tauri::command]
pub async fn register_user(pool: tauri::State<'_, DatabasePool>, username: String, password: String, confirm_password: String) -> Result<JsonValue, String> {
    if username.trim().is_empty() {
        return Err("Username cannot be empty".into());
    }
    
    if password.trim().is_empty() {
        return Err("Password cannot be empty".into());
    }
    
    if password != confirm_password {
        return Err("Passwords do not match".into());
    }
    
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&*pool.0)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    
    if existing_user.is_some() {
        return Err("Username already exists".into());
    }

    let password_hash = crypto::hash_password(&password)
        .map_err(|e| format!("Password hashing error: {}", e))?;
    
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query("INSERT INTO users (id, username, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(user_id.to_string())
        .bind(&username)
        .bind(&password_hash)
        .bind(now)
        .bind(now)
        .execute(&*pool.0)
        .await
        .map_err(|e| format!("Failed to create user: {}", e))?;

    let encrypted_key = crypto::generate_encryption_key(&password)
        .map_err(|e| format!("Failed to generate encryption key: {}", e))?;
    
    Ok(json!({
        "encKey": encrypted_key,
        "message": "User successfully registered!"
    }))
}