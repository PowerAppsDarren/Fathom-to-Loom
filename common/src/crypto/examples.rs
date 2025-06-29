//! Examples demonstrating secure API key storage patterns
//! 
//! These examples show how to securely handle API keys in worker tasks,
//! ensuring keys are never logged and only exist in memory during processing.

use super::*;
use std::collections::HashMap;

/// Example: Secure API key manager for worker tasks
pub struct SecureKeyManager {
    master_key: [u8; 32],
    encrypted_keys: HashMap<String, EncryptedApiKey>,
}

impl SecureKeyManager {
    /// Create a new key manager with a randomly generated master key
    pub fn new() -> Self {
        Self {
            master_key: generate_master_key(),
            encrypted_keys: HashMap::new(),
        }
    }
    
    /// Create a key manager with an existing master key (e.g., from secure environment)
    pub fn with_master_key(master_key: [u8; 32]) -> Self {
        Self {
            master_key,
            encrypted_keys: HashMap::new(),
        }
    }
    
    /// Store an API key securely
    pub fn store_api_key(
        &mut self,
        service: String,
        key_id: String,
        api_key: &str,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) {
        let encrypted_key = EncryptedApiKey::new(
            service.clone(),
            key_id.clone(),
            api_key,
            &self.master_key,
            expires_at,
        );
        
        let key = format!("{}:{}", service, key_id);
        self.encrypted_keys.insert(key, encrypted_key);
        
        // API key is immediately dropped from this scope
        // Only encrypted version is stored
    }
    
    /// Retrieve and decrypt an API key (only for use in worker memory)
    /// The returned key should be used immediately and not stored
    pub fn get_api_key(&self, service: &str, key_id: &str) -> Result<String, CryptoError> {
        let key = format!("{}:{}", service, key_id);
        
        match self.encrypted_keys.get(&key) {
            Some(encrypted_key) => {
                if encrypted_key.is_expired() {
                    return Err(CryptoError::DecryptionFailed("API key has expired".to_string()));
                }
                
                // Decrypt only in worker memory
                encrypted_key.decrypt_key(&self.master_key)
            }
            None => Err(CryptoError::DecryptionFailed("API key not found".to_string())),
        }
    }
    
    /// List available keys (without revealing the actual key values)
    pub fn list_keys(&self) -> Vec<(String, String, chrono::DateTime<chrono::Utc>, bool)> {
        self.encrypted_keys
            .values()
            .map(|key| {
                (
                    key.service.clone(),
                    key.key_id.clone(),
                    key.created_at,
                    key.is_expired(),
                )
            })
            .collect()
    }
    
    /// Remove an expired or unused key
    pub fn remove_key(&mut self, service: &str, key_id: &str) -> bool {
        let key = format!("{}:{}", service, key_id);
        self.encrypted_keys.remove(&key).is_some()
    }
    
    /// Export master key for secure backup (handle with extreme care)
    pub fn export_master_key(&self) -> [u8; 32] {
        self.master_key
    }
}

/// Example: Worker task that securely handles API keys
pub async fn example_worker_task(key_manager: &SecureKeyManager) -> Result<(), CryptoError> {
    // Decrypt API key only when needed, inside worker memory
    let fathom_api_key = key_manager.get_api_key("fathom", "analytics")?;
    
    // Use the API key for the required operation
    let result = simulate_api_call(&fathom_api_key).await;
    
    // Key automatically dropped when it goes out of scope
    // No logging or persistent storage of the decrypted key
    
    match result {
        Ok(_) => tracing::info!("API call successful - key details not logged"),
        Err(e) => tracing::error!("API call failed: {} - key details not logged", e),
    }
    
    Ok(())
}

/// Simulate an API call (placeholder)
async fn simulate_api_call(_api_key: &str) -> Result<String, &'static str> {
    // In real implementation, use the API key to make authenticated requests
    // The key should never be logged at any point
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok("API response".to_string())
}

/// Example: Loading keys from environment variables securely
pub fn load_keys_from_env(key_manager: &mut SecureKeyManager) {
    // Load API keys from environment variables (common pattern)
    if let Ok(fathom_key) = std::env::var("FATHOM_API_KEY") {
        key_manager.store_api_key(
            "fathom".to_string(),
            "analytics".to_string(),
            &fathom_key,
            None,
        );
        // fathom_key is dropped here
    }
    
    if let Ok(loom_key) = std::env::var("LOOM_API_KEY") {
        key_manager.store_api_key(
            "loom".to_string(),
            "video".to_string(),
            &loom_key,
            None,
        );
        // loom_key is dropped here
    }
    
    if let Ok(pb_key) = std::env::var("POCKETBASE_API_KEY") {
        key_manager.store_api_key(
            "pocketbase".to_string(),
            "admin".to_string(),
            &pb_key,
            None,
        );
        // pb_key is dropped here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_manager_basic_operations() {
        let mut manager = SecureKeyManager::new();
        
        // Store a key
        manager.store_api_key(
            "test-service".to_string(),
            "main".to_string(),
            "secret-api-key-123",
            None,
        );
        
        // Retrieve the key
        let retrieved = manager.get_api_key("test-service", "main").unwrap();
        assert_eq!(retrieved, "secret-api-key-123");
        
        // List keys
        let keys = manager.list_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].0, "test-service");
        assert_eq!(keys[0].1, "main");
        assert!(!keys[0].3); // not expired
    }
    
    #[test]
    fn test_key_not_found() {
        let manager = SecureKeyManager::new();
        
        let result = manager.get_api_key("nonexistent", "key");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_expired_key() {
        let mut manager = SecureKeyManager::new();
        
        // Create key that expired 1 hour ago
        let expired_time = chrono::Utc::now() - chrono::Duration::hours(1);
        
        manager.store_api_key(
            "test-service".to_string(),
            "expired".to_string(),
            "expired-key",
            Some(expired_time),
        );
        
        // Should fail to retrieve expired key
        let result = manager.get_api_key("test-service", "expired");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_remove_key() {
        let mut manager = SecureKeyManager::new();
        
        manager.store_api_key(
            "test-service".to_string(),
            "temp".to_string(),
            "temp-key",
            None,
        );
        
        // Key should exist
        assert!(manager.get_api_key("test-service", "temp").is_ok());
        
        // Remove key
        assert!(manager.remove_key("test-service", "temp"));
        
        // Key should no longer exist
        assert!(manager.get_api_key("test-service", "temp").is_err());
        
        // Removing again should return false
        assert!(!manager.remove_key("test-service", "temp"));
    }
    
    #[tokio::test]
    async fn test_worker_task_example() {
        let mut manager = SecureKeyManager::new();
        
        manager.store_api_key(
            "fathom".to_string(),
            "analytics".to_string(),
            "test-fathom-key",
            None,
        );
        
        // This should complete without error
        example_worker_task(&manager).await.unwrap();
    }
}
