use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::sync::OnceLock;

static MASTER_KEY: OnceLock<Vec<u8>> = OnceLock::new();
const NONCE_LEN: usize = 12;

pub fn initialize_encryption(app_handle: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app_handle.store_builder("encryption.json").build()
        .map_err(|e| format!("Failed to build encryption store: {}", e))?;
    
    let key = if let Some(stored_key) = store.get("master_key") {
        let key_str: String = serde_json::from_value(stored_key.clone())
            .map_err(|e| format!("Failed to parse stored key: {}", e))?;
        BASE64.decode(key_str)
            .map_err(|e| format!("Failed to decode key: {}", e))?
    } else {
        let rng = SystemRandom::new();
        let mut key = vec![0u8; 32];
        rng.fill(&mut key)
            .map_err(|_| "Failed to generate random key")?;
        
        let key_str = BASE64.encode(&key);
        store.set("master_key", serde_json::json!(key_str));
        store.save().map_err(|e| format!("Failed to save encryption store: {}", e))?;
        
        key
    };
    
    MASTER_KEY.set(key).map_err(|_| "Failed to set master key")?;
    Ok(())
}

pub fn encrypt_password(password: &str) -> Result<String, String> {
    let key_bytes = MASTER_KEY.get()
        .ok_or("Encryption not initialized")?;
    
    let unbound_key = UnboundKey::new(&AES_256_GCM, key_bytes)
        .map_err(|_| "Failed to create encryption key")?;
    let key = LessSafeKey::new(unbound_key);
    
    // Generate a random nonce
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| "Failed to generate nonce")?;
    
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|_| "Failed to create nonce")?;
    
    // Encrypt the password
    let mut in_out = password.as_bytes().to_vec();
    
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Failed to encrypt password")?;
    
    // Combine nonce and ciphertext+tag
    let mut result = Vec::with_capacity(nonce_bytes.len() + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);
    
    Ok(BASE64.encode(result))
}

pub fn decrypt_password(encrypted: &str) -> Result<String, String> {
    // First check if this is actually an encrypted password
    if encrypted.is_empty() {
        return Ok(String::new());
    }
    
    // Try to decode from base64
    let encrypted_bytes = match BASE64.decode(encrypted) {
        Ok(bytes) => bytes,
        Err(_) => {
            // Not base64, might be plain text - return as is
            println!("Warning: Password doesn't appear to be base64 encoded, returning as-is");
            return Ok(encrypted.to_string());
        }
    };
    
    // Check minimum length (12 byte nonce + 1 byte ciphertext + 16 byte tag = 29 bytes)
    if encrypted_bytes.len() < 29 {
        println!("Warning: Encrypted password too short ({}), returning as-is", encrypted_bytes.len());
        return Ok(encrypted.to_string());
    }
    
    let key_bytes = MASTER_KEY.get()
        .ok_or("Encryption not initialized")?;
    
    let unbound_key = UnboundKey::new(&AES_256_GCM, key_bytes)
        .map_err(|_| "Failed to create decryption key")?;
    let key = LessSafeKey::new(unbound_key);
    
    // Split nonce and ciphertext+tag
    let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(NONCE_LEN);
    
    let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| "Failed to create nonce from bytes")?;
    
    let mut in_out = ciphertext.to_vec();
    
    // Decrypt
    let plaintext = key.open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Failed to decrypt - password may be corrupted: {:?}", e))?;
    
    String::from_utf8(plaintext.to_vec())
        .map_err(|e| format!("Failed to convert decrypted bytes to string: {}", e))
}

pub fn is_encrypted(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    
    // Check if it's valid base64 and has the right length
    if let Ok(decoded) = BASE64.decode(value) {
        // Must have at least 12 byte nonce + 1 byte ciphertext + 16 byte auth tag = 29 bytes
        decoded.len() >= 29
    } else {
        false
    }
}

pub fn migrate_existing_connections(app_handle: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app_handle.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    if let Some(connections_value) = store.get("connections") {
        let mut connections: Vec<serde_json::Value> = serde_json::from_value(connections_value.clone())
            .unwrap_or_default();
        
        let mut updated = false;
        for conn in &mut connections {
            if let Some(password) = conn.get("password").and_then(|p| p.as_str()) {
                if !password.is_empty() && !is_encrypted(password) {
                    println!("Migrating unencrypted password for connection");
                    match encrypt_password(password) {
                        Ok(encrypted) => {
                            conn["password"] = serde_json::json!(encrypted);
                            updated = true;
                        },
                        Err(e) => {
                            println!("Warning: Failed to migrate password: {}", e);
                        }
                    }
                }
            }
        }
        
        if updated {
            store.set("connections", serde_json::json!(connections));
            store.save().map_err(|e| format!("Failed to save migrated connections: {}", e))?;
            println!("Migrated {} connections to encrypted passwords", connections.len());
        }
    }
    
    Ok(())
}