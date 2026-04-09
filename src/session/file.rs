use std::collections::HashMap;
use std::fs::read_to_string;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use crate::models::user::User;

const SESSION_FILE: &str = ".session_db";
const CURRENT_SESSION_FILE: &str = ".session";

#[derive(Default, Serialize, Deserialize)]
pub struct SessionDB {
    sessions: HashMap<String, User>,
}


fn load_db() -> SessionDB {
    match read_to_string(SESSION_FILE) {
        Ok(data) => from_str(&data).unwrap_or_default(),
        Err(_) => SessionDB::default(),
    }
}

fn save_db(db: &SessionDB) {
    let data = to_string_pretty(db).unwrap();
    std::fs::write(SESSION_FILE, data).expect("Unable to write session file");
}

pub fn save_session_with_user(session_id: &str, user: &User) {
    let mut db = load_db();
    db.sessions.insert(session_id.to_string(), user.clone());
    save_db(&db);
    
    std::fs::write(CURRENT_SESSION_FILE, session_id).expect("Unable to write session file");
}

pub fn get_current_user() -> Option<User> {
    let session_id = std::fs::read_to_string(CURRENT_SESSION_FILE).ok()?;
    let db = load_db();
    db.sessions.get(session_id.trim()).cloned()
}

pub fn clear_session() {
    std::fs::remove_file(CURRENT_SESSION_FILE).ok();
}
