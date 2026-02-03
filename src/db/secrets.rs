use std::fs;
use std::env;

pub fn get_secret(key: &str) -> String {
    // 1. Check for the Docker Secret file path
    // Docker mounts secrets at /run/secrets/<name>
    let secret_path = format!("/run/secrets/{}", key.to_lowercase());
    
    if let Ok(content) = fs::read_to_string(&secret_path) {
        return content.trim().to_string();
    }

    // 2. Fallback to standard environment variable
    env::var(key).unwrap_or_else(|_| panic!("Secret {} not found in file or env", key))
}