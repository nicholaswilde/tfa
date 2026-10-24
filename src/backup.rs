use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, bail, Context, Result};
use argon2::Argon2;
use base64ct::{Base64, Encoding};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct EncryptedBackup {
    pub version: u32,
    pub kdf: String,
    pub cipher: String,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

pub fn is_encrypted_backup(input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return false;
    }
    serde_json::from_str::<EncryptedBackup>(trimmed).is_ok()
}

pub fn encrypt_backup(data: &[u8], passphrase: &str) -> Result<String> {
    if passphrase.is_empty() {
        bail!("Passphrase cannot be empty");
    }

    let mut salt = [0u8; 16];
    let mut nonce_bytes = [0u8; 12];
    rand::fill(&mut salt);
    rand::fill(&mut nonce_bytes);

    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("Argon2 key derivation failed: {e}"))?;

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow!("Failed initializing AES-256-GCM cipher: {e}"))?;
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext_bytes = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!("Encryption failed: {e}"))?;

    let backup = EncryptedBackup {
        version: 1,
        kdf: "argon2id".to_string(),
        cipher: "aes-256-gcm".to_string(),
        salt: Base64::encode_string(&salt),
        nonce: Base64::encode_string(&nonce_bytes),
        ciphertext: Base64::encode_string(&ciphertext_bytes),
    };

    Ok(serde_json::to_string_pretty(&backup)?)
}

pub fn decrypt_backup(json_str: &str, passphrase: &str) -> Result<Vec<u8>> {
    let backup: EncryptedBackup =
        serde_json::from_str(json_str.trim()).context("Invalid encrypted backup structure")?;

    if backup.version != 1 {
        bail!("Unsupported backup version: {}", backup.version);
    }
    if backup.kdf != "argon2id" {
        bail!("Unsupported KDF algorithm: {}", backup.kdf);
    }
    if backup.cipher != "aes-256-gcm" {
        bail!("Unsupported cipher: {}", backup.cipher);
    }

    let salt =
        Base64::decode_vec(&backup.salt).map_err(|e| anyhow!("Failed decoding salt: {e}"))?;
    let nonce_bytes =
        Base64::decode_vec(&backup.nonce).map_err(|e| anyhow!("Failed decoding nonce: {e}"))?;
    let ciphertext = Base64::decode_vec(&backup.ciphertext)
        .map_err(|e| anyhow!("Failed decoding ciphertext: {e}"))?;

    if nonce_bytes.len() != 12 {
        bail!("Invalid nonce length (expected 12 bytes)");
    }

    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("Argon2 key derivation failed: {e}"))?;

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow!("Failed initializing AES-256-GCM cipher: {e}"))?;
    let nonce =
        Nonce::try_from(nonce_bytes.as_slice()).map_err(|e| anyhow!("Invalid nonce: {e}"))?;
    let decrypted = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_| anyhow!("Decryption failed: incorrect passphrase or corrupted data"))?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let secret_payload = r#"{"github":"JBSWY3DPEHPK3PXP","aws":"otpauth://totp/AWS:user?secret=JBSWY3DPEHPK3PXP"}"#;
        let passphrase = "correct horse battery staple";

        let encrypted_json = encrypt_backup(secret_payload.as_bytes(), passphrase).unwrap();
        assert!(is_encrypted_backup(&encrypted_json));

        let decrypted = decrypt_backup(&encrypted_json, passphrase).unwrap();
        assert_eq!(String::from_utf8(decrypted).unwrap(), secret_payload);
    }

    #[test]
    fn test_decrypt_with_wrong_password_fails() {
        let payload = b"sensitive 2fa data";
        let encrypted_json = encrypt_backup(payload, "secret123").unwrap();

        let err = decrypt_backup(&encrypted_json, "wrongsecret");
        assert!(err.is_err());
        assert!(err
            .unwrap_err()
            .to_string()
            .contains("incorrect passphrase"));
    }

    #[test]
    fn test_is_encrypted_backup_detection() {
        assert!(!is_encrypted_backup("otpauth://totp/test?secret=ABC"));
        assert!(!is_encrypted_backup(r#"{"items": []}"#));
        assert!(!is_encrypted_backup("plain text"));
    }

    #[test]
    fn test_encrypt_empty_passphrase_fails() {
        let err = encrypt_backup(b"data", "");
        assert!(err.is_err());
    }

    #[test]
    fn test_decrypt_invalid_fields_fails() {
        // Invalid json
        assert!(decrypt_backup("not json", "pass").is_err());

        // Unsupported version
        let bad_ver = serde_json::to_string(&EncryptedBackup {
            version: 99,
            kdf: "argon2id".to_string(),
            cipher: "aes-256-gcm".to_string(),
            salt: "AA==".to_string(),
            nonce: "AA==".to_string(),
            ciphertext: "AA==".to_string(),
        })
        .unwrap();
        assert!(decrypt_backup(&bad_ver, "pass").is_err());

        // Unsupported kdf
        let bad_kdf = serde_json::to_string(&EncryptedBackup {
            version: 1,
            kdf: "pbkdf2".to_string(),
            cipher: "aes-256-gcm".to_string(),
            salt: "AA==".to_string(),
            nonce: "AA==".to_string(),
            ciphertext: "AA==".to_string(),
        })
        .unwrap();
        assert!(decrypt_backup(&bad_kdf, "pass").is_err());

        // Unsupported cipher
        let bad_cipher = serde_json::to_string(&EncryptedBackup {
            version: 1,
            kdf: "argon2id".to_string(),
            cipher: "des".to_string(),
            salt: "AA==".to_string(),
            nonce: "AA==".to_string(),
            ciphertext: "AA==".to_string(),
        })
        .unwrap();
        assert!(decrypt_backup(&bad_cipher, "pass").is_err());

        // Invalid nonce length
        let bad_nonce = serde_json::to_string(&EncryptedBackup {
            version: 1,
            kdf: "argon2id".to_string(),
            cipher: "aes-256-gcm".to_string(),
            salt: "c29tZXNhbHQxMjM0NTY3OA==".to_string(),
            nonce: "c2hvcnQ=".to_string(), // short nonce
            ciphertext: "AA==".to_string(),
        })
        .unwrap();
        assert!(decrypt_backup(&bad_nonce, "pass").is_err());
    }
}
