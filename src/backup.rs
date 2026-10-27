use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, bail, Context, Result};
use argon2::Argon2;
use base64ct::{Base64, Encoding};
use serde::{Deserialize, Serialize};
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};

#[cfg(test)]
pub(crate) static GPG_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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

fn gpg_program() -> String {
    std::env::var("TFA_GPG_BIN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "gpg".to_string())
}

pub fn is_gpg_backup(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.starts_with("-----BEGIN PGP MESSAGE-----")
        && trimmed.ends_with("-----END PGP MESSAGE-----")
}

pub fn encrypt_backup_gpg(data: &[u8], recipient: &str) -> Result<String> {
    let recipient = recipient.trim();
    if recipient.is_empty() {
        bail!("GPG key fingerprint cannot be empty");
    }
    if recipient.starts_with('-') {
        bail!("Invalid GPG key fingerprint '{recipient}'");
    }

    let mut child = Command::new(gpg_program())
        .args([
            "--batch",
            "--yes",
            "--armor",
            "--trust-model",
            "always",
            "--recipient",
            recipient,
            "--encrypt",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to execute gpg for encryption: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(data);
    }

    let output = child
        .wait_with_output()
        .context("Failed waiting for gpg encryption process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        if detail.contains("No public key")
            || detail.contains("not found")
            || detail.contains("skipped")
        {
            bail!("GPG key '{recipient}' not found in local keyring ({detail})");
        }
        bail!("GPG encryption failed for recipient '{recipient}': {detail}");
    }

    String::from_utf8(output.stdout).context("GPG encrypted output is not valid UTF-8")
}

pub fn decrypt_backup_gpg(armored: &str) -> Result<Vec<u8>> {
    if !is_gpg_backup(armored) {
        bail!("Invalid GPG backup format: missing PGP message header or footer");
    }

    let mut cmd = Command::new(gpg_program());
    if !std::io::stdin().is_terminal()
        || (cfg!(test) && std::env::var("TFA_TEST_INTERACTIVE").is_err())
    {
        cmd.arg("--batch");
    }

    let mut child = cmd
        .args(["--yes", "--quiet", "--decrypt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to execute gpg for decryption: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(armored.as_bytes());
    }

    let output = child
        .wait_with_output()
        .context("Failed waiting for gpg decryption process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        if detail.contains("No secret key") {
            bail!(
                "GPG decryption failed: secret key not found in local keyring or GPG agent ({detail})"
            );
        }
        bail!("GPG decryption failed: {detail}");
    }

    Ok(output.stdout)
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

        // Invalid base64 in salt, nonce, ciphertext
        let bad_b64_salt = serde_json::to_string(&EncryptedBackup {
            version: 1,
            kdf: "argon2id".to_string(),
            cipher: "aes-256-gcm".to_string(),
            salt: "!not-base64!".to_string(),
            nonce: "c29tZXNhbHQxMjM0".to_string(),
            ciphertext: "AA==".to_string(),
        })
        .unwrap();
        assert!(decrypt_backup(&bad_b64_salt, "pass").is_err());

        let bad_b64_nonce = serde_json::to_string(&EncryptedBackup {
            version: 1,
            kdf: "argon2id".to_string(),
            cipher: "aes-256-gcm".to_string(),
            salt: "c29tZXNhbHQxMjM0NTY3OA==".to_string(),
            nonce: "!not-base64!".to_string(),
            ciphertext: "AA==".to_string(),
        })
        .unwrap();
        assert!(decrypt_backup(&bad_b64_nonce, "pass").is_err());

        let bad_b64_cipher = serde_json::to_string(&EncryptedBackup {
            version: 1,
            kdf: "argon2id".to_string(),
            cipher: "aes-256-gcm".to_string(),
            salt: "c29tZXNhbHQxMjM0NTY3OA==".to_string(),
            nonce: "MTIzNDU2Nzg5MDEy".to_string(),
            ciphertext: "!not-base64!".to_string(),
        })
        .unwrap();
        assert!(decrypt_backup(&bad_b64_cipher, "pass").is_err());

        // Debug and PartialEq on EncryptedBackup
        let b1: EncryptedBackup = serde_json::from_str(&bad_b64_cipher).unwrap();
        let b2: EncryptedBackup = serde_json::from_str(&bad_b64_cipher).unwrap();
        assert_eq!(b1, b2);
        assert!(!format!("{b1:?}").is_empty());
    }

    #[test]
    fn test_is_gpg_backup_detection() {
        let valid_gpg = "-----BEGIN PGP MESSAGE-----\n\nhF4D...\n-----END PGP MESSAGE-----\n";
        assert!(is_gpg_backup(valid_gpg));
        assert!(!is_gpg_backup("otpauth://totp/test?secret=ABC"));
        assert!(!is_gpg_backup(r#"{"version": 1}"#));
        assert!(!is_gpg_backup("-----BEGIN PGP PUBLIC KEY BLOCK-----"));
    }

    #[test]
    fn test_encrypt_gpg_empty_or_invalid_recipient_fails() {
        let err_empty = encrypt_backup_gpg(b"data", "   ");
        assert!(err_empty.is_err());
        assert!(err_empty
            .unwrap_err()
            .to_string()
            .contains("cannot be empty"));

        let err_flag = encrypt_backup_gpg(b"data", "--armor");
        assert!(err_flag.is_err());
        assert!(err_flag
            .unwrap_err()
            .to_string()
            .contains("Invalid GPG key fingerprint"));
    }

    #[test]
    fn test_gpg_encrypt_decrypt_and_missing_key() {
        let _guard = GPG_TEST_MUTEX.lock().unwrap();
        let gpg_dir = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(gpg_dir.path(), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        std::env::set_var("GNUPGHOME", gpg_dir.path());

        // 1. Missing key fingerprint should fail with clear "not found" error
        let missing_err = encrypt_backup_gpg(b"secret", "NONEXISTENT_FINGERPRINT_9999");
        assert!(missing_err.is_err());
        let msg = missing_err.unwrap_err().to_string();
        assert!(
            msg.contains("not found"),
            "Expected 'not found' in error message, got: {msg}"
        );

        // 2. Generate a temporary GPG key in isolated GNUPGHOME
        let status = Command::new("gpg")
            .args([
                "--batch",
                "--passphrase",
                "",
                "--quick-generate-key",
                "TFA Test <tfa-test@example.com>",
                "default",
                "default",
                "never",
            ])
            .status()
            .unwrap();
        assert!(status.success());

        let list_out = Command::new("gpg")
            .args(["--list-keys", "--with-colons"])
            .output()
            .unwrap();
        let list_str = String::from_utf8_lossy(&list_out.stdout);
        let fpr = list_str
            .lines()
            .find(|l| l.starts_with("fpr:"))
            .and_then(|l| l.split(':').nth(9))
            .expect("Expected fingerprint in gpg --list-keys output")
            .to_string();

        // 3. Encrypt using full key fingerprint and decrypt roundtrip
        let payload = r#"{"github":"JBSWY3DPEHPK3PXP"}"#;
        let armored = encrypt_backup_gpg(payload.as_bytes(), &fpr).unwrap();
        assert!(is_gpg_backup(&armored));

        let decrypted = decrypt_backup_gpg(&armored).unwrap();
        assert_eq!(String::from_utf8(decrypted).unwrap(), payload);

        // 4. Decrypt invalid format fails
        assert!(decrypt_backup_gpg("not a pgp message").is_err());

        // 5. Decrypt with missing secret key in another empty keyring fails clearly
        let empty_gpg_dir = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(empty_gpg_dir.path(), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        std::env::set_var("GNUPGHOME", empty_gpg_dir.path());
        let no_sec_err = decrypt_backup_gpg(&armored);
        assert!(no_sec_err.is_err());
        assert!(no_sec_err
            .unwrap_err()
            .to_string()
            .contains("secret key not found"));

        let _ = Command::new("gpgconf")
            .args([
                "--homedir",
                gpg_dir.path().to_str().unwrap(),
                "--kill",
                "all",
            ])
            .status();
        let _ = Command::new("gpgconf")
            .args([
                "--homedir",
                empty_gpg_dir.path().to_str().unwrap(),
                "--kill",
                "all",
            ])
            .status();
        std::env::remove_var("GNUPGHOME");
    }
}
