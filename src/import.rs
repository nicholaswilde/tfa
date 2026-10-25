use anyhow::{bail, Result};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportItem {
    pub account: String,
    pub secret: String,
}

/// Parse import payload from string (migration URL, OTPAuth URLs, or JSON export)
pub fn parse_import_payload(input: &str) -> Result<Vec<ImportItem>> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        bail!("Import payload is empty");
    }

    // 1. Google Authenticator migration payload
    if trimmed.starts_with("otpauth-migration://") {
        return parse_google_migration(trimmed);
    }

    // 2. Try JSON (tfa export, Bitwarden, Aegis, key-value map)
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if let Ok(items) = parse_json_export(trimmed) {
            if !items.is_empty() {
                return Ok(items);
            }
        }
    }

    // 3. Line-separated otpauth:// or otpauth-migration:// URIs
    let mut items = Vec::new();
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("otpauth-migration://") {
            let mut mig_items = parse_google_migration(line)?;
            items.append(&mut mig_items);
        } else if line.starts_with("otpauth://") {
            let account = extract_account_from_otpauth(line);
            items.push(ImportItem {
                account,
                secret: line.to_string(),
            });
        }
    }

    if items.is_empty() {
        bail!("Could not identify valid 2FA secrets or accounts in import payload");
    }

    Ok(items)
}

fn parse_google_migration(uri: &str) -> Result<Vec<ImportItem>> {
    let accounts = google_authenticator_converter::process_data(uri)
        .map_err(|e| anyhow::anyhow!("Failed parsing Google Authenticator migration URL: {e}"))?;

    let items = accounts
        .into_iter()
        .map(|acc| {
            let account_name = if acc.name.is_empty() {
                acc.issuer
            } else {
                acc.name
            };
            ImportItem {
                account: account_name,
                secret: acc.secret,
            }
        })
        .collect();

    Ok(items)
}

fn parse_json_export(json_str: &str) -> Result<Vec<ImportItem>> {
    let v: Value = serde_json::from_str(json_str)?;
    let mut items = Vec::new();

    match v {
        // Simple map: { "account_name": "secret_or_url", ... }
        Value::Object(map) => {
            // Check for Bitwarden format: { "items": [ { "name": "...", "login": { "totp": "..." } } ] }
            if let Some(Value::Array(bw_items)) = map.get("items") {
                for item in bw_items {
                    if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                        if let Some(totp) = item
                            .get("login")
                            .and_then(|l| l.get("totp"))
                            .and_then(|t| t.as_str())
                        {
                            if !totp.is_empty() {
                                items.push(ImportItem {
                                    account: name.to_string(),
                                    secret: totp.to_string(),
                                });
                            }
                        }
                    }
                }
                if !items.is_empty() {
                    return Ok(items);
                }
            }

            // Check for Aegis format: { "db": { "entries": [ { "name": "...", "info": { "secret": "..." } } ] } }
            if let Some(entries) = map
                .get("db")
                .and_then(|db| db.get("entries"))
                .and_then(|e| e.as_array())
            {
                for entry in entries {
                    if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                        if let Some(secret) = entry
                            .get("info")
                            .and_then(|i| i.get("secret"))
                            .and_then(|s| s.as_str())
                        {
                            items.push(ImportItem {
                                account: name.to_string(),
                                secret: secret.to_string(),
                            });
                        }
                    }
                }
                if !items.is_empty() {
                    return Ok(items);
                }
            }

            // Generic Key-Value map
            for (key, val) in map {
                if let Some(secret_str) = val.as_str() {
                    items.push(ImportItem {
                        account: key,
                        secret: secret_str.to_string(),
                    });
                }
            }
        }
        // Array of items: [ { "name": "...", "secret": "..." } ]
        Value::Array(arr) => {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    let name = obj
                        .get("account")
                        .or_else(|| obj.get("name"))
                        .and_then(|n| n.as_str());
                    let secret = obj.get("secret").and_then(|s| s.as_str());
                    if let (Some(n), Some(s)) = (name, secret) {
                        items.push(ImportItem {
                            account: n.to_string(),
                            secret: s.to_string(),
                        });
                    }
                }
            }
        }
        _ => {}
    }

    Ok(items)
}

