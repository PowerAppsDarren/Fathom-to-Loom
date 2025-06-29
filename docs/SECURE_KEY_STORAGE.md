# Secure API Key Storage

This document describes how to securely store and handle API keys using the `common::crypto` module in the Fathom-to-Loom system.

## Overview

The `common::crypto` module provides secure encryption and decryption functionality using AES-256-GCM for storing sensitive API keys. The system ensures that:

- **Keys are never logged** - API keys are never written to logs or console output
- **Decryption only in worker memory** - Keys are only decrypted when needed inside worker task memory
- **Automatic key rotation support** - Keys can have expiration times
- **Tamper detection** - Any modification to encrypted data is detected

## Core Functions

```rust
fn encrypt(master_key: &[u8; 32], plaintext: &[u8]) -> CiphertextBundle;
fn decrypt(master_key: &[u8; 32], bundle: &CiphertextBundle) -> Vec<u8>;
```

## Basic Usage

### 1. Encrypting API Keys

```rust
use common::crypto::{encrypt, generate_master_key};

// Generate a secure master key (store this securely!)
let master_key = generate_master_key();

// Encrypt an API key
let api_key = "sk-your-secret-api-key";
let encrypted_bundle = encrypt(&master_key, api_key.as_bytes());

// api_key is dropped here - no longer in memory
```

### 2. Decrypting API Keys (Worker Context Only)

```rust
use common::crypto::decrypt;

// Inside worker task - decrypt only when needed
let decrypted_bytes = decrypt(&master_key, &encrypted_bundle)?;
let api_key = String::from_utf8(decrypted_bytes)?;

// Use the API key immediately
let response = api_client.call_with_key(&api_key).await?;

// api_key is automatically dropped at end of scope
```

## Secure Key Manager Pattern

For production use, utilize the `SecureKeyManager` from `common::crypto::examples`:

```rust
use common::crypto::examples::SecureKeyManager;

// Initialize key manager
let mut key_manager = SecureKeyManager::new();

// Store API keys securely
key_manager.store_api_key(
    "fathom".to_string(),
    "analytics".to_string(),
    "your-fathom-api-key",
    None, // No expiration
);

key_manager.store_api_key(
    "loom".to_string(), 
    "video".to_string(),
    "your-loom-api-key",
    Some(Utc::now() + Duration::days(30)), // Expires in 30 days
);

// In worker task - retrieve and use key
async fn process_analytics_task(key_manager: &SecureKeyManager) -> Result<(), Error> {
    // Decrypt only when needed
    let fathom_key = key_manager.get_api_key("fathom", "analytics")?;
    
    // Use immediately
    let analytics_data = fathom_client.fetch_data(&fathom_key).await?;
    
    // Process data...
    
    // fathom_key is dropped automatically
    Ok(())
}
```

## Loading Keys from Environment

```rust
use common::crypto::examples::load_keys_from_env;

let mut key_manager = SecureKeyManager::new();

// Securely load from environment variables
load_keys_from_env(&mut key_manager);

// Environment variables are read once and immediately encrypted
// Original environment values are not stored in memory
```

## Security Best Practices

### 1. Master Key Management

- **Never hardcode master keys** - Generate or load from secure key management systems
- **Store master keys securely** - Use environment variables, key vaults, or HSMs
- **Rotate master keys regularly** - Implement key rotation policies

```rust
// Good: Load from secure environment
let master_key_hex = std::env::var("MASTER_KEY")?;
let master_key = hex::decode(&master_key_hex)?;

// Bad: Hardcoded in source
let master_key = [0x01, 0x02, ...]; // Never do this!
```

### 2. Memory Management

- **Minimize key lifetime** - Decrypt keys only when needed
- **Use immediately** - Don't store decrypted keys in variables
- **Trust Rust's drop semantics** - Let keys go out of scope automatically

```rust
// Good: Immediate use
let response = api_call(&key_manager.get_api_key("service", "key")?).await?;

// Bad: Storing decrypted key
let decrypted_key = key_manager.get_api_key("service", "key")?;
// ... other code ...
let response = api_call(&decrypted_key).await?; // Key in memory too long
```

### 3. Logging and Debugging

- **Never log API keys** - Even in debug mode
- **Use key IDs for tracking** - Log service and key_id, never the actual key
- **Implement secure debug modes** - Mask sensitive data in debug output

```rust
// Good: Safe logging
tracing::info!("Using API key for service: {}, key_id: {}", service, key_id);

// Bad: Exposing keys
tracing::debug!("API key: {}", api_key); // Never do this!
```

### 4. Error Handling

- **Don't expose keys in error messages**
- **Use generic error messages for crypto failures**

```rust
// Good: Safe error handling
.map_err(|_| Error::CryptoFailed)?

// Bad: Potential key exposure
.map_err(|e| Error::CryptoFailed(format!("Failed with key: {}", e)))?
```

## Testing

The crypto module includes comprehensive tests covering:

- **Roundtrip encryption/decryption**
- **Tamper detection**
- **Key expiration**
- **Different nonce generation**
- **Invalid input handling**

Run tests with:
```bash
cargo test --package common crypto::
```

## Production Deployment

### Environment Setup

Set these environment variables in production:

```bash
# Master key (32-byte hex string)
MASTER_KEY=your-64-character-hex-string

# API Keys (will be encrypted on first load)
FATHOM_API_KEY=your-fathom-key
LOOM_API_KEY=your-loom-key
POCKETBASE_API_KEY=your-pocketbase-key
```

### Worker Configuration

In your worker initialization:

```rust
use common::crypto::examples::{SecureKeyManager, load_keys_from_env};

async fn initialize_worker() -> Result<SecureKeyManager, Error> {
    let mut key_manager = SecureKeyManager::new();
    
    // Load all API keys from environment
    load_keys_from_env(&mut key_manager);
    
    // Keys are now encrypted in memory
    // Original environment values are dropped
    
    Ok(key_manager)
}
```

## Migration from Plain Text Storage

If you're migrating from plain text API key storage:

1. **Install the crypto module** - Add to your worker dependencies
2. **Initialize SecureKeyManager** - Create new key manager instance  
3. **Encrypt existing keys** - Use `store_api_key()` for each key
4. **Update worker code** - Replace direct key access with `get_api_key()`
5. **Remove plain text storage** - Delete old configuration files
6. **Test thoroughly** - Verify all API calls still work

## Performance Considerations

- **Encryption/Decryption overhead** - Minimal (~microseconds for typical API keys)
- **Memory usage** - Encrypted keys use ~50% more memory than plaintext
- **CPU usage** - AES-GCM is hardware-accelerated on modern CPUs

## Security Audit Checklist

- [ ] Master key stored securely (not in source code)
- [ ] API keys never appear in logs
- [ ] Decryption only happens in worker memory
- [ ] Keys have appropriate expiration times
- [ ] Error messages don't leak sensitive data
- [ ] Environment variables cleared after loading
- [ ] Tests cover all security scenarios
- [ ] Production deployment uses secure key management

## Support and Questions

For questions about secure key storage implementation, please refer to:

- `common/src/crypto.rs` - Core encryption functions
- `common/src/crypto/examples.rs` - Usage patterns and SecureKeyManager
- Tests in both files for comprehensive examples
