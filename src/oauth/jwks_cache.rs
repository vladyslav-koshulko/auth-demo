use std::time::Instant;
use crate::oauth::jwks::Jwks;


#[derive(Clone)]
pub struct JwksCache {
    pub jwks: Option<Jwks>,
    pub expires_at: Option<Instant>,
}

impl JwksCache {
    pub fn new() -> Self {
        Self {
            jwks: None,
            expires_at: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.expires_at.is_some() && self.expires_at.unwrap() > Instant::now()
    }

    pub fn set(&mut self, jwks: Jwks, ttl_secs: u64) {
        self.jwks = Some(jwks);
        self.expires_at = Some(Instant::now() + std::time::Duration::from_secs(ttl_secs));
    }
}