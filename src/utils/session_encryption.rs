use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::env;

const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_SIZE] {
    let mut key = [0u8; KEY_SIZE];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let password = env::var("SESSION_KEY").map_err(|_| "SESSION_KEY not set")?;
    let salt: [u8; SALT_SIZE] = rand::random();

    let key = derive_key(&password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Key ini failed: {}", e))?;

    let nonce_bytes: [u8; NONCE_SIZE] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut encrypted_data = Vec::with_capacity(salt.len() + nonce_bytes.len() + ciphertext.len());
    encrypted_data.extend_from_slice(&salt);
    encrypted_data.extend_from_slice(&nonce_bytes);
    encrypted_data.extend_from_slice(&ciphertext);

    Ok(encrypted_data)
}

pub fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if ciphertext.len() < SALT_SIZE + NONCE_SIZE {
        return Err("Invalid ciphertext length".to_string());
    }

    let password = env::var("SESSION_KEY").map_err(|_| "SESSION_KEY not set".to_string())?;

    let salt = &ciphertext[0..SALT_SIZE];
    let nonce = &ciphertext[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &ciphertext[SALT_SIZE + NONCE_SIZE..];

    let key = derive_key(&password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Key init failed: {}", e))?;

    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}
