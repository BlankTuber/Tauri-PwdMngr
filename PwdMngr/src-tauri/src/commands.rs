use crate::{models::User, crypto, DatabasePool, UserState, user_state};
use uuid::Uuid;
use chrono::Utc;
use serde_json::{json, Value as JsonValue};
use tauri::State;

#[tauri::command]
pub async fn register_user(pool: State<'_, DatabasePool>, user_state: State<'_, UserState>, username: String, password: String, confirm_password: String) -> Result<JsonValue, String> {
    if user_state::require_no_authentication(&user_state).is_err() {
        return Err("Already authenticated".into());
    }
    
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
    
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    sqlx::query("INSERT INTO users (id, username, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&user_id)
        .bind(&username)
        .bind(&password_hash)
        .bind(now)
        .bind(now)
        .execute(&*pool.0)
        .await
        .map_err(|e| format!("Failed to create user: {}", e))?;

    let encrypted_key = crypto::generate_encryption_key(&password)
        .map_err(|e| format!("Failed to generate encryption key: {}", e))?;
    
    user_state::set_current_user(&user_state, user_id);

    Ok(json!({
        "encKey": encrypted_key,
        "message": "User successfully registered!"
    }))
}

#[tauri::command]
pub async fn login_user(pool: State<'_, DatabasePool>, user_state: State<'_, UserState>, username: String, password: String) -> Result<JsonValue, String> {
    if user_state::require_no_authentication(&user_state).is_err() {
        return Err("Already authenticated".into());
    }

    if username.trim().is_empty() {
        return Err("Username cannot be empty".into());
    }
    if password.trim().is_empty() {
        return Err("Password cannot be empty".into());
    }
    
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&*pool.0)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    
    let existing_user = match existing_user {
        Some(user) => user,
        None => return Err("Username does not exist".into()),
    };
    
    let is_correct_pwd = crypto::verify_password(&password, &existing_user.password_hash)
        .map_err(|e| format!("Password verification error: {}", e))?;
    
    if !is_correct_pwd {
        return Err("Password does not match!".into());
    }
    
    let encrypted_key = crypto::generate_encryption_key(&password)
        .map_err(|e| format!("Failed to generate encryption key: {}", e))?;

    user_state::set_current_user(&user_state, existing_user.id.clone());

    Ok(json!({
        "encKey": encrypted_key,
        "message": "Login successful!"
    }))
}

#[tauri::command]
pub async fn logout_user(user_state: State<'_, UserState>) -> Result<JsonValue, String> {
    user_state::clear_current_user(&user_state);
    Ok(json!({
        "message": "Logout successful!"
    }))
}

#[tauri::command]
pub async fn new_password(user_state: State<'_, UserState>, pool: State<'_, DatabasePool>, website: String, web_uri: Option<String>, username: String, password: String, notes: Option<String>) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not authenticated".into());
    }
    
    if website.trim().is_empty() {
        return Err("Website cannot be empty".into());
    }
    
    if username.trim().is_empty() {
        return Err("Username cannot be empty".into());
    }
    
    if password.trim().is_empty() {
        return Err("Password cannot be empty".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();
    let now = Utc::now();
    let password_id = Uuid::new_v4().to_string();

    let encrypted_username = crypto::encrypt(&username, &user_id)
        .map_err(|e| format!("Failed to encrypt username: {}", e))?;

    let encrypted_password = crypto::encrypt(&password, &user_id)
        .map_err(|e| format!("Failed to encrypt password: {}", e))?;

    sqlx::query("INSERT INTO passwords (id, user_id, website, website_url, encrypted_username, encrypted_password, notes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&password_id)
        .bind(&user_id)
        .bind(&website)
        .bind(&web_uri)
        .bind(&encrypted_username)
        .bind(&encrypted_password)
        .bind(&notes)
        .bind(now)
        .bind(now)
        .execute(&*pool.0)
        .await
        .map_err(|e| format!("Failed to create password: {}", e))?;
    
    Ok(json!({
        "message": "Password successfully saved!"
    }))
}