pub fn extract_account_from_otpauth(url_str: &str) -> String {
    if let Ok(url) = url::Url::parse(url_str) {
        let path = url.path().trim_start_matches('/');
        if let Some(acc) = path.split(':').next_back() {
            if !acc.is_empty() {
                return acc.to_string();
            }
        }
        if !path.is_empty() {
            return path.to_string();
        }
    }
    "imported_account".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_google_authenticator_migration() {
        let uri = "otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8SGFRlc3QxOnRlc3QxQGV4YW1wbGUxLmNvbRoFVGVzdDEgASgBMAIKMwoKSGVsbG8h3q2%2B8BIYVGVzdDI6dGVzdDJAZXhhbXBsZTIuY29tGgVUZXN0MiABKAEwAgozCgpIZWxsbyHerb7xEhhUZXN0Mzp0ZXN0M0BleGFtcGxlMy5jb20aBVRlc3QzIAEoATACEAEYASAAKI3orYEE";
        let items = parse_import_payload(uri).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].account, "Test1:test1@example1.com");
        assert!(!items[0].secret.is_empty());
        assert_eq!(items[1].account, "Test2:test2@example2.com");
        assert_eq!(items[2].account, "Test3:test3@example3.com");

        // Verify valid secret for TfaTotp
        let engine = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        let code = engine.generate_current();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_import_json_key_value_map() {
        let json = r#"{
            "github": "JBSWY3DPEHPK3PXP",
            "aws": "otpauth://totp/Amazon:user?secret=JBSWY3DPEHPK3PXP&issuer=Amazon"
        }"#;
        let items = parse_import_payload(json).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_import_bitwarden_format() {
        let json = r#"{
            "items": [
                {
                    "name": "Bitwarden Account",
                    "login": {
                        "totp": "otpauth://totp/BW:user?secret=JBSWY3DPEHPK3PXP&issuer=BW"
                    }
                },
                {
                    "name": "No 2FA",
                    "login": {}
                }
            ]
        }"#;
        let items = parse_import_payload(json).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].account, "Bitwarden Account");
        assert!(items[0].secret.contains("JBSWY3DPEHPK3PXP"));
    }

    #[test]
    fn test_import_otpauth_lines() {
        let lines = "otpauth://totp/GitHub:alice?secret=JBSWY3DPEHPK3PXP&issuer=GitHub\notpauth://totp/AWS:bob?secret=JBSWY3DPEHPK3PXP&issuer=AWS\n";
        let items = parse_import_payload(lines).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "alice");
        assert_eq!(items[1].account, "bob");
    }

    #[test]
    fn test_import_aegis_format() {
        let json = r#"{
            "db": {
                "entries": [
                    {
                        "name": "Aegis Account",
                        "info": {
                            "secret": "JBSWY3DPEHPK3PXP"
                        }
                    }
                ]
            }
        }"#;
        let items = parse_import_payload(json).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].account, "Aegis Account");
        assert_eq!(items[0].secret, "JBSWY3DPEHPK3PXP");
    }

    #[test]
    fn test_import_array_format() {
        let json = r#"[
            {"name": "entry1", "secret": "SEC1"},
            {"account": "entry2", "secret": "SEC2"}
        ]"#;
        let items = parse_import_payload(json).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "entry1");
        assert_eq!(items[1].account, "entry2");
    }

    #[test]
    fn test_import_empty_or_invalid_payload_fails() {
        assert!(parse_import_payload("").is_err());
        assert!(parse_import_payload("   ").is_err());
        assert!(parse_import_payload("just plain text without uri").is_err());
        assert!(parse_import_payload("{}").is_err());
        assert!(parse_import_payload("[]").is_err());
        assert!(parse_import_payload(r#"{"items": []}"#).is_err());
        assert!(parse_import_payload(r#"{"db": {"entries": []}}"#).is_err());
        assert!(parse_import_payload(r#"[{"no_secret": 123}, 42]"#).is_err());
    }

    #[test]
    fn test_import_multiline_with_comments_and_migration() {
        let migration_uri = "otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8SGFRlc3QxOnRlc3QxQGV4YW1wbGUxLmNvbRoFVGVzdDEgASgBMAIKMwoKSGVsbG8h3q2%2B8BIYVGVzdDI6dGVzdDJAZXhhbXBsZTIuY29tGgVUZXN0MiABKAEwAgozCgpIZWxsbyHerb7xEhhUZXN0Mzp0ZXN0M0BleGFtcGxlMy5jb20aBVRlc3QzIAEoATACEAEYASAAKI3orYEE";
        let content = format!(
            "# Header comment\n\n{}\n# Another comment\notpauth://totp/alice?secret=JBSWY3DPEHPK3PXP\n",
            migration_uri
        );
        let items = parse_import_payload(&content).unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[3].account, "alice");
    }

    #[test]
    fn test_extract_account_from_otpauth_variations() {
        assert_eq!(
            extract_account_from_otpauth("otpauth://totp/single"),
            "single"
        );
        assert_eq!(
            extract_account_from_otpauth("otpauth://totp/issuer:user"),
            "user"
        );
        assert_eq!(
            extract_account_from_otpauth("otpauth://totp/"),
            "imported_account"
        );
        assert_eq!(
            extract_account_from_otpauth("://invalid"),
            "imported_account"
        );
    }
}
