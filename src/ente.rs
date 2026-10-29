use anyhow::{anyhow, bail, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chacha20poly1305::aead::Aead as ChaChaAead;
use chacha20poly1305::{KeyInit as ChaChaKeyInit, XChaCha20Poly1305, XNonce};
use crypto_secretstream::{Header, Key, PullStream};
use serde_json::Value;

/// Check if a string payload is an encrypted Ente Auth export.
pub fn is_encrypted_ente_backup(input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return false;
    }
    if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
        if let Some(obj) = val.as_object() {
            return obj.contains_key("kdfParams")
                && obj.contains_key("encryptedData")
                && obj.contains_key("encryptionNonce");
        }
    }
    false
}

/// Decrypt an encrypted Ente Auth backup payload.
pub fn decrypt_ente_backup(input: &str, passphrase: &str) -> Result<String> {
    if passphrase.is_empty() {
        bail!("Passphrase cannot be empty");
    }

    let val: Value =
        serde_json::from_str(input.trim()).map_err(|e| anyhow!("Invalid Ente Auth JSON: {e}"))?;

    let kdf_params = val
        .get("kdfParams")
        .and_then(|k| k.as_object())
        .ok_or_else(|| anyhow!("Missing or invalid kdfParams in Ente export"))?;

    let mem_limit = kdf_params
        .get("memLimit")
        .and_then(|m| m.as_u64())
        .ok_or_else(|| anyhow!("Missing memLimit in Ente kdfParams"))?;
    let ops_limit = kdf_params
        .get("opsLimit")
        .and_then(|o| o.as_u64())
        .ok_or_else(|| anyhow!("Missing opsLimit in Ente kdfParams"))? as u32;
    let salt_b64 = kdf_params
        .get("salt")
        .and_then(|s| s.as_str())
        .ok_or_else(|| anyhow!("Missing salt in Ente kdfParams"))?;

    let clean_salt: String = salt_b64.chars().filter(|c| !c.is_whitespace()).collect();
    let salt = BASE64_STANDARD
        .decode(&clean_salt)
        .map_err(|e| anyhow!("Failed decoding Ente salt: {e}"))?;

    // memLimit in Ente Auth is in bytes (minimum 1024). Argon2 params expect KiB.
    let m_cost = (mem_limit / 1024) as u32;
    if m_cost == 0 || ops_limit == 0 {
        bail!("Invalid memory or operation limits in Ente kdfParams");
    }

    let params = argon2::Params::new(m_cost, ops_limit, 1, Some(32))
        .map_err(|e| anyhow!("Invalid Argon2 parameters: {e}"))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("Argon2 key derivation failed: {e}"))?;

    let enc_data_b64 = val
        .get("encryptedData")
        .and_then(|d| d.as_str())
        .ok_or_else(|| anyhow!("Missing encryptedData in Ente export"))?;
    let enc_nonce_b64 = val
        .get("encryptionNonce")
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow!("Missing encryptionNonce in Ente export"))?;

    let clean_enc_data: String = enc_data_b64
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let enc_data = BASE64_STANDARD
        .decode(&clean_enc_data)
        .map_err(|e| anyhow!("Failed decoding Ente encryptedData: {e}"))?;

    let clean_enc_nonce: String = enc_nonce_b64
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let enc_nonce = BASE64_STANDARD
        .decode(&clean_enc_nonce)
        .map_err(|e| anyhow!("Failed decoding Ente encryptionNonce: {e}"))?;

    // 1. Try libsodium crypto_secretstream (default format for Ente Auth exports)
    if enc_nonce.len() == 24 {
        if let (Ok(header), Ok(key_obj)) = (
            Header::try_from(enc_nonce.as_slice()),
            Key::try_from(key.as_slice()),
        ) {
            let mut pull_stream = PullStream::init(header, &key_obj);
            let mut buf = enc_data.clone();
            if pull_stream.pull(&mut buf, &[]).is_ok() {
                if let Ok(plaintext) = String::from_utf8(buf) {
                    return Ok(plaintext);
                }
            }
        }
    }

    // 2. Try XChaCha20-Poly1305 AEAD
    if enc_nonce.len() == 24 {
        if let Ok(cipher) = XChaCha20Poly1305::new_from_slice(&key) {
            let nonce = XNonce::from_slice(enc_nonce.as_slice());
            if let Ok(dec) = cipher.decrypt(nonce, enc_data.as_slice()) {
                if let Ok(plaintext) = String::from_utf8(dec) {
                    return Ok(plaintext);
                }
            }
        }
    }

    // 3. Try AES-256-GCM
    if enc_nonce.len() == 12 {
        use aes_gcm::aead::{Aead, KeyInit};
        use aes_gcm::Aes256Gcm;
        if let (Ok(cipher), Ok(nonce)) = (
            Aes256Gcm::new_from_slice(&key),
            aes_gcm::Nonce::try_from(enc_nonce.as_slice()),
        ) {
            if let Ok(dec) = cipher.decrypt(&nonce, enc_data.as_slice()) {
                if let Ok(plaintext) = String::from_utf8(dec) {
                    return Ok(plaintext);
                }
            }
        }
    }

    bail!("Failed decrypting Ente Auth export (incorrect passphrase or corrupted data)")
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const ENTE_ENCRYPTED_FIXTURE: &str = r#"{"version":1,"kdfParams":{"memLimit":1073741824,"opsLimit":4,"salt":"Vy6AFjE1CJjPgd6p2/hLsQ=="},"encryptedData":"Mpzi/NtcjSpB7nU2DvmDgh2bkas8doBtX4aCmqBetzTNuiZaTqr67bOHhc4CCWyAxxbKBw4BF1f5EyBwS29cvGmsvv2tiabxfifx/7FEDpLoH3qkXQwL9RLi3dnV/LnJQwNWeQ6YWL4kKZCWJ4xJ1AycYnQH8mr92M7oXoYz1AJn1yMInqiFyNBGMcjUfqQvXzs7+G0rTgIoh9ktVzopt/wrE2L5bDgbQAdrWEGqbWT/0l+t7v6di3TCVcibWFWSR+myUi/eF+5TX3OtEH3BtzPn2a94hXMck5eOhV9TUB26tIVWYHvdgrncveSqsIkCjYfGMKcMyehhFuyTkHoeerN2F/zDmTVBBAWyE6r+D/j/MN53x7OnL3QtoAv82dvr/3tyAjrRi4Qok7K/mBBDKkt9dB9tURjQ73cJS52NifYdqYMh1PQs+EJmy1kPuDBPLjGii9PQqqnuznorbFcx6QlxDD0SxyzJjvVtcyZvEBQeOG03tHKD0VBBd9mD5mhL6Vemzr5JNemMnqy34v2qno3HiL8pVh9O0uaTnF5P8HQExYyd","encryptionNonce":"Gzs0/9V7FT/Z0CJZrtX6bsEggRQW5NF7"}"#;

    #[test]
    fn test_is_encrypted_ente_backup() {
        assert!(is_encrypted_ente_backup(ENTE_ENCRYPTED_FIXTURE));
        assert!(!is_encrypted_ente_backup(
            r#"{"version": 1, "other": "test"}"#
        ));
        assert!(!is_encrypted_ente_backup("otpauth://totp/GitHub:alice"));
        assert!(!is_encrypted_ente_backup("not json"));
    }

    #[test]
    fn test_decrypt_ente_backup_fast_success() {
        let plaintext = "otpauth://totp/GitHub:alice@example.com?secret=JBSWY3DPEHPK3PXP&issuer=GitHub\notpauth://totp/Proton:bob@proton.me?secret=JBSWY3DPEHPK3PXP&issuer=Proton\n";
        let encrypted = encrypt_ente_export_for_test(plaintext, "testpass", 1048576, 1);
        let decrypted = decrypt_ente_backup(&encrypted, "testpass").unwrap();
        assert_eq!(decrypted, plaintext);

        let items = crate::import::parse_import_payload(&decrypted).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "alice@example.com");
        assert_eq!(items[1].account, "bob@proton.me");
        for item in &items {
            let engine = crate::totp::TfaTotp::from_input(&item.account, &item.secret).unwrap();
            let code = engine.generate_current();
            assert_eq!(code.len(), 6);
        }
    }

    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "slow in unoptimized debug build (1GB Argon2id)"
    )]
    fn test_decrypt_ente_backup_official_fixture() {
        let decrypted = decrypt_ente_backup(ENTE_ENCRYPTED_FIXTURE, "test").unwrap();
        assert!(decrypted.contains("otpauth://totp/"));

        let items = crate::import::parse_import_payload(&decrypted).unwrap();
        assert_eq!(items.len(), 5);
        for item in &items {
            let engine = crate::totp::TfaTotp::from_input(&item.account, &item.secret).unwrap();
            let code = engine.generate_current();
            assert_eq!(code.len(), 6);
        }
    }

    #[test]
    fn test_decrypt_ente_backup_wrong_password_fails() {
        let fast_fixture = encrypt_ente_export_for_test(
            "otpauth://totp/GitHub:alice?secret=JBSWY3DPEHPK3PXP\n",
            "correct_password",
            1048576,
            1,
        );
        let err = decrypt_ente_backup(&fast_fixture, "wrong_password");
        assert!(err.is_err());
    }

    #[test]
    fn test_decrypt_ente_backup_corrupted_fails() {
        let mut fast_fixture: Value = serde_json::from_str(&encrypt_ente_export_for_test(
            "otpauth://totp/GitHub:alice?secret=JBSWY3DPEHPK3PXP\n",
            "password123",
            1048576,
            1,
        ))
        .unwrap();

        fast_fixture["encryptedData"] = serde_json::Value::String("bm90dmFsaWQ=".to_string());
        let err = decrypt_ente_backup(&fast_fixture.to_string(), "password123");
        assert!(err.is_err());
    }

    #[test]
    fn test_round_trip_secretstream() {
        let plaintext =
            "otpauth://totp/Ente:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Ente\n";
        let encrypted = encrypt_ente_export_for_test(plaintext, "mypassword", 1048576, 1);
        assert!(is_encrypted_ente_backup(&encrypted));
        let decrypted = decrypt_ente_backup(&encrypted, "mypassword").unwrap();
        assert_eq!(decrypted, plaintext);
    }

    pub fn encrypt_ente_export_for_test(
        plaintext: &str,
        password: &str,
        mem_limit: u64,
        ops_limit: u32,
    ) -> String {
        use crypto_secretstream::{Key, PushStream, Tag};

        let salt = [42u8; 16];
        let m_cost = (mem_limit / 1024) as u32;
        let params = argon2::Params::new(m_cost, ops_limit, 1, Some(32)).unwrap();
        let argon2 =
            argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        let mut key_bytes = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), &salt, &mut key_bytes)
            .unwrap();

        let key = Key::try_from(key_bytes.as_slice()).unwrap();
        let (header, mut push_stream) = PushStream::init(rand_core::OsRng, &key);

        let mut buf = plaintext.as_bytes().to_vec();
        push_stream.push(&mut buf, &[], Tag::Final).unwrap();

        let export = serde_json::json!({
            "version": 1,
            "kdfParams": {
                "memLimit": mem_limit,
                "opsLimit": ops_limit,
                "salt": BASE64_STANDARD.encode(salt)
            },
            "encryptedData": BASE64_STANDARD.encode(buf),
            "encryptionNonce": BASE64_STANDARD.encode(header.as_ref())
        });

        serde_json::to_string(&export).unwrap()
    }

    #[test]
    fn test_decrypt_ente_backup_empty_password_fails() {
        let err = decrypt_ente_backup(ENTE_ENCRYPTED_FIXTURE, "");
        assert!(err.is_err());
    }
}
