

#[derive(Debug, serde::Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub aud: String,
    pub exp: u64,
    pub iss: String,
    pub nonce: Option<String>,
}