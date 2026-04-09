

const SESSION_FILE: &str = ".session";

pub fn save_session(session_id: &str) {
    std::fs::write(SESSION_FILE, session_id).expect("Unable to write session file");
}

pub fn load_session() -> Option<String> {
    std::fs::read_to_string(SESSION_FILE).ok()
}

pub fn clear_session() {
    std::fs::remove_file(SESSION_FILE).ok();
}
