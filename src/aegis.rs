use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::Aes256Gcm;
use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde_json::Value;

use crate::import::ImportItem;

/// Check if a string payload is an encrypted Aegis backup.
pub fn is_encrypted_aegis_backup(input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return false;
    }
    if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
        if let Some(obj) = val.as_object() {
            if let (Some(header), Some(db)) = (obj.get("header"), obj.get("db")) {
                if db.is_string() {
                    if let Some(slots) = header.get("slots").and_then(|s| s.as_array()) {
                        return !slots.is_empty();
                    }
                }
            }
        }
    }
    false
}

/// Decrypt an encrypted Aegis backup payload.
pub fn decrypt_aegis_backup(input: &str, passphrase: &str) -> Result<String> {
    if passphrase.is_empty() {
        bail!("Passphrase cannot be empty");
    }

    let val: Value =
        serde_json::from_str(input.trim()).map_err(|e| anyhow!("Invalid Aegis JSON: {e}"))?;

    let header = val
        .get("header")
        .and_then(|h| h.as_object())
        .ok_or_else(|| anyhow!("Missing or invalid header in Aegis backup"))?;

    let slots = header
        .get("slots")
        .and_then(|s| s.as_array())
        .ok_or_else(|| anyhow!("Missing or invalid slots in Aegis header"))?;

    let mut master_key = None;
    for slot in slots {
        if slot.get("type").and_then(|t| t.as_i64()) != Some(1) {
            continue;
        }

        let salt_hex = match slot.get("salt").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => continue,
        };
        let salt = match hex::decode(salt_hex) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let n = match slot.get("n").and_then(|n| n.as_u64()) {
            Some(n) if n > 1 && n.is_power_of_two() => n,
            _ => continue,
        };
        let r = match slot.get("r").and_then(|r| r.as_u64()) {
            Some(r) if r > 0 => r as u32,
            _ => continue,
        };
        let p = match slot.get("p").and_then(|p| p.as_u64()) {
            Some(p) if p > 0 => p as u32,
            _ => continue,
        };

        let log_n = n.trailing_zeros() as u8;
        let params = match scrypt::Params::new(log_n, r, p, 32) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let mut derived_key = [0u8; 32];
        if scrypt::scrypt(passphrase.as_bytes(), &salt, &params, &mut derived_key).is_err() {
            continue;
        }

        let key_cipher = match Aes256Gcm::new_from_slice(&derived_key) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let key_params = match slot.get("key_params").and_then(|k| k.as_object()) {
            Some(kp) => kp,
            None => continue,
        };
        let nonce_hex = match key_params.get("nonce").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => continue,
        };
        let tag_hex = match key_params.get("tag").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => continue,
        };
        let enc_key_hex = match slot.get("key").and_then(|k| k.as_str()) {
            Some(k) => k,
            None => continue,
        };

        let nonce_bytes = match hex::decode(nonce_hex) {
            Ok(b) if b.len() == 12 => b,
            _ => continue,
        };
        let tag_bytes = match hex::decode(tag_hex) {
            Ok(b) if b.len() == 16 => b,
            _ => continue,
        };
        let enc_key_bytes = match hex::decode(enc_key_hex) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let mut ct = enc_key_bytes;
        ct.extend_from_slice(&tag_bytes);

        let nonce = match aes_gcm::Nonce::try_from(nonce_bytes.as_slice()) {
            Ok(n) => n,
            Err(_) => continue,
        };

        if let Ok(decrypted) = key_cipher.decrypt(&nonce, ct.as_slice()) {
            master_key = Some(decrypted);
            break;
        }
    }

    let master_key = master_key.ok_or_else(|| {
        anyhow!("Failed decrypting Aegis master key (incorrect passphrase or corrupted data)")
    })?;

    let params = header
        .get("params")
        .and_then(|p| p.as_object())
        .ok_or_else(|| anyhow!("Missing or invalid params in Aegis header"))?;

    let nonce_hex = params
        .get("nonce")
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow!("Missing nonce in Aegis header params"))?;
    let tag_hex = params
        .get("tag")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow!("Missing tag in Aegis header params"))?;

    let nonce_bytes =
        hex::decode(nonce_hex).map_err(|e| anyhow!("Failed decoding Aegis header nonce: {e}"))?;
    let tag_bytes =
        hex::decode(tag_hex).map_err(|e| anyhow!("Failed decoding Aegis header tag: {e}"))?;

    let db_b64 = val
        .get("db")
        .and_then(|d| d.as_str())
        .ok_or_else(|| anyhow!("Missing db ciphertext in Aegis backup"))?;

    let clean_db_b64: String = db_b64.chars().filter(|c| !c.is_whitespace()).collect();
    let mut db_ct = BASE64_STANDARD
        .decode(&clean_db_b64)
        .map_err(|e| anyhow!("Failed decoding Aegis db ciphertext: {e}"))?;
    db_ct.extend_from_slice(&tag_bytes);

    let db_cipher = Aes256Gcm::new_from_slice(&master_key)
        .map_err(|e| anyhow!("Failed initializing AES-256-GCM cipher for db: {e}"))?;
    let nonce = aes_gcm::Nonce::try_from(nonce_bytes.as_slice())
        .map_err(|_| anyhow!("Invalid Aegis db nonce length"))?;

    let decrypted_bytes = db_cipher
        .decrypt(&nonce, db_ct.as_slice())
        .map_err(|_| anyhow!("Failed decrypting Aegis database (corrupted data)"))?;

    let decrypted_str = String::from_utf8(decrypted_bytes)
        .context("Decrypted Aegis database is not valid UTF-8")?;

    // Wrap in {"db": ...} if the decrypted string is a raw db object without "db" root key
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&decrypted_str) {
        if !map.contains_key("db") && map.contains_key("entries") {
            return Ok(format!(r#"{{"db":{decrypted_str}}}"#));
        }
    }

    Ok(decrypted_str)
}

/// Parse Aegis entries array into ImportItem vector.
pub fn parse_aegis_entries(entries: &[Value]) -> Vec<ImportItem> {
    let mut items = Vec::new();
    for entry in entries {
        if let Some(info) = entry.get("info").and_then(|i| i.as_object()) {
            if let Some(secret) = info.get("secret").and_then(|s| s.as_str()) {
                let raw_name = entry
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .trim();
                let issuer = entry
                    .get("issuer")
                    .and_then(|i| i.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());

                let account = match issuer {
                    Some(iss)
                        if !raw_name.is_empty()
                            && !raw_name.to_lowercase().starts_with(&iss.to_lowercase()) =>
                    {
                        format!("{iss}:{raw_name}")
                    }
                    _ if !raw_name.is_empty() => raw_name.to_string(),
                    Some(iss) => iss.to_string(),
                    _ => "imported_account".to_string(),
                };

                let digits = info.get("digits").and_then(|d| d.as_u64());
                let period = info.get("period").and_then(|p| p.as_u64());
                let algo = info.get("algo").and_then(|a| a.as_str());
                let entry_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("totp");

                let final_secret = if digits.is_some()
                    || period.is_some()
                    || algo.is_some()
                    || entry_type == "hotp"
                {
                    let host = if entry_type == "hotp" { "hotp" } else { "totp" };
                    if let Ok(mut u) = url::Url::parse(&format!("otpauth://{host}/{account}")) {
                        u.query_pairs_mut().append_pair("secret", secret);
                        if let Some(iss) = issuer {
                            u.query_pairs_mut().append_pair("issuer", iss);
                        }
                        if let Some(d) = digits {
                            u.query_pairs_mut().append_pair("digits", &d.to_string());
                        }
                        if host == "hotp" {
                            let counter = info.get("counter").and_then(|c| c.as_u64()).unwrap_or(0);
                            u.query_pairs_mut()
                                .append_pair("counter", &counter.to_string());
                        } else if let Some(p) = period {
                            u.query_pairs_mut().append_pair("period", &p.to_string());
                        }
                        if let Some(a) = algo {
                            u.query_pairs_mut().append_pair("algorithm", a);
                        }
                        u.to_string()
                    } else {
                        secret.to_string()
                    }
                } else {
                    secret.to_string()
                };

                items.push(ImportItem {
                    account,
                    secret: final_secret,
                });
            }
        }
    }
    items
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const AEGIS_ENCRYPTED_FIXTURE: &str = r#"{
    "version": 1,
    "header": {
        "slots": [
            {
                "type": 1,
                "uuid": "a8325752-c1be-458a-9b3e-5e0a8154d9ec",
                "key": "491d44550430ba248986b904b8cffd3a6c5755d176ac877bd11b82c934225017",
                "key_params": {
                    "nonce": "e9705513ba4951fa7a0608d2",
                    "tag": "931237af257b83c693ddb8f9a7eddaf0"
                },
                "n": 32768,
                "r": 8,
                "p": 1,
                "salt": "27ea9ae53fa2f08a8dcd201615a8229422647b3058f9f36b08f9457e62888be1",
                "repaired": true
            }
        ],
        "params": {
            "nonce": "095fd13dee336fa56b4634ff",
            "tag": "5db2470edf2d12f82a89ae7f48ccd50c"
        }
    },
    "db": "RtGfUrZ01nzRnvHjPJGyWjfa6shQ7NYwa491CgAWNBM8OeGZVIHhnDAVlVWNlSoq2V097p5Yq5m+SFl5g9nBBBQBNePQnj6CCvu1NfNtoA6R3hyp77gd+e+O2MRnOGH1Z1laV2Tl6p3q8IUHWgAJ36LbUxiCXmfh7bWm198uA4bgLwrEmo04MrqeYXggLuXrJrp6dUJQFD72dgoPbHijlSycY5GLel3ZbAXRsUHszd+xdywpj7\/TYa4OYFel0M0QcCpsKA1LRQz365X9OXPJdTsmVyR4dJ6x5RIVeh39lAYKUf7T4w7BLC8taST5m4J\/VXDueKbvg8R13bNWF0aRHUgeuI9BNzMZINJlzKFKNRknTaJ\/1kEUU0sLkgcaVkX\/DVTGG+pWi5MHijicrK0i4LHN3CUwV2\/\/ZNJCGXM5ErsKMOnJfma52gMdifPiXU317Klvc5oOZFYGnhbhJ2WtPIuqjdvnfuLat2JxA7Xx3LqquRWGL2113yjzVzGBDCVY6iIdedBEgH8CGD826\/3R3m6dR5sfSggQ2SbtQA\/DZNhLSNSU+bfNScVQvUWfR2Lf7Q\/4FR\/xATAQJ9IIBeL+w2ErLUPjURocFXup5YOBHxFdDjZ2FqhbAq4h3Zn\/BJ57xUcYEA+YtP5uOP2lQwUh\/0vFWizDVotzraO8tZiBZBsODyb69eJrXNwFbIjeUczY6wrJs1+676IilbCsmtoYvWEpUZF4hIi7TYAD+nyXX\/olrkog9omWZk8R7hJ9KRDfckXEc\/XSzWhk3Kmfa7pRNh9wYZsaR7VPZGZebQMuUKfRRci2qMsZOJvQsDBJvVze0xW9SqiySDgGyRX\/DwzuaZEGZZriaLf6ox7LwY2Qi6QpYOYbAaEaXAesCR1DPxFfGKsUHVjF8hKA6ZBXDXdqM3Y+14naIOH9S7UzYn32botoVLOykSjnW6z6M0ZPkz3dwowMJiVQcyD7p+9p4J6f1S81pFS7DP+jF+PTyC3c3q\/dwFhNdoG6iV9eQEAxjUi6MpzvFRsk9RsLcQqYgzJGmRjYeXlKH8k8tTu1A4puo6w3Daz8hZz9NafMgMsuqY0oKVLgdNqFz8yVMsxYfBW\/oW56SuQyyVWyxXjXmbk1vpYCTL5kXvIZWoTmBRRDb0ay5S\/dlD6z\/WR45\/C4AwcCE9m4Yf3zisRNa7AqWLVgkmJxFdfJxjiuPtUIK79s+lIJkyRENEqkvm809qIxDhkQzY8zcCt4oXCEbJUfSG4awBs1VvilJIwe6qi0bNtqXtAb5TctgxTh29A9oGlsRG4o8sHqA1mtjp5QiLWp5Hh6rOH95W6+fnBiOW+Iw0evBTduroWvx37HBTktJz79zGe0l3c0Y6VmiFvB7knmT2CrgP7woRkxGbXxdE9zMPQJM9ursD538MVDdD\/0tdkxHxilt47f1DPo2CKUWU8Q1KMm1zLXfVO8BbGUWIv4YeDKHfMUL\/HcStv5VJY+LbnOEjzGT4e1\/avSQmqBL4G9XNkYmyMhC8tlLQcmMMH4bNfPOO3vi5Pb5E7XveSgxlOHs4F0+nqxnFOAu8494MEtx6u5+B7d8LI\/DhEO5zTDwE+THiKej6vCsFxTZ519rm67HycOwRR4LKrwfDeUEK3X1PzryOD5zcv3PMcSBgZ8EWvTfZ9ygKP8BmRQRpydTbSt8Hj5fTUuajADCP0Ggw+6G7n+5FhExJNd+o9D8d4KgLPOe08M8InW7pLB389TWtSo4v3VNjcmmJNQ26wlPkhO\/xBU1URFR0fXU3eCO+w++IMt\/fOSqSpNF9bWElfWHIQ23ntxVke\/hR9j\/GG3tHGxYS5pL42sJF\/Re\/UlUJTGSQP6up2xVYs6gncQ0zACDOPjLQmQzYhz\/hr8S6EjYfK++yLZmRTjEI7xT9u\/B5YLyOQCYVTaF\/pDEegjsehXM3qJBfsA+XY7F9TRsmM\/MSVaPDkdIJ7zvL9xtaF6bXdZoZ6po3ml8uu41pSkNmMKgyEy5E0UQUTWMPLC8drUoQ\/KWQnVIN6HUXGBjYy6aax\/LYZaBcbZi97FHK0h+wsx3WN\/uQozNkQjwGYE8fwYxRYh1RaFi5PkiCM505ib7e82Yuts0l+cBb6nG1IruDplg9BD\/G9w4vVDePEikhcPyY\/p7AZ4i7u\/bL2YKlbE3HyJa+7dkbWJgGidtRZgu+Fdl2T\/rrRJ4+lVaKPVKGKT7ItZdIeitIYUdRxCzrOf1ItZCC8BWa4PElDAjj2yDNmMYRpXJBe3gQHWs\/H5SZgFuwsfCu23uzNRQYib8SuwIJQDvPiXo7m4oIySO8VyvemcExlbXSlbZbvwVxYavTVfcUpAXI6qlsg2jjk+JZahfKrWNC5COZPdVjdAXCoiKU+HBPmEFCwQv\/7zlSBEiI2piyqd+MPwnP63RdGO+oXYid6hn4Nm8kcOhtRyvYm95p66jzGlEugsfxJCED7MTh3XShqa2tt4lFG25icllzTvIJboRkz5oIB4dZVS9+q2TgGUoX7UCpobD8WkHo\/y0cpTuZr8vzXqx2fObxzPNoVgxJmp9E06G2bhMVHPpT17xbfq\/KhJJn7k1S0sfXPG+SmYlX4U7zNSe1M7JXtLf3uVOLz7Ccjp3yvcdq8nRmVym3Zwsz+vv57FA2A0dy3Db97ypJa9HGaxnnYIZHHzep0gJCeeIKE9L32zGCoUg+cPu9B2lPEgIr64iGiuvKSRwNQpOBktM6qqjQntE0Me6mh426irFQ\/3tcfH9a4lZEwwuU1X+lUBUWQp3n5Ej4BSJEs8E6H0EjBvyk69q3qjy5yi7ROVRis6y6S1v4er77RHQUf3phK5354VJHrp9pR926t5qngH5RVF4eljwtXDs3MkejADJ6stBHa\/w7FcbUClO8U+S4Bidxb3mZCiZkUVTpbzvBfYAiQvAfdkMa49o3a5DXKsbXyUPrmr6fWRfM1fS0Ehp0lUv6BDj0yR13CLMpKDU4GfDrl8UEvwh7gwtBRkuaBFzyMtd3NeE7kIGf9vFs6MEl2dmMDFSDid7MdVSDVTlhaAtp+zsRejKW3OQr5n051FzkUsIFGty9AWOkwjZCbstHYCOtyJnsnXP1i9lRDFBgPpFgmDD+bzzg0g9AOAxzqTiLF7bb1jejfe5qVr5V9+7zLpwRLiYaLkNOmpsqvNMuYVwdqTp6nyoougdgBlvve3EG0k09sFKi2Ep9lq+QkS7zGre2jJDrqgdC08+V4PXHYkP3V3Zjgn1x6RfQ2PE+2zvk1GGEgzcNww3byoYw0Ra5qS5yftMy\/2WahbA8fjUYvtmksFH8VjN3yasZt3sdQLWtv8qXxZscy+pCyjTdyxW+ddFnrWuqMIV3jbGMvngq6dL\/n5+DumjbA1gmBJVOpmyEsc1iwHDS36cNnyi1htGFO\/6\/Va4YPYK7dG6LY387UoBUU9Q9ijrBrSGpzPWYmXBLZ8e1MMPfHIN1WsaTgYO9leg3MAJTjQFTFrQ5dguYpWhlm2sWJT45jrda4uWqduB+aQLzYRWhEDBFzPV3ZgIe0SB+7h04Vm0Pu\/LDRvqaolpZ86CEm+zgjBOKeEGFwzTXxH\/5pBoca1bZ6wvsbVZxJNBeH8\/w=="
}"#;

    #[test]
    fn test_is_encrypted_aegis_backup() {
        assert!(is_encrypted_aegis_backup(AEGIS_ENCRYPTED_FIXTURE));
        assert!(!is_encrypted_aegis_backup(r#"{"version": 1, "db": {}}"#));
        assert!(!is_encrypted_aegis_backup(r#"{"other": 123}"#));
        assert!(!is_encrypted_aegis_backup("not json"));
    }

    #[test]
    fn test_decrypt_aegis_backup_success() {
        let decrypted = decrypt_aegis_backup(AEGIS_ENCRYPTED_FIXTURE, "test").unwrap();
        assert!(decrypted.contains("entries"));

        let items = crate::import::parse_import_payload(&decrypted).unwrap();
        assert_eq!(items.len(), 7);
        assert_eq!(items[0].account, "Deno:Mason");
        assert_eq!(items[1].account, "SPDX:James");
        assert_eq!(items[2].account, "Airbnb:Elijah");
        assert_eq!(items[3].account, "Issuu:James");
        assert_eq!(items[4].account, "Air Canada:Benjamin");
        assert_eq!(items[5].account, "WWE:Mason");
        assert_eq!(items[6].account, "Boeing:Sophia");

        for item in &items {
            let engine = crate::totp::TfaTotp::from_input(&item.account, &item.secret).unwrap();
            let code = engine.generate_current();
            assert!(!code.is_empty());
        }
    }

    #[test]
    fn test_decrypt_aegis_backup_wrong_password_fails() {
        let err = decrypt_aegis_backup(AEGIS_ENCRYPTED_FIXTURE, "wrong_password");
        assert!(err.is_err());
    }

    #[test]
    fn test_decrypt_aegis_backup_empty_password_fails() {
        let err = decrypt_aegis_backup(AEGIS_ENCRYPTED_FIXTURE, "");
        assert!(err.is_err());
    }
}
