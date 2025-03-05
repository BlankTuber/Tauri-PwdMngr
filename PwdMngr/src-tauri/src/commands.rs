use crate::{crypto, models::PasswordRecord, models::User, user_state, DatabasePool, UserState};
use chrono::Utc;
use serde_json::{json, Value as JsonValue};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn register_user(
    pool: State<'_, DatabasePool>,
    user_state: State<'_, UserState>,
    username: String,
    password: String,
    confirm_password: String,
) -> Result<JsonValue, String> {
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

    let password_hash =
        crypto::hash_password(&password).map_err(|e| format!("Password hashing error: {}", e))?;

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
pub async fn login_user(
    pool: State<'_, DatabasePool>,
    user_state: State<'_, UserState>,
    username: String,
    password: String,
) -> Result<JsonValue, String> {
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
pub async fn new_password(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    website: String,
    username: String,
    password: String,
    website_url: Option<String>,
    notes: Option<String>,
    enc_key: String,
) -> Result<JsonValue, String> {
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

    let encrypted_username = crypto::encrypt(&username, &user_id, &enc_key)
        .map_err(|e| format!("Failed to encrypt username: {}", e))?;

    let encrypted_password = crypto::encrypt(&password, &user_id, &enc_key)
        .map_err(|e| format!("Failed to encrypt password: {}", e))?;

    sqlx::query("INSERT INTO passwords (id, user_id, website, website_url, encrypted_username, encrypted_password, notes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&password_id)
        .bind(&user_id)
        .bind(&website)
        .bind(&website_url)
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

#[tauri::command]
pub async fn get_passwords(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    page: i32,
    enc_key: String,
) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not authenticated".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();
    let page_size = 6;
    let offset = (page - 1) * page_size;

    let total_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM passwords WHERE user_id = ?")
            .bind(&user_id)
            .fetch_one(&*pool.0)
            .await
            .map_err(|e| format!("Failed to count passwords: {}", e))?;

    let passwords = sqlx::query_as::<_, PasswordRecord>(
        "
        SELECT id, website, website_url, encrypted_username, encrypted_password, notes, updated_at 
        FROM passwords 
        WHERE user_id = ? 
        ORDER BY updated_at DESC
        LIMIT ? OFFSET ?
    ",
    )
    .bind(&user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&*pool.0)
    .await
    .map_err(|e| format!("Failed to fetch passwords: {}", e))?;

    let total_pages = (total_count as f64 / page_size as f64).ceil() as i32;

    let password_list: Vec<JsonValue> = passwords
        .into_iter()
        .map(|password| {
            json!({
                "id": password.id,
                "website": password.website,
                "website_url": password.website_url,
                "username": crypto::decrypt(&password.encrypted_username, &enc_key),
                "password": crypto::decrypt(&password.encrypted_password, &enc_key),
                "notes": password.notes,
                "updated_at": password.updated_at.to_rfc3339()
            })
        })
        .collect();

    Ok(json!({
        "passwords": password_list,
        "total": total_count,
        "page": page,
        "total_pages": total_pages
    }))
}
