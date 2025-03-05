use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use arrayref::array_ref;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64, Engine as _};
use ring::aead::{LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::{aead, pbkdf2, rand};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum CryptoError {
    #[error("Failed to hash password: {0}")]
    HashingError(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Verification failed: {0}")]
    VerifyError(String),
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
    let salt_input = format!("salt-prefix-{}", password);

    let salt = ring::digest::digest(&ring::digest::SHA256, salt_input.as_bytes());
    let salt_bytes = &salt.as_ref()[0..SALT_LEN];

    let mut key = [0u8; KEY_LEN];
    let iterations = 100_000;

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations.try_into().unwrap(),
        salt_bytes,
        password.as_bytes(),
        &mut key,
    );

    let mut result = Vec::with_capacity(SALT_LEN + KEY_LEN);
    result.extend_from_slice(salt_bytes);
    result.extend_from_slice(&key);

    Ok(BASE64.encode(result))
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, CryptoError> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| CryptoError::HashingError(format!("Invalid password hash: {}", e)))?;

    let new_argon2 = Argon2::default();

    match new_argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn process_encryption_key(
    encryption_key: &str,
    for_encryption: bool,
) -> Result<LessSafeKey, CryptoError> {
    if encryption_key.is_empty() {
        let action = if for_encryption {
            "Encryption"
        } else {
            "Decryption"
        };
        return Err(CryptoError::EncryptionError(
            format!("{} key cannot be empty", action).into(),
        ));
    }

    let key_bytes = BASE64.decode(encryption_key.as_bytes()).map_err(|e| {
        let action = if for_encryption {
            "Encryption"
        } else {
            "Decryption"
        };
        CryptoError::EncryptionError(format!("Invalid {} key: {}", action, e))
    })?;

    let actual_key = &key_bytes[key_bytes.len() - KEY_LEN..];

    let unbound_key = UnboundKey::new(&AES_256_GCM, actual_key).map_err(|_| {
        let action = if for_encryption {
            "Encryption"
        } else {
            "Decryption"
        };
        CryptoError::EncryptionError(format!("Failed to create {} key", action).into())
    })?;

    Ok(LessSafeKey::new(unbound_key))
}

pub fn encrypt(input: &str, user_id: &str, encryption_key: &str) -> Result<String, CryptoError> {
    if input.is_empty() {
        return Err(CryptoError::EncryptionError("Input cannot be empty".into()));
    }

    if user_id.is_empty() {
        return Err(CryptoError::EncryptionError(
            "User ID cannot be empty".into(),
        ));
    }

    let key = process_encryption_key(encryption_key, true)?;

    let rng = rand::SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rand::SecureRandom::fill(&rng, &mut nonce_bytes)
        .map_err(|_| CryptoError::EncryptionError("Failed to generate nonce".into()))?;

    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = input.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| CryptoError::EncryptionError("Encryption failed!".into()))?;

    let mut result = Vec::with_capacity(nonce_bytes.len() + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);

    Ok(BASE64.encode(result))
}

pub fn decrypt(encrypted_data: &str, encryption_key: &str) -> Result<String, CryptoError> {
    if encrypted_data.is_empty() {
        return Err(CryptoError::DecryptionError(
            "Encrypted data cannot be empty".into(),
        ));
    }

    let encrypted_bytes = BASE64
        .decode(encrypted_data.as_bytes())
        .map_err(|e| CryptoError::DecryptionError(format!("Invalid encrypted data: {}", e)))?;

    if encrypted_bytes.len() <= 12 {
        return Err(CryptoError::DecryptionError(
            "Encrypted data is too short".into(),
        ));
    }

    let key = process_encryption_key(encryption_key, false)?;

    let nonce_bytes = &encrypted_bytes[0..12];
    let nonce = Nonce::assume_unique_for_key(*array_ref![nonce_bytes, 0, 12]);

    let mut ciphertext = encrypted_bytes[12..].to_vec();

    let plaintext = key
        .open_in_place(nonce, aead::Aad::empty(), &mut ciphertext)
        .map_err(|_| CryptoError::DecryptionError("Decryption failed".into()))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|_| CryptoError::DecryptionError("Decrypted data is not valid UTF-8".into()))
}
