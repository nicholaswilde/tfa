use anyhow::{anyhow, bail, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde_json::Value;

use crate::import::ImportItem;

/// Check if a string payload is an encrypted 2FAS backup.
pub fn is_encrypted_2fas_backup(input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return false;
    }
    if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
        if let Some(obj) = val.as_object() {
            if let Some(enc) = obj.get("servicesEncrypted") {
                if let Some(s) = enc.as_str() {
                    let parts: Vec<&str> = s.split(':').collect();
                    return parts.len() == 3;
                } else if let Some(enc_obj) = enc.as_object() {
                    return enc_obj.contains_key("data")
                        && enc_obj.contains_key("salt")
                        && enc_obj.contains_key("iv");
                }
            }
        }
    }
    false
}

/// Extract ciphertext, salt, and iv from servicesEncrypted payload.
fn extract_2fas_encrypted_fields(val: &Value) -> Option<(String, String, String)> {
    let obj = val.as_object()?;
    let enc = obj.get("servicesEncrypted")?;
    if let Some(s) = enc.as_str() {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 3 {
            return Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ));
        }
    } else if let Some(enc_obj) = enc.as_object() {
        let data = enc_obj.get("data").and_then(|v| v.as_str());
        let salt = enc_obj.get("salt").and_then(|v| v.as_str());
        let iv = enc_obj.get("iv").and_then(|v| v.as_str());
        if let (Some(d), Some(s), Some(i)) = (data, salt, iv) {
            return Some((d.to_string(), s.to_string(), i.to_string()));
        }
    }
    None
}

/// Decode Base64 or Hex encoded bytes, ignoring whitespace.
fn decode_bytes(input: &str) -> Result<Vec<u8>> {
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if let Ok(bytes) = BASE64_STANDARD.decode(&clean) {
        return Ok(bytes);
    }
    if let Ok(bytes) = hex::decode(&clean) {
        return Ok(bytes);
    }
    bail!("Could not decode cryptographic field as Base64 or Hex");
}

/// Decrypt an encrypted 2FAS backup payload.
pub fn decrypt_2fas_backup(input: &str, passphrase: &str) -> Result<String> {
    if passphrase.is_empty() {
        bail!("Passphrase cannot be empty");
    }

    let val: Value =
        serde_json::from_str(input.trim()).map_err(|e| anyhow!("Invalid 2FAS JSON: {e}"))?;

    let (data_b64, salt_b64, iv_b64) = extract_2fas_encrypted_fields(&val)
        .ok_or_else(|| anyhow!("Missing or invalid servicesEncrypted in 2FAS backup"))?;

    let ciphertext_with_tag =
        decode_bytes(&data_b64).map_err(|e| anyhow!("Failed decoding 2FAS ciphertext: {e}"))?;
    let salt = decode_bytes(&salt_b64).map_err(|e| anyhow!("Failed decoding 2FAS salt: {e}"))?;
    let iv = decode_bytes(&iv_b64).map_err(|e| anyhow!("Failed decoding 2FAS iv: {e}"))?;

    if iv.len() != 12 {
        bail!(
            "Invalid 2FAS IV length: expected 12 bytes, got {}",
            iv.len()
        );
    }
    if ciphertext_with_tag.len() < 16 {
        bail!("Invalid 2FAS ciphertext length: too short to contain authentication tag");
    }

    let mut key = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<sha2_v10::Sha256>(passphrase.as_bytes(), &salt, 10_000, &mut key);

    use aes_gcm::aead::{Aead, KeyInit};
    let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow!("Failed initializing AES-256-GCM cipher: {e}"))?;
    let nonce = aes_gcm::Nonce::try_from(iv.as_slice())
        .map_err(|_| anyhow!("Invalid 2FAS nonce length"))?;

    let decrypted = cipher
        .decrypt(&nonce, ciphertext_with_tag.as_slice())
        .map_err(|_| {
            anyhow!("Failed decrypting 2FAS backup (incorrect passphrase or corrupted data)")
        })?;

    let decrypted_str = String::from_utf8(decrypted)
        .map_err(|e| anyhow!("Decrypted 2FAS payload is not valid UTF-8: {e}"))?;

    Ok(decrypted_str)
}

