//! Cryptographic utilities for secure storage of API keys and sensitive data
//!
//! This module provides secure encryption and decryption functionality using AES-256-GCM.
//! Keys are never logged and decryption only happens inside worker task memory.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use serde::{Deserialize, Serialize};

/// Example implementations showing secure usage patterns
pub mod examples;

/// Error types for cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Invalid key format")]
    InvalidKey,
    
    #[error("Invalid ciphertext bundle")]
    InvalidCiphertextBundle,
}

/// Encrypted data bundle containing ciphertext and nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiphertextBundle {
    /// Encrypted data
    pub ciphertext: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    pub nonce: Vec<u8>,
}

impl CiphertextBundle {
    /// Create a new CiphertextBundle
    pub fn new(ciphertext: Vec<u8>, nonce: Vec<u8>) -> Result<Self, CryptoError> {
        if nonce.len() != 12 {
            return Err(CryptoError::InvalidCiphertextBundle);
        }
        Ok(Self { ciphertext, nonce })
    }
    
    /// Get the nonce as a fixed-size array
    pub fn nonce_array(&self) -> Result<[u8; 12], CryptoError> {
        self.nonce
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidCiphertextBundle)
    }
}

/// Encrypt plaintext using AES-256-GCM with the provided master key
///
/// # Arguments
/// * `master_key` - 32-byte master key for encryption
/// * `plaintext` - Data to encrypt
///
/// # Returns
/// * `CiphertextBundle` containing encrypted data and nonce
///
/// # Security Notes
/// * Uses AES-256-GCM for authenticated encryption
/// * Generates a random 12-byte nonce for each encryption
/// * Master key is never logged or stored
pub fn encrypt(master_key: &[u8; 32], plaintext: &[u8]) -> CiphertextBundle {
    let cipher = Aes256Gcm::new_from_slice(master_key)
        .expect("Invalid master key size");
    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("Encryption should never fail with valid key and nonce");
    
    CiphertextBundle::new(ciphertext, nonce.to_vec())
        .expect("Generated nonce should always be valid size")
}

/// Decrypt ciphertext using AES-256-GCM with the provided master key
///
/// # Arguments
/// * `master_key` - 32-byte master key for decryption
/// * `bundle` - CiphertextBundle containing encrypted data and nonce
///
/// # Returns
/// * Decrypted plaintext as Vec<u8>
///
/// # Security Notes
/// * Verifies authentication tag during decryption
/// * Returns error if data has been tampered with
/// * Master key is never logged or stored
/// * Decrypted data should only exist in worker task memory
pub fn decrypt(master_key: &[u8; 32], bundle: &CiphertextBundle) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(master_key)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    
    let nonce_array = bundle.nonce_array()?;
    let nonce = Nonce::from_slice(&nonce_array);
    
    let plaintext = cipher
        .decrypt(nonce, bundle.ciphertext.as_ref())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    
    Ok(plaintext)
}

