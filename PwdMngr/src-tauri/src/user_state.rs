use crate::UserState;
use tauri::State;

pub fn set_current_user(state: &State<UserState>, user_id: String) {
    println!("Setting current user to {}", user_id);
    *state.0.lock().unwrap() = Some(user_id);
}

pub fn get_current_user(state: &State<UserState>) -> Option<String> {
    println!("Getting current user");
    state.0.lock().unwrap().clone()
}

pub fn clear_current_user(state: &State<UserState>) {
    println!("Clearing current user");
    *state.0.lock().unwrap() = None;
}

pub fn require_authentication(state: &State<UserState>) -> Result<String, String> {
    println!("Requiring authentication");
    match get_current_user(state) {
        Some(user_id) => Ok(user_id),
        None => Err("Not authenticated".to_string())
    }
}

pub fn require_no_authentication(state: &State<UserState>) -> Result<(), String> {
    println!("Requiring no authentication");
    match get_current_user(state) {
        Some(_) => Err("Already authenticated".to_string()),
        None => Ok(())
    }
}