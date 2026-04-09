use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn generate_session_id() -> String {
    generate_random_string(32)
}