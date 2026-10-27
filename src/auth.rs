use anyhow::{anyhow, bail, Result};
use argon2::Argon2;
use base64ct::{Base64, Encoding};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub const KEYRING_SERVICE: &str = "tfa";
pub const KEYRING_AUTH_KEY: &str = "__tfa_master_password__";

/// Record containing the salt and Argon2 derived hash for master password verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PasswordHashRecord {
    pub salt: String, // Base64 encoded 16-byte salt
    pub hash: String, // Base64 encoded 32-byte Argon2 hash
}

/// Active session cache indicating an unlocked state and last access time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionCache {
    pub unlocked_at: u64,
    pub last_accessed: u64,
    pub auth_token: String,
}

/// Create a salted Argon2id password verification record from a plaintext passphrase.
pub fn create_password_record(passphrase: &str) -> Result<PasswordHashRecord> {
    if passphrase.is_empty() {
        bail!("Password cannot be empty");
    }
    let mut salt = [0u8; 16];
    rand::fill(&mut salt);
    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("Argon2 derivation failed: {e}"))?;

    Ok(PasswordHashRecord {
        salt: Base64::encode_string(&salt),
        hash: Base64::encode_string(&key),
    })
}

/// Verify a passphrase against a stored password record using constant-time comparison.
pub fn verify_password(passphrase: &str, record: &PasswordHashRecord) -> bool {
    if passphrase.is_empty() {
        return false;
    }
    let Ok(salt) = Base64::decode_vec(&record.salt) else {
        return false;
    };
    let Ok(expected_key) = Base64::decode_vec(&record.hash) else {
        return false;
    };
    if salt.len() != 16 || expected_key.len() != 32 {
        return false;
    }

    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    if argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .is_err()
    {
        return false;
    }

    // Constant-time byte comparison
    let mut diff = 0u8;
    for (a, b) in key.iter().zip(expected_key.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Generate a unique authentication token derived from the current password record.
pub fn compute_auth_token(record: &PasswordHashRecord) -> String {
    format!("{}:{}", record.salt, record.hash)
}

/// Path to persistent auth record file when keyring is unavailable or overridden.
pub fn auth_file_path() -> PathBuf {
    if let Ok(path) = std::env::var("TFA_AUTH_FILE") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    if let Ok(data) = std::env::var("XDG_DATA_HOME") {
        if !data.is_empty() {
            return PathBuf::from(data).join("tfa").join(".password_hash");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("tfa")
            .join(".password_hash");
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        if !local.is_empty() {
            return PathBuf::from(local).join("tfa").join(".password_hash");
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        if !appdata.is_empty() {
            return PathBuf::from(appdata).join("tfa").join(".password_hash");
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        if !userprofile.is_empty() {
            return PathBuf::from(userprofile)
                .join(".local")
                .join("share")
                .join("tfa")
                .join(".password_hash");
        }
    }
    std::env::temp_dir().join(".tfa_password_hash")
}

/// Path to transient session cache file.
pub fn session_file_path() -> PathBuf {
    if let Ok(path) = std::env::var("TFA_SESSION_FILE") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime.is_empty() {
            return PathBuf::from(runtime).join("tfa").join("session");
        }
    }
    if let Ok(data) = std::env::var("XDG_DATA_HOME") {
        if !data.is_empty() {
            return PathBuf::from(data).join("tfa").join(".session");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("tfa")
            .join(".session");
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        if !local.is_empty() {
            return PathBuf::from(local).join("tfa").join(".session");
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        if !appdata.is_empty() {
            return PathBuf::from(appdata).join("tfa").join(".session");
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        if !userprofile.is_empty() {
            return PathBuf::from(userprofile)
                .join(".local")
                .join("share")
                .join("tfa")
                .join(".session");
        }
    }
    std::env::temp_dir().join(".tfa_session")
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Save password record to keyring if available, and write to auth file with 0600 permissions.
pub fn save_password_record(record: &PasswordHashRecord) -> Result<()> {
    let json_str = serde_json::to_string(record)?;

    // If explicit TFA_AUTH_FILE is set (e.g. during testing or headless override), isolate to file
    if std::env::var("TFA_AUTH_FILE").is_ok() {
        let path = auth_file_path();
        return write_secure_file(&path, &json_str);
    }

    // Attempt saving to OS keyring
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_AUTH_KEY) {
        let _ = entry.set_password(&json_str);
    }

    // Always also persist to local auth file with restricted permissions (0600)
    let path = auth_file_path();
    write_secure_file(&path, &json_str)?;
    Ok(())
}

/// Retrieve stored password record, checking keyring first and falling back to auth file.
pub fn get_password_record() -> Result<Option<PasswordHashRecord>> {
    // If explicit TFA_AUTH_FILE is set, isolate to that file
    if std::env::var("TFA_AUTH_FILE").is_ok() {
        let path = auth_file_path();
        if path.is_file() {
            let content = fs::read_to_string(&path)?;
            let record: PasswordHashRecord = serde_json::from_str(&content)?;
            return Ok(Some(record));
        }
        return Ok(None);
    }

    // 1. Try keyring
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_AUTH_KEY) {
        if let Ok(secret) = entry.get_password() {
            if let Ok(record) = serde_json::from_str::<PasswordHashRecord>(&secret) {
                return Ok(Some(record));
            }
        }
    }

    // 2. Try file fallback
    let path = auth_file_path();
    if path.is_file() {
        let content = fs::read_to_string(&path)?;
        let record: PasswordHashRecord = serde_json::from_str(&content)?;
        return Ok(Some(record));
    }

    Ok(None)
}

/// Delete stored password record from both keyring and auth file.
pub fn delete_password_record() -> Result<()> {
    if std::env::var("TFA_AUTH_FILE").is_err() {
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_AUTH_KEY) {
            let _ = entry.delete_credential();
        }
    }
    let path = auth_file_path();
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

/// Helper to write file with 0600 permissions.
fn write_secure_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
        }
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options.open(path)?;
    file.write_all(content.as_bytes())?;
    #[cfg(unix)]
    {
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Check if session is currently unlocked and within timeout window.
/// If valid, updates last_accessed timestamp (sliding window like sudo).
pub fn is_session_unlocked(expected_token: &str, timeout_secs: u64) -> bool {
    if timeout_secs == 0 {
        return false;
    }
    let path = session_file_path();
    if !path.is_file() {
        return false;
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut session: SessionCache = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(_) => return false,
    };

    if session.auth_token != expected_token {
        let _ = fs::remove_file(&path);
        return false;
    }

    let now = current_timestamp();
    if now < session.last_accessed {
        // Clock tampering or skew
        let _ = fs::remove_file(&path);
        return false;
    }

    let elapsed = now.saturating_sub(session.last_accessed);
    if elapsed < timeout_secs {
        // Still valid! Refresh last_accessed timestamp (sliding window)
        session.last_accessed = now;
        if let Ok(updated_json) = serde_json::to_string(&session) {
            let _ = write_secure_file(&path, &updated_json);
        }
        true
    } else {
        // Session expired
        let _ = fs::remove_file(&path);
        false
    }
}

/// Record an unlock event into the transient session cache.
pub fn record_unlock(auth_token: &str) -> Result<()> {
    let now = current_timestamp();
    let session = SessionCache {
        unlocked_at: now,
        last_accessed: now,
        auth_token: auth_token.to_string(),
    };
    let path = session_file_path();
    let json = serde_json::to_string(&session)?;
    write_secure_file(&path, &json)?;
    Ok(())
}

/// Invalidate active session immediately (lock).
pub fn clear_session() -> Result<()> {
    let path = session_file_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Return remaining seconds of session unlock, or None if locked / expired.
pub fn session_status(expected_token: &str, timeout_secs: u64) -> Option<u64> {
    if timeout_secs == 0 {
        return None;
    }
    let path = session_file_path();
    if !path.is_file() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let session: SessionCache = serde_json::from_str(&content).ok()?;

    if session.auth_token != expected_token {
        return None;
    }

    let now = current_timestamp();
    if now < session.last_accessed {
        return None;
    }

    let elapsed = now.saturating_sub(session.last_accessed);
    if elapsed <= timeout_secs {
        Some(timeout_secs.saturating_sub(elapsed))
    } else {
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::tempdir;

    pub static TEST_AUTH_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_password_hash_and_verify() {
        let pass = "SuperSecretPassword123!";
        let record = create_password_record(pass).unwrap();

        // 1. Correct password verifies
        assert!(verify_password(pass, &record));

        // 2. Incorrect password fails
        assert!(!verify_password("WrongPassword", &record));

        // 3. Empty password fails
        assert!(!verify_password("", &record));
        assert!(create_password_record("").is_err());
    }

    #[test]
    fn test_verify_password_tampered_record_fails() {
        let pass = "Password";
        let mut record = create_password_record(pass).unwrap();

        // 1. Corrupt salt
        record.salt = "invalid-base64!".to_string();
        assert!(!verify_password(pass, &record));

        // 2. Corrupt hash
        record.hash = "invalid-base64!".to_string();
        assert!(!verify_password(pass, &record));

        // 3. Wrong length salt / hash
        record.salt = Base64::encode_string(&[1u8; 8]); // 8 bytes instead of 16
        record.hash = Base64::encode_string(&[2u8; 32]);
        assert!(!verify_password(pass, &record));

        // 4. Compute auth token
        let token = compute_auth_token(&record);
        assert!(token.contains(':'));
    }

    #[test]
    fn test_auth_and_session_file_paths() {
        let _guard = TEST_AUTH_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();

        std::env::set_var("XDG_DATA_HOME", dir_str);
        std::env::set_var("XDG_RUNTIME_DIR", dir_str);
        std::env::remove_var("TFA_AUTH_FILE");
        std::env::remove_var("TFA_SESSION_FILE");

        let auth_path = auth_file_path();
        assert!(auth_path.starts_with(dir.path()));
        assert!(auth_path.ends_with(".password_hash"));

        let sess_path = session_file_path();
        assert!(sess_path.starts_with(dir.path()));
        assert!(sess_path.ends_with("session"));

        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_auth_record_default_storage_fallback() {
        let _guard = TEST_AUTH_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();

        std::env::remove_var("TFA_AUTH_FILE");
        std::env::set_var("XDG_DATA_HOME", dir_str);

        let record = create_password_record("DefaultStoragePass").unwrap();
        save_password_record(&record).unwrap();

        let loaded = get_password_record().unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), record);

        delete_password_record().unwrap();
        assert!(get_password_record().unwrap().is_none());

        std::env::remove_var("XDG_DATA_HOME");
    }

    #[test]
    fn test_session_edge_cases() {
        let _guard = TEST_AUTH_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let session_path = dir.path().join("edge_session.json");
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let token = "edge_token_123";

        // 1. Corrupt json in session file
        std::fs::write(&session_path, "not json").unwrap();
        assert!(!is_session_unlocked(token, 300));
        assert_eq!(session_status(token, 300), None);

        // 2. Clock tampering (future timestamp)
        let future_session = SessionCache {
            unlocked_at: u64::MAX,
            last_accessed: u64::MAX,
            auth_token: token.to_string(),
        };
        std::fs::write(
            &session_path,
            serde_json::to_string(&future_session).unwrap(),
        )
        .unwrap();
        assert!(!is_session_unlocked(token, 300));
        assert_eq!(session_status(token, 300), None);

        // 3. Expired session
        let expired_session = SessionCache {
            unlocked_at: 100,
            last_accessed: 100,
            auth_token: token.to_string(),
        };
        std::fs::write(
            &session_path,
            serde_json::to_string(&expired_session).unwrap(),
        )
        .unwrap();
        assert!(!is_session_unlocked(token, 10));
        assert_eq!(session_status(token, 10), None);

        // 4. Wrong token session status
        record_unlock(token).unwrap();
        assert_eq!(session_status("other_token", 300), None);
        assert_eq!(session_status(token, 0), None);

        std::env::remove_var("TFA_SESSION_FILE");
    }

    #[test]
    fn test_auth_record_file_storage() {
        let _guard = TEST_AUTH_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let auth_path = dir.path().join("tfa_auth.json");
        std::env::set_var("TFA_AUTH_FILE", &auth_path);

        let record = create_password_record("P@ssw0rd").unwrap();
        save_password_record(&record).unwrap();

        let loaded = get_password_record().unwrap().unwrap();
        assert_eq!(loaded, record);

        delete_password_record().unwrap();
        assert!(get_password_record().unwrap().is_none());

        std::env::remove_var("TFA_AUTH_FILE");
    }

    #[test]
    fn test_session_lifecycle_and_timeout() {
        let _guard = TEST_AUTH_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let session_path = dir.path().join("tfa_session.json");
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let token = "test_auth_token_123";

        // 1. Initially locked
        assert!(!is_session_unlocked(token, 300));
        assert_eq!(session_status(token, 300), None);

        // 2. Unlock session
        record_unlock(token).unwrap();
        assert!(is_session_unlocked(token, 300));
        let remaining = session_status(token, 300);
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= 300);

        // 3. Different token fails
        assert!(!is_session_unlocked("wrong_token", 300));

        // 4. Zero timeout expires immediately
        record_unlock(token).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(!is_session_unlocked(token, 0));

        // 5. Clear session locks
        record_unlock(token).unwrap();
        clear_session().unwrap();
        assert!(!is_session_unlocked(token, 300));

        std::env::remove_var("TFA_SESSION_FILE");
    }
}
