use fs::write;
use std::collections::HashMap;
use std::fs;
use std::fs::read_to_string;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use crate::models::user::User;

const SESSION_FILE: &str = ".session_db";
const CURRENT_SESSION_FILE: &str = ".session";

#[derive(Default, Serialize, Deserialize)]
pub struct SessionDB {
    sessions: HashMap<String, Session>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub user: User,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
}


fn load_db() -> SessionDB {
    match read_to_string(SESSION_FILE) {
        Ok(data) => from_str(&data).unwrap_or_default(),
        Err(_) => SessionDB::default(),
    }
}

fn save_db(db: &SessionDB) {
    let data = to_string_pretty(db).unwrap();
    write(SESSION_FILE, data).expect("Unable to write session file");
}

pub fn save_session_with_user(session_id: &str, session: Session) {
    let mut db = load_db();
    db.sessions.insert(session_id.to_string(), session.clone());
    save_db(&db);
    
    write(CURRENT_SESSION_FILE, session_id).expect("Unable to write session file");
}

pub fn get_current_user() -> Option<Session> {
    let session_id = read_to_string(CURRENT_SESSION_FILE).ok()?;
    let db = load_db();
    db.sessions.get(session_id.trim()).cloned()
}

pub fn get_current_session_id() -> Option<String> {
    read_to_string(CURRENT_SESSION_FILE).ok()
        .map(|s| s.trim().to_string())
}

pub fn clear_session() {
    fs::remove_file(CURRENT_SESSION_FILE).ok();
}


pub fn update_session(session_id: &str, session: Session) {
    let mut db = load_db();
    db.sessions.insert(session_id.to_string(), session);
    save_db(&db);
}