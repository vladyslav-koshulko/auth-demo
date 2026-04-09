use std::collections::HashMap;
use crate::models::user::User;

pub struct SessionStore {
    sessions: HashMap<String, User>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn get(&self, session_id: &str) -> Option<&User> {
        self.sessions.get(session_id)
    }

    pub fn insert(&mut self, session_id: String, user: User) {
        self.sessions.insert(session_id, user);
    }

    pub fn remove(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }
}

