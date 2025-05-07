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
pub async fn get_all_passwords_for_export(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    enc_key: String
) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not Authenticated".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();

    let passwords = sqlx::query_as::<_, PasswordRecord>(
        "SELECT id, website, website_url, encrypted_username, encrypted_password, notes, updated_at
        FROM passwords
        WHERE user_id = ?
        ORDER BY website ASC"
    ).bind(&user_id)
    .fetch_all(&*pool.0)
    .await
    .map_err(|e| format!("Failed to fetch passwords: {}", e))?;

    let password_list: Vec<JsonValue> = passwords.into_iter().map(|password| {
        let username = crypto::decrypt(&password.encrypted_username, &enc_key).unwrap_or_else(|_| "Error decrypting username".to_string());

        let decrypted_password = crypto::decrypt(&password.encrypted_password, &enc_key).unwrap_or_else(|_| "Error decrypting username".to_string());

        json!({
            "id": password.id,
            "website": password.website,
            "website_url": password.website_url,
            "username": username,
            "password": decrypted_password,
            "notes": password.notes,
            "updated_at": password.updated_at.to_rfc3339()
        })
    })
    .collect();

    Ok(json!({
        "passwords": password_list,
        "count": password_list.len()
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


#[tauri::command]
pub async fn get_password_details(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    id: String,
    enc_key: String,
) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not authenticated".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();

    let password = sqlx::query_as::<_, PasswordRecord>(
        "
        SELECT id, website, website_url, encrypted_username, encrypted_password, notes, updated_at 
        FROM passwords 
        WHERE id = ? AND user_id = ?
    ",
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&*pool.0)
    .await
    .map_err(|e| format!("Failed to fetch password: {}", e))?;

    match password {
        Some(pwd) => {
            Ok(json!({
                "id": pwd.id,
                "website": pwd.website,
                "website_url": pwd.website_url,
                "username": crypto::decrypt(&pwd.encrypted_username, &enc_key),
                "password": crypto::decrypt(&pwd.encrypted_password, &enc_key),
                "notes": pwd.notes,
                "updated_at": pwd.updated_at.to_rfc3339()
            }))
        }
        None => Err("Password not found".into()),
    }
}

#[tauri::command]
pub async fn update_password(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    id: String,
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

    // Check if password exists and belongs to the user
    let existing_password = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM passwords WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&*pool.0)
    .await
    .map_err(|e| format!("Failed to check password: {}", e))?;

    if existing_password == 0 {
        return Err("Password not found or you don't have permission to edit it".into());
    }

    let encrypted_username = crypto::encrypt(&username, &user_id, &enc_key)
        .map_err(|e| format!("Failed to encrypt username: {}", e))?;

    let encrypted_password = crypto::encrypt(&password, &user_id, &enc_key)
        .map_err(|e| format!("Failed to encrypt password: {}", e))?;

    sqlx::query(
        "UPDATE passwords SET 
        website = ?, 
        website_url = ?, 
        encrypted_username = ?, 
        encrypted_password = ?, 
        notes = ?, 
        updated_at = ? 
        WHERE id = ? AND user_id = ?"
    )
    .bind(&website)
    .bind(&website_url)
    .bind(&encrypted_username)
    .bind(&encrypted_password)
    .bind(&notes)
    .bind(now)
    .bind(&id)
    .bind(&user_id)
    .execute(&*pool.0)
    .await
    .map_err(|e| format!("Failed to update password: {}", e))?;

    Ok(json!({
        "message": "Password successfully updated!"
    }))
}

#[tauri::command]
pub async fn delete_password(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    id: String,
) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not authenticated".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();

    // Check if password exists and belongs to the user
    let existing_password = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM passwords WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&*pool.0)
    .await
    .map_err(|e| format!("Failed to check password: {}", e))?;

    if existing_password == 0 {
        return Err("Password not found or you don't have permission to delete it".into());
    }

    sqlx::query("DELETE FROM passwords WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&*pool.0)
        .await
        .map_err(|e| format!("Failed to delete password: {}", e))?;

    Ok(json!({
        "message": "Password successfully deleted!"
    }))
}