/// Check if a JSON object or array represents a 2FAS service list.
pub fn is_2fas_services(val: &Value) -> bool {
    match val {
        Value::Object(map) => map.contains_key("services"),
        Value::Array(arr) => {
            if arr.is_empty() {
                return false;
            }
            arr.iter().any(|item| {
                if let Some(obj) = item.as_object() {
                    obj.contains_key("otp")
                        || obj.contains_key("updatedAt")
                        || obj.contains_key("tokenType")
                } else {
                    false
                }
            })
        }
        _ => false,
    }
}

/// Parse an array of 2FAS services into `ImportItem` items.
pub fn parse_2fas_services_array(arr: &[Value]) -> Vec<ImportItem> {
    let mut items = Vec::new();
    for entry in arr {
        if let Some(item) = parse_2fas_service_entry(entry) {
            items.push(item);
        }
    }
    items
}

/// Parse a single 2FAS service entry into `ImportItem`.
pub fn parse_2fas_service_entry(val: &Value) -> Option<ImportItem> {
    let obj = val.as_object()?;

    let service_name = obj
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .trim();

    let secret = obj
        .get("secret")
        .and_then(|s| s.as_str())
        .or_else(|| {
            obj.get("otp")
                .and_then(|o| o.get("secret"))
                .and_then(|s| s.as_str())
        })
        .unwrap_or("")
        .trim();

    let otp = obj.get("otp").and_then(|o| o.as_object());

    // 1. If otp contains a pre-built OTPAuth URI link, use it
    let otp_link = otp
        .and_then(|o| o.get("link"))
        .and_then(|l| l.as_str())
        .map(|s| s.trim());

    let otp_account = otp
        .and_then(|o| o.get("account"))
        .and_then(|a| a.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let account_name = match (service_name, otp_account) {
        (name, Some(acc)) if !name.is_empty() && name != acc => format!("{name}:{acc}"),
        (name, _) if !name.is_empty() => name.to_string(),
        (_, Some(acc)) => acc.to_string(),
        _ => "imported_account".to_string(),
    };

    if let Some(link) = otp_link {
        if link.starts_with("otpauth://") {
            return Some(ImportItem {
                account: account_name,
                secret: link.to_string(),
            });
        }
    }

    // If secret is empty, cannot generate valid 2FA
    if secret.is_empty() {
        return None;
    }

    let token_type = otp
        .and_then(|o| o.get("tokenType"))
        .and_then(|t| t.as_str())
        .unwrap_or("TOTP")
        .to_uppercase();

    let digits = otp
        .and_then(|o| o.get("digits"))
        .and_then(|d| d.as_u64())
        .map(|d| d as u8);

    let period = otp.and_then(|o| o.get("period")).and_then(|p| p.as_u64());

    let algorithm = otp
        .and_then(|o| o.get("algorithm"))
        .and_then(|a| a.as_str())
        .map(|a| a.to_uppercase());

    let counter = otp.and_then(|o| o.get("counter")).and_then(|c| c.as_u64());

    let is_hotp = token_type == "HOTP" || counter.is_some();
    let type_str = if is_hotp { "hotp" } else { "totp" };

    let url_str = format!("otpauth://{type_str}/{account_name}");
    let mut u = url::Url::parse(&url_str).ok()?;

    u.query_pairs_mut().append_pair("secret", secret);

    if !service_name.is_empty() {
        u.query_pairs_mut().append_pair("issuer", service_name);
    }

    if is_hotp {
        u.query_pairs_mut()
            .append_pair("counter", &counter.unwrap_or(0).to_string());
    } else if let Some(p) = period {
        u.query_pairs_mut().append_pair("period", &p.to_string());
    }

    if token_type == "STEAM" {
        u.query_pairs_mut().append_pair("algorithm", "STEAM");
        u.query_pairs_mut().append_pair("digits", "5");
    } else {
        if let Some(d) = digits {
            u.query_pairs_mut().append_pair("digits", &d.to_string());
        }
        if let Some(ref alg) = algorithm {
            u.query_pairs_mut().append_pair("algorithm", alg);
        }
    }

    Some(ImportItem {
        account: account_name,
        secret: u.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCRYPTED_2FAS_FIXTURE: &str = r#"{"services":[],"groups":[],"updatedAt":1708958781890,"schemaVersion":4,"appVersionCode":5000017,"appVersionName":"5.3.5","appOrigin":"android","servicesEncrypted":"HhVj7hNTJBdmidKVsCrgK0EafXu2aBshvz3KlBX0dIHHXA/GGCD5Ws8h3NE59DLXl3dgiPQgDYzCgTEYQIqgSXylmMh77odkAYQKDFWGL74etmdPulab480QbrJuUDfGOl6DtZynEn6WS/uyvJ5nrpEHp2UVAXCu6y7EIHuNgtwiY+O2UXEYX8zMKJWVqMeTZ4VyQCQvT9nTBbINJi8lyYIRmIup0cwPbh6JORpzV5yKWG0Jmbi/vdqXsFftYHjRmkwt/KsnjNmNWHTeztBOSZ4RlXpW24pEctUUqkiqz8HkMpOlOxkgR97MpxxoubN7gS/dqn8J6lAgBa0yWlf0QgqNVY5mqhSuMlfug2xTZDeG93jeiWQOqb2cs1bnEb5iUaUzpThw1P8XuzrmlByFQ3K7b71lXzyX6Rdqt05u4SIJMtc/I6CVqAta1fuQ1fcjufjcaIxglGOxKM5Fxy7gLp/Jwhl9Fin1RKC4JRd1h+siC87Lm9qA/WW7XmR81+wlhzlcXQ+xLxoccPl1CNhxibOswR0sQEhMPrOXLQk7Dy5KT9lxTIV0o67e9WgzbfOoIgQp2zxWeT3QjYrROivC2c6OsktUqgonFzzNTAxWxQtxOlUzS0BOmbn6t12KbJYE7Lv9fnv9wQJoKDU9QJai4fqFpcZeXnObN2VIt65QA7bumsr74PvgmS9Y+hu4xYhWEUkag0Q5YTpeLBiWCp71l0cx8uMrvoCw1JPphGnkjuFgVsBpcqKxHudsbi83fDVHBDv1HogO5cyMDDi4/h8N6Cdfz3xCsQg39X8RtvfFMf7kYSRmbWcNlqkmVHVWuIoxlXcsbAHw1LYRgYyodeCFDoR6NMmmP3I2mKAZfG+fXGnhHHb1+dwnNp/hyDFQ/RRFpSNMwAaU+/SUeYbE5yx2ux2dHNUJ2W7j19It1vdAE8xfQMl/qzxMODdq1L1D8lDyrzfwKT1S41wKDXjJBrx7EQAN/Eh5awBGEN+CWyce6l4wOOfvFfYx7iAmH7XDEzjcyRe3wsPlhqTZXr/VMsUKX//xONa2eSjunodKHekr/bI2xGV0qbisWtJmbEZBKkDhsrdUbPBSvADwSqJqlDIpqspljmm+xSpSNW2qPuRD5RIgXI4JkfjjBC9E2hgb+8otanTBI2y8i22GLkz+vr+JmqjDufLzLvvtO6Fj8SPf2B2pMgQNyRw8agUxXbdLlqhRpMPNlaxBKoU3lgitwGoQui7Va+r8kAtKdgZuC0EV/XOPKFY4KUDh+vXbjRsMQbf9xXHnwqkC5tkNk17r+4nVqoMeW4fTfXPNjn2v5euqa7bnUGKx3RhD6dleqH1mJLylyvla3g4EP+eI3sxW925sVK83j6AO8/zJz4zowqKAqDI1KiC1FnlU65TA2qMUU4t2YE1CwzGykWNtcHfjQDhGMOzpnhdhcIylcuJ/qdE2iVZ2ACA4R9VpHPxmoNuqlz4YbtGism6JSjOc43KxPblz+LOT+OXU1XYH1CuaB4Btrty0CG68OS10IXXKJ3UqiDubNfGZ0T0XH+yu87QJkIO0bpwoT45M+Sb9Zps8/zjE8dgVnsncf/7Y27wwemQJK5N4Ndju0/CCfixy4hNPpcGdqQsLRVdU44OLe+nblwA94wLv+SkJXrDYZqu8xbLVxd3Zbi/+nUs084oNSfMwsrq90qhjn3uKmtPTBMqi0IVxAqnmLhLHqEQcBCYFt+uxXJt6Btiehxs76LtMGnuT+t1N5kDpvnuXduXYqvm/lpR1W6rblaYyhnfTdT8S7zecM8BAQ5SN/Z284DkdMUVreYrInXUGc/uJHK8UswrpYfNonvtYpPP9Hc9qrkN/DtdsCmzUd4a1acexYjc31t0fBABl0cihW4yK4vwi77Z2mVyV+6eT2OVUaEGG/zFJtU0UIT/tN50gFXCP63pZu9RdYLD2isIDnlfluQO9gDgcp89KZcbkXB/2cafBypLq5r9RFKcacxY9zM8sEgZv5+p+WEEzP2v/Bk32d6zMJa8mcJcHdeUDsCqAzhsZ7yaJUwYpKqrWY1BUXszLBi2RbmqdVaqgdmYtj6sM9RaW+HHPbMTPwwe6N3MBcQpCFxMMXhY1ivmPzxoj5t/hMaDuDAT6HKT70KHr3ZCA9FqVi2VbEzg4bUBjMKXauZgzb6NZFGucSfV05wcB0Ao/MmBfmUhEb4QhLvhbPu3G4a10YVUsiNol0nypKlbnGsp93mvK7KW60EVOFiQbgsvicIeqFxgO2cZ/PJgbB5GJjPswGs6mMK07CEYJf5spaKBj0FK83NTW+8dg9D0C7ZKIB1huOtPa3rRUGC6RXl1p7BQy5ym9r6zpiTMNLoiJ5zkCUGrr4k88YjZnhcjr89T2T1hQo4qGoEIzKQfGzNobeHvBMj2wq63rTGfeX+uw6mqEPF0P3HG3JninzNQ+gVIWzei0VS1PM3X7dviehhHJm4QJLel5om/vOBtzqtOMf5MV79fmEDUx+FrxJSYiBu+glsB9R50eYaGjM0KDiwtPvDmXN8Haa0TAbegzx8fAFHd7NUoRrOR+p4dTY1Qs9Ut0/m3Ji3V5PxU6ra9GfJhsb6ABSHMauKR1d2jx6dl7hz+iVL2mtJwgERw+31jtktBZSPOcRK9Qsk6UvCHqPsMTcTFTOCxoZkFC5S4AVR/gMTxt+wG3DnYuGv3O2MIsOuVp3vk2vAeSRl8KEPHmwhNwmBPNcWVGNfIp/7rJfQwpwa5vjH5u7Hrr:xsCM/GAwNcyqrDcYodp58e6xxXl+cj0P+1Bh9mH4f7+UYKrQV4cpMAbQRPyNJz5CbsvSsFGYr+Ls1N+GyX6fp8LahIyovloySTRqQZzBI0VgKTKy1g7PlSSVjhedokyK5osUg6lUTimr29SGyvL4r/ornfkKygDZry8gHjyANX06mfxBK46+qomjsw5TErS0VlitPMJ1OWoh5/ZArEZBSczTGSOLjdQ3uMkQGOEUCJAd9wruBViN7td/0tmBAhzkG7EtrOJN7YNCGSLCiRoeLqS+unbaIOmUeKyn2AWd+jT/k4WcxIkHlYPRumy1DzS/REh6NUfagoO/1fPLMUYUug==:5UW4AuvcvsEi0jYe"}"#;

    #[test]
    fn test_is_encrypted_2fas_backup() {
        assert!(is_encrypted_2fas_backup(ENCRYPTED_2FAS_FIXTURE));
        assert!(!is_encrypted_2fas_backup(r#"{"services": []}"#));
        assert!(!is_encrypted_2fas_backup(r#"{"other": 123}"#));
        assert!(!is_encrypted_2fas_backup("not json"));
    }

    #[test]
    fn test_decrypt_2fas_backup_success() {
        let decrypted = decrypt_2fas_backup(ENCRYPTED_2FAS_FIXTURE, "example.com").unwrap();
        assert!(decrypted.contains("Deno"));
        assert!(decrypted.contains("4SJHB4GSD43FZBAI7C2HLRJGPQ"));

        let val: Value = serde_json::from_str(&decrypted).unwrap();
        let items = parse_2fas_services_array(val.as_array().unwrap());
        assert_eq!(items.len(), 5);

        // 1. Deno:Mason (TOTP, 6 digits, 30s, SHA1)
        assert_eq!(items[0].account, "Deno:Mason");
        assert!(items[0]
            .secret
            .contains("secret=4SJHB4GSD43FZBAI7C2HLRJGPQ"));
        assert!(items[0].secret.contains("issuer=Deno"));
        assert!(items[0].secret.contains("digits=6"));
        assert!(items[0].secret.contains("period=30"));

        // 2. SPDX:James (TOTP, 7 digits, 30s, SHA256)
        assert_eq!(items[1].account, "SPDX:James");
        assert!(items[1].secret.contains("digits=7"));
        assert!(items[1].secret.contains("algorithm=SHA256"));

        // 3. Airbnb:Elijah (TOTP, 8 digits, 60s, SHA512)
        assert_eq!(items[2].account, "Airbnb:Elijah");
        assert!(items[2].secret.contains("digits=8"));
        assert!(items[2].secret.contains("period=60"));
        assert!(items[2].secret.contains("algorithm=SHA512"));

        // 4. Boeing:Sophia (Steam, 5 digits, 10s, STEAM)
        assert_eq!(items[3].account, "Boeing:Sophia");
        assert!(items[3].secret.contains("algorithm=STEAM"));
        assert!(items[3].secret.contains("digits=5"));

        // 5. Air Canada:Benjamin (HOTP, 8 digits, SHA256, counter=10)
        assert_eq!(items[4].account, "Air Canada:Benjamin");
        assert!(items[4].secret.starts_with("otpauth://hotp/"));
        assert!(items[4].secret.contains("counter=10"));
    }

    #[test]
    fn test_decrypt_2fas_backup_wrong_password_fails() {
        let err = decrypt_2fas_backup(ENCRYPTED_2FAS_FIXTURE, "wrong_password");
        assert!(err.is_err());
    }

    #[test]
    fn test_decrypt_2fas_backup_empty_password_fails() {
        let err = decrypt_2fas_backup(ENCRYPTED_2FAS_FIXTURE, "");
        assert!(err.is_err());
    }

    #[test]
    fn test_parse_unencrypted_2fas_json() {
        let json_payload = r#"{
            "schemaVersion": 4,
            "services": [
                {
                    "name": "GitHub",
                    "secret": "JBSWY3DPEHPK3PXP",
                    "otp": {
                        "account": "alice@example.com",
                        "digits": 6,
                        "period": 30,
                        "algorithm": "SHA1",
                        "tokenType": "TOTP"
                    }
                },
                {
                    "name": "AWS",
                    "secret": "JBSWY3DPEHPK3PXP",
                    "otp": {
                        "account": "AWS",
                        "digits": 8,
                        "period": 60,
                        "algorithm": "SHA256",
                        "tokenType": "TOTP"
                    }
                }
            ]
        }"#;

        let val: Value = serde_json::from_str(json_payload).unwrap();
        assert!(is_2fas_services(&val));
        let arr = val.get("services").unwrap().as_array().unwrap();
        let items = parse_2fas_services_array(arr);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "GitHub:alice@example.com");
        assert_eq!(items[1].account, "AWS");
    }

    #[test]
    fn test_object_services_encrypted_support() {
        // Construct simulated servicesEncrypted as JSON object: { "data": "...", "salt": "...", "iv": "..." }
        let fixture_val: Value = serde_json::from_str(ENCRYPTED_2FAS_FIXTURE).unwrap();
        let (data, salt, iv) = extract_2fas_encrypted_fields(&fixture_val).unwrap();

        let obj_format = serde_json::json!({
            "schemaVersion": 4,
            "servicesEncrypted": {
                "data": data,
                "salt": salt,
                "iv": iv
            }
        });
        let obj_str = serde_json::to_string(&obj_format).unwrap();
        assert!(is_encrypted_2fas_backup(&obj_str));

        let decrypted = decrypt_2fas_backup(&obj_str, "example.com").unwrap();
        assert!(decrypted.contains("Deno"));
    }
}
