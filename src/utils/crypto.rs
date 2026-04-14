use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::distributions::Alphanumeric;
use rand::Rng;
use sha2::{Digest, Sha256};

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

pub fn generate_code_verifier() -> String {
    generate_random_string(128)
}

pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(&hash)
}