#[tauri::command]
pub async fn search_passwords(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    search_term: String,
    page: i32,
    enc_key: String,
) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not authenticated".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();
    
    // If search term is empty, return all passwords
    if search_term.trim().is_empty() {
        return get_passwords(user_state, pool, page, enc_key).await;
    }

    // Use SQL LIKE for case-insensitive search with wildcards
    let search_pattern = format!("%{}%", search_term);
    let page_size = 6; // Should match the PAGE_SIZE in JavaScript
    let offset = (page - 1) * page_size;

    // First get the total count of matching items for pagination
    let total_count = sqlx::query_scalar::<_, i64>(
        "
        SELECT COUNT(*) 
        FROM passwords 
        WHERE user_id = ? 
        AND (
            website LIKE ? 
            OR website_url LIKE ? 
            OR notes LIKE ?
        )
        "
    )
    .bind(&user_id)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_one(&*pool.0)
    .await
    .map_err(|e| format!("Failed to count search results: {}", e))?;

    // Get paginated search results
    let passwords = sqlx::query_as::<_, PasswordRecord>(
        "
        SELECT id, website, website_url, encrypted_username, encrypted_password, notes, updated_at 
        FROM passwords 
        WHERE user_id = ? 
        AND (
            website LIKE ? 
            OR website_url LIKE ? 
            OR notes LIKE ?
        )
        ORDER BY updated_at DESC
        LIMIT ? OFFSET ?
        "
    )
    .bind(&user_id)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&*pool.0)
    .await
    .map_err(|e| format!("Failed to search passwords: {}", e))?;

    let total_pages = (total_count as f64 / page_size as f64).ceil() as i32;

    // Process results to handle encrypted fields and build response
    let mut password_list: Vec<JsonValue> = Vec::new();
    
    for password in passwords {
        // Decrypt the fields
        let username = crypto::decrypt(&password.encrypted_username, &enc_key)
            .unwrap_or_else(|_| "Error decoding username".to_string());
        
        let decrypted_password = crypto::decrypt(&password.encrypted_password, &enc_key)
            .unwrap_or_else(|_| "Error decoding password".to_string());
        
        // Check if username matches search pattern (after decryption)
        let username_match = username.to_lowercase().contains(&search_term.to_lowercase());
        
        // Create the entry
        let entry = json!({
            "id": password.id,
            "website": password.website,
            "website_url": password.website_url,
            "username": json!({"Ok": username}),
            "password": json!({"Ok": decrypted_password}),
            "notes": password.notes,
            "updated_at": password.updated_at.to_rfc3339(),
            "match_type": if username_match { "username" } else { "other" }
        });
        
        password_list.push(entry);
    }
    
    // Sort the list to prioritize username matches
    password_list.sort_by(|a, b| {
        let a_match_type = a["match_type"].as_str().unwrap_or("other");
        let b_match_type = b["match_type"].as_str().unwrap_or("other");
        
        // Username matches come first
        if a_match_type == "username" && b_match_type != "username" {
            std::cmp::Ordering::Less
        } else if a_match_type != "username" && b_match_type == "username" {
            std::cmp::Ordering::Greater
        } else {
            // If same match type, sort by updated_at (newest first)
            let a_date = a["updated_at"].as_str().unwrap_or("");
            let b_date = b["updated_at"].as_str().unwrap_or("");
            b_date.cmp(a_date)
        }
    });

    Ok(json!({
        "passwords": password_list,
        "total": total_count,
        "page": page,
        "total_pages": total_pages,
        "is_search_result": true
    }))
}

