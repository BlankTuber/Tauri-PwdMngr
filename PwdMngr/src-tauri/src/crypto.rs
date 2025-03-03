use argon2::{
    password_hash::{PasswordHasher, SaltString, PasswordVerifier, PasswordHash},
    Argon2,
};
use ring::pbkdf2;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Failed to hash password: {0}")]
    HashingError(String),
    
    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),
}

const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;

pub fn hash_password(password: &str) -> Result<String, CryptoError> {
    let rng = ring::rand::SystemRandom::new();
    let mut salt_bytes = [0u8; SALT_LEN];
    ring::rand::SecureRandom::fill(&rng, &mut salt_bytes)
        .map_err(|_| CryptoError::HashingError("Failed to generate salt".into()))?;
    
    let salt_b64 = BASE64.encode(salt_bytes);
    let salt = SaltString::from_b64(&salt_b64)
        .map_err(|e| CryptoError::HashingError(format!("Invalid salt: {}", e)))?;
    
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::HashingError(e.to_string()))?
        .to_string();
    
    Ok(password_hash)
}

pub fn generate_encryption_key(password: &str) -> Result<String, CryptoError> {
    let rng = ring::rand::SystemRandom::new();
    let mut salt = [0u8; SALT_LEN];
    ring::rand::SecureRandom::fill(&rng, &mut salt)
        .map_err(|_| CryptoError::KeyDerivationError("Failed to generate salt".into()))?;
    
    let mut key = [0u8; KEY_LEN];
    let iterations = 100_000;
    
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations.try_into().unwrap(),
        &salt,
        password.as_bytes(),
        &mut key,
    );
    
    let mut result = Vec::with_capacity(SALT_LEN + KEY_LEN);
    result.extend_from_slice(&salt);
    result.extend_from_slice(&key);
    
    Ok(BASE64.encode(result))
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, CryptoError> {
    let parsed_hash = PasswordHash::new(stored_hash).map_err(|e| CryptoError::HashingError(format!("Invalid password hash: {}", e)))?;

    let new_argon2 = Argon2::default();

    match new_argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }

}