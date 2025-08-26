use anyhow::Result;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::{SaltString, PasswordHash}};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use rand_core::OsRng; // <- not rand::rngs::OsRng


#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn hash_password(plain: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?  // map argon2 error to anyhow
        .to_string();
    Ok(hash)
}

pub fn verify_password(hash: &str, plain: &str) -> bool {
    PasswordHash::new(hash)
        .and_then(|ph| Argon2::default().verify_password(plain.as_bytes(), &ph))
        .is_ok()
}

pub fn make_jwt_3min(secret: &str, user_id: &str) -> Result<String> {
    let exp = (chrono::Utc::now().timestamp() + 180) as usize; // 3 minutes
    let claims = Claims { sub: user_id.to_string(), exp };
    Ok(encode(&Header::new(Algorithm::HS256), &claims,
              &EncodingKey::from_secret(secret.as_bytes()))?)
}

pub fn parse_jwt(secret: &str, token: &str) -> Result<String> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(data.claims.sub)
}