#[tauri::command]
pub async fn prepare_passwords_for_export(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    enc_key: String,
    selected_ids: Vec<String>,
) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not authenticated".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();

    // Prepare the query with an "IN" clause for selected passwords
    let query = format!(
        "SELECT id, website, website_url, encrypted_username, encrypted_password, notes, updated_at 
         FROM passwords 
         WHERE user_id = ? {}
         ORDER BY website ASC",
        if !selected_ids.is_empty() {
            format!("AND id IN ({})", selected_ids.iter().map(|_| "?").collect::<Vec<_>>().join(","))
        } else {
            String::new()
        }
    );

    // Prepare the query
    let mut query_builder = sqlx::query_as::<_, PasswordRecord>(&query)
        .bind(&user_id);
    
    // Bind all selected IDs if filtering
    if !selected_ids.is_empty() {
        for id in &selected_ids {
            query_builder = query_builder.bind(id);
        }
    }
    
    // Execute the query
    let passwords = query_builder
        .fetch_all(&*pool.0)
        .await
        .map_err(|e| format!("Failed to fetch passwords: {}", e))?;

    // Decrypt passwords and prepare for export
    let mut password_data = Vec::new();
    for password in passwords {
        let username = crypto::decrypt(&password.encrypted_username, &enc_key)
            .unwrap_or_else(|_| "Error decoding username".to_string());
        
        let decrypted_password = crypto::decrypt(&password.encrypted_password, &enc_key)
            .unwrap_or_else(|_| "Error decoding password".to_string());
        
        password_data.push(json!({
            "website": password.website,
            "username": username,
            "password": decrypted_password,
            "website_url": password.website_url,
            "notes": password.notes,
        }));
    }

    Ok(json!({
        "success": true,
        "data": password_data,
        "count": password_data.len()
    }))
}

#[tauri::command]
pub async fn import_passwords_from_data(
    user_state: State<'_, UserState>,
    pool: State<'_, DatabasePool>,
    passwords_data: Vec<serde_json::Value>,
    enc_key: String,
) -> Result<JsonValue, String> {
    if user_state::require_authentication(&user_state).is_err() {
        return Err("Not authenticated".into());
    }

    let user_id = user_state::get_current_user(&user_state).unwrap();
    
    if passwords_data.is_empty() {
        return Err("No passwords data provided".into());
    }
    
    // Validate passwords
    let mut valid_passwords = Vec::new();
    for pwd in &passwords_data {
        if let (Some(website), Some(username), Some(password)) = (
            pwd.get("website").and_then(|v| v.as_str()),
            pwd.get("username").and_then(|v| v.as_str()),
            pwd.get("password").and_then(|v| v.as_str()),
        ) {
            if !website.is_empty() && !username.is_empty() && !password.is_empty() {
                valid_passwords.push(pwd);
            }
        }
    }
    
    if valid_passwords.is_empty() {
        return Err("No valid passwords found in the import data".into());
    }
    
    // Import each valid password
    let now = Utc::now();
    let mut success_count = 0;
    let mut error_count = 0;
    
    for pwd in valid_passwords {
        let website = pwd["website"].as_str().unwrap();
        let username = pwd["username"].as_str().unwrap();
        let password = pwd["password"].as_str().unwrap();
        let website_url = pwd.get("website_url").and_then(|v| v.as_str()).map(String::from);
        let notes = pwd.get("notes").and_then(|v| v.as_str()).map(String::from);
        
        // Encrypt sensitive data
        let encrypted_username = match crypto::encrypt(username, &user_id, &enc_key) {
            Ok(value) => value,
            Err(e) => {
                error_count += 1;
                println!("Failed to encrypt username: {}", e);
                continue;
            }
        };
        
        let encrypted_password = match crypto::encrypt(password, &user_id, &enc_key) {
            Ok(value) => value,
            Err(e) => {
                error_count += 1;
                println!("Failed to encrypt password: {}", e);
                continue;
            }
        };
        
        // Generate a new password ID
        let password_id = Uuid::new_v4().to_string();
        
        // Insert into database
        let result = sqlx::query(
            "INSERT INTO passwords (id, user_id, website, website_url, encrypted_username, encrypted_password, notes, created_at, updated_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
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
        .await;
        
        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                println!("Failed to insert password: {}", e);
            }
        }
    }
    
    Ok(json!({
        "success": success_count > 0,
        "success_count": success_count,
        "error_count": error_count,
        "message": format!("Imported {} passwords with {} errors", success_count, error_count)
    }))
}