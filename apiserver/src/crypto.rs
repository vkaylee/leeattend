use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce}; // Or `Aes128Gcm`
use rand::rngs::OsRng;
use rand::RngCore;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Missing active key")]
    MissingActiveKey,
    #[error("Key version '{0}' not found")]
    KeyNotFound(String),
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid key length")]
    InvalidKeyLength,
}

#[derive(Clone)]
pub struct CryptoService {
    keys: HashMap<String, [u8; 32]>,
    active_key_id: String,
}

impl CryptoService {
    /// Creates a new CryptoService with a predefined active key
    pub fn new(
        active_key_id: String,
        keys: HashMap<String, [u8; 32]>,
    ) -> Result<Self, CryptoError> {
        if !keys.contains_key(&active_key_id) {
            return Err(CryptoError::MissingActiveKey);
        }
        Ok(Self {
            keys,
            active_key_id,
        })
    }

    /// Helper to parse a JSON configuration string of keys
    pub fn from_json(config_json: &str) -> Result<Self, CryptoError> {
        let parsed: serde_json::Value =
            serde_json::from_str(config_json).map_err(|_err| CryptoError::InvalidFormat)?;
        let active_key_id = parsed["active_key"]
            .as_str()
            .ok_or(CryptoError::InvalidFormat)?
            .to_string();

        let mut keys = HashMap::new();
        if let Some(keys_obj) = parsed["keys"].as_object() {
            for (k, v) in keys_obj {
                if let Some(key_b64) = v.as_str() {
                    use base64::{engine::general_purpose, Engine as _};
                    let key_bytes = general_purpose::STANDARD
                        .decode(key_b64)
                        .map_err(|_err| CryptoError::InvalidFormat)?;

                    if key_bytes.len() != 32 {
                        return Err(CryptoError::InvalidKeyLength);
                    }
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&key_bytes);
                    keys.insert(k.clone(), arr);
                }
            }
        }
        Self::new(active_key_id, keys)
    }

    /// Encrypts plaintext and returns `version:nonce_b64:ciphertext_b64`
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let key_bytes = self
            .keys
            .get(&self.active_key_id)
            .ok_or(CryptoError::MissingActiveKey)?;
        let cipher = Aes256Gcm::new(key_bytes.into());

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext.as_bytes(),
                    aad: self.active_key_id.as_bytes(),
                },
            )
            .map_err(|_err| CryptoError::EncryptionFailed)?;

        use base64::{engine::general_purpose, Engine as _};
        let b64_nonce = general_purpose::STANDARD.encode(nonce_bytes);
        let b64_ciphertext = general_purpose::STANDARD.encode(ciphertext);

        Ok(format!(
            "{}:{}:{}",
            self.active_key_id, b64_nonce, b64_ciphertext
        ))
    }

    /// Decrypts ciphertext in the format `version:nonce_b64:ciphertext_b64`
    pub fn decrypt(&self, encrypted_payload: &str) -> Result<String, CryptoError> {
        let parts: Vec<&str> = encrypted_payload.split(':').collect();
        if parts.len() != 3 {
            return Err(CryptoError::InvalidFormat);
        }

        let version = parts[0];
        let b64_nonce = parts[1];
        let b64_ciphertext = parts[2];

        let key_bytes = self
            .keys
            .get(version)
            .ok_or_else(|| CryptoError::KeyNotFound(version.to_string()))?;
        let cipher = Aes256Gcm::new(key_bytes.into());

        use base64::{engine::general_purpose, Engine as _};
        let nonce_bytes = general_purpose::STANDARD
            .decode(b64_nonce)
            .map_err(|_err| CryptoError::InvalidFormat)?;
        let ciphertext = general_purpose::STANDARD
            .decode(b64_ciphertext)
            .map_err(|_err| CryptoError::InvalidFormat)?;

        if nonce_bytes.len() != 12 {
            return Err(CryptoError::InvalidFormat);
        }
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext_bytes = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &ciphertext,
                    aad: version.as_bytes(),
                },
            )
            .map_err(|_err| CryptoError::DecryptionFailed)?;

        String::from_utf8(plaintext_bytes).map_err(|_err| CryptoError::DecryptionFailed)
    }
}

/// Hashes an agent token using SHA-256 for secure verification and storage (SOC 2).
pub fn hash_agent_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

