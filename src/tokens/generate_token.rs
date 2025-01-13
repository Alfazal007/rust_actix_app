use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub username: String,
    pub exp: usize,
}

pub fn generate_token(
    username: &str,
    user_id: i32,
    secret: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let claims = Claims {
        user_id,
        username: username.to_string(),
        exp: (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 86400) as usize,
    };
    let header = Header::default();
    let token = encode(&header, &claims, &EncodingKey::from_secret(secret.as_ref()))?;
    Ok(token)
}
