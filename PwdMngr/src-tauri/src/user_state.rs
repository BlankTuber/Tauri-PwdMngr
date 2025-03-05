use crate::UserState;
use tauri::State;

pub fn set_current_user(state: &State<UserState>, user_id: String) {
    *state.0.lock().unwrap() = Some(user_id);
}

pub fn get_current_user(state: &State<UserState>) -> Option<String> {
    state.0.lock().unwrap().clone()
}

pub fn clear_current_user(state: &State<UserState>) {
    *state.0.lock().unwrap() = None;
}

pub fn require_authentication(state: &State<UserState>) -> Result<String, String> {
    match get_current_user(state) {
        Some(user_id) => Ok(user_id),
        None => Err("Not authenticated".to_string()),
    }
}

pub fn require_no_authentication(state: &State<UserState>) -> Result<(), String> {
    match get_current_user(state) {
        Some(_) => Err("Already authenticated".to_string()),
        None => Ok(()),
    }
}