/// Generate a secure random 32-byte master key
pub fn generate_master_key() -> [u8; 32] {
    use rand::RngCore;
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Securely store encrypted API key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedApiKey {
    /// Service name (e.g., "pocketbase", "fathom", "loom")
    pub service: String,
    /// Key identifier or description
    pub key_id: String,
    /// Encrypted API key
    pub encrypted_key: CiphertextBundle,
    /// Timestamp when key was encrypted
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Optional expiration time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl EncryptedApiKey {
    /// Create a new encrypted API key entry
    pub fn new(
        service: String,
        key_id: String,
        api_key: &str,
        master_key: &[u8; 32],
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        let encrypted_key = encrypt(master_key, api_key.as_bytes());
        
        Self {
            service,
            key_id,
            encrypted_key,
            created_at: chrono::Utc::now(),
            expires_at,
        }
    }
    
    /// Decrypt the API key (should only be done in worker task memory)
    pub fn decrypt_key(&self, master_key: &[u8; 32]) -> Result<String, CryptoError> {
        let plaintext = decrypt(master_key, &self.encrypted_key)?;
        String::from_utf8(plaintext)
            .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
    }
    
    /// Check if the key has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let master_key = generate_master_key();
        let plaintext = b"test-api-key-12345";
        
        // Encrypt
        let bundle = encrypt(&master_key, plaintext);
        
        // Decrypt
        let decrypted = decrypt(&master_key, &bundle).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_encrypt_different_nonces() {
        let master_key = generate_master_key();
        let plaintext = b"same-plaintext";
        
        let bundle1 = encrypt(&master_key, plaintext);
        let bundle2 = encrypt(&master_key, plaintext);
        
        // Different nonces should produce different ciphertext
        assert_ne!(bundle1.nonce, bundle2.nonce);
        assert_ne!(bundle1.ciphertext, bundle2.ciphertext);
        
        // But both should decrypt to the same plaintext
        let decrypted1 = decrypt(&master_key, &bundle1).unwrap();
        let decrypted2 = decrypt(&master_key, &bundle2).unwrap();
        assert_eq!(decrypted1, decrypted2);
        assert_eq!(plaintext, decrypted1.as_slice());
    }
    
    #[test]
    fn test_tamper_detection() {
        let master_key = generate_master_key();
        let plaintext = b"secret-data";
        
        let mut bundle = encrypt(&master_key, plaintext);
        
        // Tamper with ciphertext
        bundle.ciphertext[0] ^= 0x01;
        
        // Decryption should fail
        assert!(decrypt(&master_key, &bundle).is_err());
    }
    
    #[test]
    fn test_wrong_key_fails() {
        let master_key1 = generate_master_key();
        let master_key2 = generate_master_key();
        let plaintext = b"secret-data";
        
        let bundle = encrypt(&master_key1, plaintext);
        
        // Wrong key should fail decryption
        assert!(decrypt(&master_key2, &bundle).is_err());
    }
    
    #[test]
    fn test_encrypted_api_key() {
        let master_key = generate_master_key();
        let api_key = "sk-1234567890abcdef";
        
        let encrypted = EncryptedApiKey::new(
            "test-service".to_string(),
            "main-key".to_string(),
            api_key,
            &master_key,
            None,
        );
        
        // Decrypt and verify
        let decrypted = encrypted.decrypt_key(&master_key).unwrap();
        assert_eq!(api_key, decrypted);
        
        // Check not expired
        assert!(!encrypted.is_expired());
    }
    
    #[test]
    fn test_api_key_expiration() {
        let master_key = generate_master_key();
        let api_key = "sk-expired";
        
        // Create key that expired 1 hour ago
        let expired_time = chrono::Utc::now() - chrono::Duration::hours(1);
        
        let encrypted = EncryptedApiKey::new(
            "test-service".to_string(),
            "expired-key".to_string(),
            api_key,
            &master_key,
            Some(expired_time),
        );
        
        // Should be expired
        assert!(encrypted.is_expired());
    }
    
    #[test]
    fn test_invalid_nonce_size() {
        let invalid_bundle = CiphertextBundle {
            ciphertext: vec![1, 2, 3],
            nonce: vec![1, 2, 3], // Invalid size (should be 12 bytes)
        };
        
        assert!(invalid_bundle.nonce_array().is_err());
    }
    
    #[test]
    fn test_empty_plaintext() {
        let master_key = generate_master_key();
        let empty_plaintext = b"";
        
        let bundle = encrypt(&master_key, empty_plaintext);
        let decrypted = decrypt(&master_key, &bundle).unwrap();
        
        assert_eq!(empty_plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_large_plaintext() {
        let master_key = generate_master_key();
        let large_plaintext = vec![42u8; 10000]; // 10KB of data
        
        let bundle = encrypt(&master_key, &large_plaintext);
        let decrypted = decrypt(&master_key, &bundle).unwrap();
        
        assert_eq!(large_plaintext, decrypted);
    }
}
