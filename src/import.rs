use anyhow::{bail, Result};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportItem {
    pub account: String,
    pub secret: String,
}

/// Construct a canonical `Issuer:account` label when both fields are present and distinct.
pub fn canonical_account_label(issuer: Option<&str>, account: Option<&str>) -> String {
    let iss = issuer
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("tfa"));
    let acc = account.map(str::trim).filter(|s| !s.is_empty());

    match (iss, acc) {
        (Some(i), Some(a)) => {
            // Check if already in Issuer:account format with the same issuer
            if let Some((existing_i, _)) = a.split_once(':') {
                if existing_i.trim().eq_ignore_ascii_case(i) {
                    return a.to_string();
                }
            }
            if a.eq_ignore_ascii_case(i) {
                i.to_string()
            } else {
                format!("{i}:{a}")
            }
        }
        (Some(i), None) => i.to_string(),
        (None, Some(a)) => a.to_string(),
        (None, None) => "imported_account".to_string(),
    }
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

    // 3. Try CSV (Bitwarden, 1Password)
    if let Some(items) = parse_csv_export(trimmed) {
        if !items.is_empty() {
            return Ok(items);
        }
    }

    // 4. Line-separated otpauth:// or otpauth-migration:// URIs
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

    if !items.is_empty() {
        return Ok(items);
    }

    // 5. Try YAML (totp-cli dump or key-value map)
    if let Ok(v) = serde_yaml_ng::from_str::<Value>(trimmed) {
        if let Ok(yaml_items) = parse_value_export(v) {
            if !yaml_items.is_empty() {
                return Ok(yaml_items);
            }
        }
    }

    bail!("Could not identify valid 2FA secrets or accounts in import payload")
}

fn parse_google_migration(uri: &str) -> Result<Vec<ImportItem>> {
    let accounts = google_authenticator_converter::process_data(uri)
        .map_err(|e| anyhow::anyhow!("Failed parsing Google Authenticator migration URL: {e}"))?;

    let items = accounts
        .into_iter()
        .map(|acc| {
            let account_name = canonical_account_label(
                if acc.issuer.is_empty() {
                    None
                } else {
                    Some(&acc.issuer)
                },
                if acc.name.is_empty() {
                    None
                } else {
                    Some(&acc.name)
                },
            );
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
    parse_value_export(v)
}

fn parse_value_export(v: Value) -> Result<Vec<ImportItem>> {
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
                let aegis_items = crate::aegis::parse_aegis_entries(entries);
                if !aegis_items.is_empty() {
                    return Ok(aegis_items);
                }
            }

            // Check for direct Aegis entries format: { "entries": [ { "name": "...", "info": { "secret": "..." } } ] }
            if let Some(Value::Array(entries)) = map.get("entries") {
                let aegis_items = crate::aegis::parse_aegis_entries(entries);
                if !aegis_items.is_empty() {
                    return Ok(aegis_items);
                }
            }

            // Check for 2FAS format: { "services": [ { "name": "...", "secret": "...", "otp": { ... } } ] }
            if let Some(services) = map.get("services").and_then(|s| s.as_array()) {
                let twofas_items = crate::twofas::parse_2fas_services_array(services);
                if !twofas_items.is_empty() {
                    return Ok(twofas_items);
                }
            }

            // Check for LastPass Authenticator format: { "accounts": [ { "issuerName": "...", "userName": "...", "secret": "..." } ] }
            if let Some(Value::Array(lp_accounts)) = map.get("accounts") {
                for item in lp_accounts {
                    if let Some(obj) = item.as_object() {
                        if let Some(imp) = build_item_from_obj(obj) {
                            items.push(imp);
                        }
                    }
                }
                if !items.is_empty() {
                    return Ok(items);
                }
            }

            // Check for Proton Authenticator format: { "entries": [ { "content": { "name": "...", "uri": "otpauth://..." } } ] }
            if let Some(Value::Array(proton_entries)) = map.get("entries") {
                for entry in proton_entries {
                    if let Some(content) = entry.get("content").and_then(|c| c.as_object()) {
                        if let Some(uri) = content
                            .get("uri")
                            .and_then(|u| u.as_str())
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                        {
                            let name = content
                                .get("name")
                                .and_then(|n| n.as_str())
                                .map(str::trim)
                                .filter(|s| !s.is_empty());
                            let extracted = extract_account_from_otpauth(uri);
                            let account = if extracted != "imported_account" {
                                canonical_account_label(name, Some(&extracted))
                            } else if let Some(n) = name {
                                n.to_string()
                            } else {
                                extracted
                            };
                            items.push(ImportItem {
                                account,
                                secret: uri.to_string(),
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
            // Check for decrypted 2FAS services array
            if crate::twofas::is_2fas_services(&Value::Array(arr.clone())) {
                let twofas_items = crate::twofas::parse_2fas_services_array(&arr);
                if !twofas_items.is_empty() {
                    return Ok(twofas_items);
                }
            }

            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(ns_accounts) = obj.get("accounts").and_then(|a| a.as_array()) {
                        let ns_name = obj
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("default");
                        for acc_val in ns_accounts {
                            if let Some(acc_obj) = acc_val.as_object() {
                                if let Some(mut imp) = build_item_from_obj(acc_obj) {
                                    imp.account =
                                        crate::namespace::scoped_key(ns_name, &imp.account);
                                    items.push(imp);
                                }
                            }
                        }
                        continue;
                    }
                    if let Some(imp) = build_item_from_obj(obj) {
                        items.push(imp);
                    }
                }
            }
        }
        _ => {}
    }

    Ok(items)
}

fn build_item_from_obj(obj: &serde_json::Map<String, Value>) -> Option<ImportItem> {
    let raw_name = obj
        .get("account")
        .or_else(|| obj.get("name"))
        .or_else(|| obj.get("label"))
        .or_else(|| obj.get("userName"))
        .or_else(|| obj.get("originalUserName"))
        .and_then(|n| n.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let issuer = obj
        .get("issuer")
        .or_else(|| obj.get("issuerName"))
        .or_else(|| obj.get("originalIssuerName"))
        .and_then(|i| i.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let secret_str = obj
        .get("secret")
        .or_else(|| obj.get("token"))
        .and_then(|s| s.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    let prefix = obj
        .get("prefix")
        .and_then(|p| p.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let account_name = if issuer.is_none() && raw_name.is_none() {
        return None;
    } else {
        canonical_account_label(issuer, raw_name)
    };

    let has_otp_meta = issuer.is_some()
        || prefix.is_some()
        || obj.contains_key("type")
        || obj.contains_key("kind")
        || obj.contains_key("digits")
        || obj.contains_key("length")
        || obj.contains_key("period")
        || obj.contains_key("timer")
        || obj.contains_key("timeStep")
        || obj.contains_key("timePeriod")
        || obj.contains_key("algorithm");

    let final_secret = if has_otp_meta && !secret_str.starts_with("otpauth://") {
        let otp_type = obj
            .get("type")
            .or_else(|| obj.get("kind"))
            .and_then(|t| t.as_str())
            .unwrap_or("TOTP")
            .to_uppercase();
        let host = match otp_type.as_str() {
            "HOTP" => "hotp",
            "STEAM" => "steam",
            _ => "totp",
        };
        if let Ok(mut u) = url::Url::parse(&format!("otpauth://{host}/{account_name}")) {
            u.query_pairs_mut().append_pair("secret", secret_str);
            if let Some(iss) = issuer {
                u.query_pairs_mut().append_pair("issuer", iss);
            }
            if let Some(alg) = obj.get("algorithm").and_then(|a| a.as_str()) {
                let alg_trimmed = alg.trim();
                if !alg_trimmed.is_empty() {
                    u.query_pairs_mut()
                        .append_pair("algorithm", &alg_trimmed.to_uppercase());
                }
            }
            if let Some(d) = val_to_u64(obj.get("digits").or_else(|| obj.get("length"))) {
                u.query_pairs_mut().append_pair("digits", &d.to_string());
            }
            if host == "hotp" {
                let c = val_to_u64(obj.get("counter")).unwrap_or(0);
                u.query_pairs_mut().append_pair("counter", &c.to_string());
            } else if let Some(p) = val_to_u64(
                obj.get("period")
                    .or_else(|| obj.get("timer"))
                    .or_else(|| obj.get("timeStep"))
                    .or_else(|| obj.get("timePeriod")),
            ) {
                u.query_pairs_mut().append_pair("period", &p.to_string());
            }
            if let Some(pfx) = prefix {
                u.query_pairs_mut().append_pair("prefix", pfx);
            }
            u.to_string()
        } else {
            secret_str.to_string()
        }
    } else {
        secret_str.to_string()
    };

    Some(ImportItem {
        account: account_name,
        secret: final_secret,
    })
}

fn val_to_u64(v: Option<&Value>) -> Option<u64> {
    v.and_then(|x| {
        x.as_u64()
            .or_else(|| x.as_str().and_then(|s| s.trim().parse().ok()))
    })
}

fn parse_csv_export(input: &str) -> Option<Vec<ImportItem>> {
    let first_line = input.lines().next()?;
    if !first_line.contains(',') || first_line.trim_start().starts_with("otpauth") {
        return None;
    }

    let records = parse_csv_records(input);
    let headers = records.first()?;
    let norm_headers: Vec<String> = headers
        .iter()
        .map(|h| h.trim().trim_start_matches('\u{feff}').to_lowercase())
        .collect();

    let totp_idx = norm_headers.iter().position(|h| {
        matches!(
            h.as_str(),
            "login_totp" | "one-time password" | "otpauth" | "totp"
        )
    })?;
    let name_idx = norm_headers
        .iter()
        .position(|h| matches!(h.as_str(), "name" | "title"));
    let user_idx = norm_headers
        .iter()
        .position(|h| matches!(h.as_str(), "login_username" | "username"));

    let mut items = Vec::new();
    for row in records.iter().skip(1) {
        let Some(secret) = row
            .get(totp_idx)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let name = name_idx
            .and_then(|i| row.get(i))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let user = user_idx
            .and_then(|i| row.get(i))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        let account = if let Some(n) = name {
            n.to_string()
        } else if let Some(u) = user {
            u.to_string()
        } else if secret.starts_with("otpauth://") {
            extract_account_from_otpauth(secret)
        } else {
            "imported_account".to_string()
        };

        items.push(ImportItem {
            account,
            secret: secret.to_string(),
        });
    }

    Some(items)
}

fn parse_csv_records(input: &str) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' if field.trim().is_empty() => {
                    field.clear();
                    in_quotes = true;
                }
                ',' => row.push(std::mem::take(&mut field)),
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    row.push(std::mem::take(&mut field));
                    if row.iter().any(|s| !s.trim().is_empty()) {
                        records.push(std::mem::take(&mut row));
                    } else {
                        row.clear();
                    }
                }
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    if row.iter().any(|s| !s.trim().is_empty()) {
                        records.push(std::mem::take(&mut row));
                    } else {
                        row.clear();
                    }
                }
                _ => field.push(c),
            }
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        if row.iter().any(|s| !s.trim().is_empty()) {
            records.push(row);
        }
    }
    records
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push(((h << 4) | l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn extract_account_from_otpauth(url_str: &str) -> String {
    if let Ok(url) = url::Url::parse(url_str) {
        let path = percent_decode(url.path().trim_start_matches('/'));
        let mut query_issuer = None;
        for (k, v) in url.query_pairs() {
            if k.eq_ignore_ascii_case("issuer") {
                let trimmed = v.trim();
                if !trimmed.is_empty() {
                    query_issuer = Some(trimmed.to_string());
                }
            }
        }

        if let Some((path_issuer, path_acc)) = path.split_once(':') {
            let pi = path_issuer.trim();
            let pa = path_acc.trim();
            let chosen_issuer = query_issuer
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(pi);
            let account_part = if !pa.is_empty() { Some(pa) } else { None };
            let result = canonical_account_label(
                if !chosen_issuer.is_empty() {
                    Some(chosen_issuer)
                } else {
                    None
                },
                account_part,
            );
            if result != "imported_account" {
                return result;
            }
        }

        let trimmed_path = path.trim();
        let account_part = if !trimmed_path.is_empty() {
            Some(trimmed_path)
        } else {
            None
        };
        let result = canonical_account_label(query_issuer.as_deref(), account_part);
        if result != "imported_account" {
            return result;
        }
    }
    "imported_account".to_string()
}

/// Compare two secrets for equivalence, resolving TOTP/HOTP URLs and raw keys.
pub fn are_secrets_equivalent(s1: &str, s2: &str) -> bool {
    let t1 = s1.trim();
    let t2 = s2.trim();
    if t1 == t2 {
        return true;
    }

    match (
        crate::totp::TfaTotp::from_input("tmp", t1),
        crate::totp::TfaTotp::from_input("tmp", t2),
    ) {
        (Ok(e1), Ok(e2)) => {
            e1.secret_bytes() == e2.secret_bytes()
                && e1.digits() == e2.digits()
                && e1.step() == e2.step()
                && e1.algorithm() == e2.algorithm()
                && e1.is_hotp() == e2.is_hotp()
                && e1.counter() == e2.counter()
                && e1.prefix() == e2.prefix()
        }
        _ => false,
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ResolvedImportAction {
    /// New account to import
    ImportNew { account: String },
    /// Colliding account with differing secret, auto-suffixed (<account>-2, <account>-3)
    ImportSuffixed { account: String, original: String },
    /// Existing account to update (--overwrite)
    UpdateExisting { account: String },
    /// Existing account with identical secret (true duplicate)
    SkipDuplicate { account: String },
    /// Colliding account with differing secret skipped (--skip)
    SkipExisting { account: String },
}

/// Tracker for batch import collision resolution and deduplication.
#[derive(Default)]
pub struct ImportCollisionTracker {
    assigned: HashMap<String, String>,
}

impl ImportCollisionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve action for an incoming import item against the store and current batch history.
    pub fn resolve_item(
        &mut self,
        store: &dyn crate::storage::AccountStore,
        item: &ImportItem,
        overwrite: bool,
        skip: bool,
    ) -> ResolvedImportAction {
        let base_name = &item.account;

        // Check if base_name already exists in this batch or in store
        let existing_secret = self
            .assigned
            .get(base_name)
            .cloned()
            .or_else(|| store.get_secret(base_name).ok());

        match existing_secret {
            None => {
                // Not found anywhere: brand new account
                self.assigned.insert(base_name.clone(), item.secret.clone());
                ResolvedImportAction::ImportNew {
                    account: base_name.clone(),
                }
            }
            Some(ref existing_sec) => {
                // Base name already exists.
                // 1. If overwrite is explicitly set, update the existing account
                if overwrite {
                    self.assigned.insert(base_name.clone(), item.secret.clone());
                    return ResolvedImportAction::UpdateExisting {
                        account: base_name.clone(),
                    };
                }

                // 2. Secret deduplication: check if identical secret
                if are_secrets_equivalent(existing_sec, &item.secret) {
                    return ResolvedImportAction::SkipDuplicate {
                        account: base_name.clone(),
                    };
                }

                // 3. Secret differs. Check skip flag.
                if skip {
                    ResolvedImportAction::SkipExisting {
                        account: base_name.clone(),
                    }
                } else {
                    // 3. Default: Collision Auto-Suffixing (<account>-2, <account>-3, ...)
                    // First check if any existing suffixed version already holds this identical secret
                    let mut n = 2;
                    loop {
                        let candidate = format!("{base_name}-{n}");
                        let cand_sec = self
                            .assigned
                            .get(&candidate)
                            .cloned()
                            .or_else(|| store.get_secret(&candidate).ok());

                        match cand_sec {
                            Some(ref s) if are_secrets_equivalent(s, &item.secret) => {
                                return ResolvedImportAction::SkipDuplicate {
                                    account: base_name.clone(),
                                };
                            }
                            Some(_) => {
                                n += 1;
                            }
                            None => {
                                self.assigned.insert(candidate.clone(), item.secret.clone());
                                return ResolvedImportAction::ImportSuffixed {
                                    account: candidate,
                                    original: base_name.clone(),
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::AccountStore;

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
        assert_eq!(items[0].account, "GitHub:alice");
        assert_eq!(items[1].account, "AWS:bob");
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
            "issuer:user"
        );
        assert_eq!(
            extract_account_from_otpauth("otpauth://totp/user?issuer=GitHub"),
            "GitHub:user"
        );
        assert_eq!(
            extract_account_from_otpauth("otpauth://totp/GitHub:user?issuer=GitHub"),
            "GitHub:user"
        );
        assert_eq!(
            extract_account_from_otpauth("otpauth://totp/GitHub:GitHub?issuer=GitHub"),
            "GitHub"
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

    #[test]
    fn test_import_2fas_unencrypted_json() {
        let json = r#"{
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
                }
            ]
        }"#;
        let items = parse_import_payload(json).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].account, "GitHub:alice@example.com");
        assert!(items[0].secret.contains("JBSWY3DPEHPK3PXP"));
    }

    #[test]
    fn test_import_ente_auth_text() {
        let ente_txt = "\
otpauth://totp/GitHub%3Aalice%40example.com?period=30&digits=6&algorithm=SHA1&secret=JBSWY3DPEHPK3PXP&issuer=GitHub&codeDisplay=%7B%22pinned%22%3Atrue%7D
otpauth://totp/Proton%20Mail:bob%40proton.me?secret=jbswy3dpehpk3pxp====&issuer=Proton%20Mail&algorithm=SHA256&digits=6&period=30&codeDisplay=%7B%22pinned%22%3Afalse%7D
";
        let items = parse_import_payload(ente_txt).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "GitHub:alice@example.com");
        assert_eq!(items[1].account, "Proton Mail:bob@proton.me");

        for item in &items {
            let engine = crate::totp::TfaTotp::from_input(&item.account, &item.secret).unwrap();
            assert_eq!(engine.generate_current().len(), 6);
        }
    }

    #[test]
    fn test_import_stratum_text() {
        let stratum_txt = "\
otpauth://totp/GitLab:dev%40example.com?secret=JBSWY3DPEHPK3PXP&issuer=GitLab&algorithm=SHA256&digits=8&period=30
otpauth://hotp/YubiKey:admin?secret=JBSWY3DPEHPK3PXP&issuer=YubiKey&algorithm=SHA1&digits=6&counter=42
otpauth://steam/Steam:gamer?secret=JBSWY3DPEHPK3PXP&issuer=Steam&algorithm=SHA1&digits=5&period=30
otpauth://totp/Steam:gamer2?secret=JBSWY3DPEHPK3PXP&issuer=Steam&encoder=steam
";
        let items = parse_import_payload(stratum_txt).unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].account, "GitLab:dev@example.com");
        assert_eq!(items[1].account, "YubiKey:admin");
        assert_eq!(items[2].account, "Steam:gamer");
        assert_eq!(items[3].account, "Steam:gamer2");

        let eng0 = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(eng0.digits(), 8);
        assert_eq!(eng0.algorithm(), totp_rs::Algorithm::SHA256);

        let eng1 = crate::totp::TfaTotp::from_input(&items[1].account, &items[1].secret).unwrap();
        assert!(eng1.is_hotp());
        assert_eq!(eng1.counter(), Some(42));

        let eng2 = crate::totp::TfaTotp::from_input(&items[2].account, &items[2].secret).unwrap();
        assert_eq!(eng2.account_type(), "Steam");
        assert_eq!(eng2.digits(), 5);

        let eng3 = crate::totp::TfaTotp::from_input(&items[3].account, &items[3].secret).unwrap();
        assert_eq!(eng3.account_type(), "Steam");
        assert_eq!(eng3.digits(), 5);
    }

    #[test]
    fn test_import_andotp_json() {
        let andotp_json = r#"[
            {
                "secret": "JBSWY3DPEHPK3PXP",
                "issuer": "GitHub",
                "label": "alice@example.com",
                "digits": 6,
                "type": "TOTP",
                "algorithm": "SHA1",
                "period": 30,
                "counter": 0
            },
            {
                "secret": "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
                "issuer": "GitLab",
                "label": "octocat",
                "digits": 8,
                "type": "TOTP",
                "algorithm": "SHA256",
                "period": 60,
                "counter": 0
            },
            {
                "secret": "JBSWY3DPEHPK3PXP",
                "issuer": "Yubi",
                "label": "hwkey",
                "digits": 6,
                "type": "HOTP",
                "algorithm": "SHA1",
                "period": 30,
                "counter": 7
            }
        ]"#;

        let items = parse_import_payload(andotp_json).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].account, "GitHub:alice@example.com");
        assert_eq!(items[1].account, "GitLab:octocat");
        assert_eq!(items[2].account, "Yubi:hwkey");

        let eng0 = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(eng0.digits(), 6);
        assert_eq!(eng0.step(), 30);

        let eng1 = crate::totp::TfaTotp::from_input(&items[1].account, &items[1].secret).unwrap();
        assert_eq!(eng1.digits(), 8);
        assert_eq!(eng1.step(), 60);
        assert_eq!(eng1.algorithm(), totp_rs::Algorithm::SHA256);

        let eng2 = crate::totp::TfaTotp::from_input(&items[2].account, &items[2].secret).unwrap();
        assert!(eng2.is_hotp());
        assert_eq!(eng2.counter(), Some(7));
    }

    #[test]
    fn test_import_raivo_otp_json() {
        let raivo_json = r#"[
            {
                "kind": "TOTP",
                "account": "google@gmail.com",
                "secret": "JBSWY3DPEHPK3PXP",
                "iconType": "raivo_repository",
                "issuer": "Google.com",
                "timer": "60",
                "digits": "8",
                "counter": "0",
                "algorithm": "SHA256",
                "iconValue": "google.com/google-gmail.png"
            },
            {
                "kind": "HOTP",
                "account": "user@example.com",
                "secret": "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
                "iconType": "custom",
                "issuer": "Example",
                "timer": "30",
                "digits": "6",
                "counter": "3",
                "algorithm": "SHA1",
                "iconValue": ""
            }
        ]"#;

        let items = parse_import_payload(raivo_json).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "Google.com:google@gmail.com");
        assert_eq!(items[1].account, "Example:user@example.com");

        let eng0 = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(eng0.digits(), 8);
        assert_eq!(eng0.step(), 60);
        assert_eq!(eng0.algorithm(), totp_rs::Algorithm::SHA256);

        let eng1 = crate::totp::TfaTotp::from_input(&items[1].account, &items[1].secret).unwrap();
        assert!(eng1.is_hotp());
        assert_eq!(eng1.counter(), Some(3));
    }

    #[test]
    fn test_import_lastpass_json() {
        let lp_json = r#"{
            "version": 1,
            "accounts": [
                {
                    "accountID": "12345678-abcd-efgh-ijkl-901234567890",
                    "issuerName": "GitHub",
                    "userName": "alice@example.com",
                    "secret": "JBSW Y3DP EHPK 3PXP",
                    "timeStep": 30,
                    "digits": 6,
                    "algorithm": "SHA1"
                },
                {
                    "accountID": "87654321-dcba-hgfe-lkji-098765432109",
                    "originalIssuerName": "AWS",
                    "originalUserName": "bob@aws.com",
                    "secret": "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
                    "timeStep": 60,
                    "digits": 8,
                    "algorithm": "SHA256"
                }
            ]
        }"#;

        let items = parse_import_payload(lp_json).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "GitHub:alice@example.com");
        assert_eq!(items[1].account, "AWS:bob@aws.com");

        let eng0 = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(eng0.digits(), 6);
        assert_eq!(eng0.step(), 30);

        let eng1 = crate::totp::TfaTotp::from_input(&items[1].account, &items[1].secret).unwrap();
        assert_eq!(eng1.digits(), 8);
        assert_eq!(eng1.step(), 60);
        assert_eq!(eng1.algorithm(), totp_rs::Algorithm::SHA256);
    }

    #[test]
    fn test_import_proton_authenticator_json() {
        let proton_json = r#"{
            "version": 1,
            "entries": [
                {
                    "id": "entry-1",
                    "content": {
                        "name": "Proton",
                        "uri": "otpauth://totp/Proton:alice%40proton.me?secret=JBSWY3DPEHPK3PXP&issuer=Proton&algorithm=SHA1&digits=6&period=30"
                    }
                },
                {
                    "id": "entry-2",
                    "content": {
                        "uri": "otpauth://totp/GitLab:bob%40gitlab.com?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=GitLab&algorithm=SHA256&digits=8&period=30"
                    }
                }
            ]
        }"#;

        let items = parse_import_payload(proton_json).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "Proton:alice@proton.me");
        assert_eq!(items[1].account, "GitLab:bob@gitlab.com");

        let eng0 = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(eng0.digits(), 6);

        let eng1 = crate::totp::TfaTotp::from_input(&items[1].account, &items[1].secret).unwrap();
        assert_eq!(eng1.digits(), 8);
        assert_eq!(eng1.algorithm(), totp_rs::Algorithm::SHA256);
    }

    #[test]
    fn test_import_bitwarden_csv() {
        let bw_csv = "\
folder,favorite,type,name,notes,fields,reprompt,login_uri,login_username,login_password,login_totp
Work,1,login,GitHub,\"Multi-line\nnotes, with comma\",,0,https://github.com,alice@example.com,pass123,JBSWY3DPEHPK3PXP
Personal,,login,No 2FA Account,,,0,https://example.com,bob@example.com,pass456,
Work,,login,AWS Console,,,0,https://aws.amazon.com,admin@aws.com,secretpass,otpauth://totp/AWS:admin@aws.com?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=AWS&digits=8&algorithm=SHA256
";

        let items = parse_import_payload(bw_csv).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "GitHub");
        assert_eq!(items[0].secret, "JBSWY3DPEHPK3PXP");
        assert_eq!(items[1].account, "AWS Console");

        let eng0 = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(eng0.digits(), 6);

        let eng1 = crate::totp::TfaTotp::from_input(&items[1].account, &items[1].secret).unwrap();
        assert_eq!(eng1.digits(), 8);
        assert_eq!(eng1.algorithm(), totp_rs::Algorithm::SHA256);
    }

    #[test]
    fn test_import_1password_csv() {
        let op_csv = "\
Title,Website,Username,Password,One-time password,Favorite status,Archived status,Tags,Notes
\"Cloudflare, Inc.\",https://dash.cloudflare.com,alice@example.com,pw1,otpauth://totp/Cloudflare:alice%40example.com?secret=JBSWY3DPEHPK3PXP&issuer=Cloudflare,true,false,work,\"Quoted \"\"note\"\" here\"
 Plain Login,https://example.org,bob,pw2,,false,false,,
DigitalOcean,https://cloud.digitalocean.com,devops@example.com,pw3,GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ,false,false,infra,
";

        let items = parse_import_payload(op_csv).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "Cloudflare, Inc.");
        assert_eq!(items[1].account, "DigitalOcean");

        for item in &items {
            let eng = crate::totp::TfaTotp::from_input(&item.account, &item.secret).unwrap();
            assert_eq!(eng.generate_current().len(), 6);
        }
    }

    #[test]
    fn test_import_totp_cli_yaml() {
        let yaml = "\
- name: default
  accounts:
    - name: github
      token: JBSWY3DPEHPK3PXP
- name: work
  accounts:
    - name: aws
      token: GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ
      length: 8
      timePeriod: 60
      algorithm: sha256
      prefix: mypin
    - name: gitlab
      token: JBSWY3DPEHPK3PXP
";

        let items = parse_import_payload(yaml).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].account, "github");
        assert_eq!(items[0].secret, "JBSWY3DPEHPK3PXP");
        assert_eq!(items[1].account, "work:aws");
        assert!(items[1].secret.contains("digits=8"));
        assert!(items[1].secret.contains("period=60"));
        assert!(items[1].secret.contains("algorithm=SHA256"));
        assert!(items[1].secret.contains("prefix=mypin"));
        assert_eq!(items[2].account, "work:gitlab");

        let eng0 = crate::totp::TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(eng0.digits(), 6);

        let eng1 = crate::totp::TfaTotp::from_input(&items[1].account, &items[1].secret).unwrap();
        assert_eq!(eng1.digits(), 8);
        assert_eq!(eng1.step(), 60);
        assert_eq!(eng1.algorithm(), totp_rs::Algorithm::SHA256);
    }

    #[test]
    fn test_canonical_account_label_cases() {
        assert_eq!(
            canonical_account_label(Some("GitHub"), Some("alice@example.com")),
            "GitHub:alice@example.com"
        );
        assert_eq!(
            canonical_account_label(Some("GitHub"), Some("GitHub:alice@example.com")),
            "GitHub:alice@example.com"
        );
        assert_eq!(
            canonical_account_label(Some("github"), Some("GitHub:alice@example.com")),
            "GitHub:alice@example.com"
        );
        assert_eq!(
            canonical_account_label(Some("Google"), Some("Google")),
            "Google"
        );
        assert_eq!(
            canonical_account_label(Some("Google"), Some("google")),
            "Google"
        );
        assert_eq!(canonical_account_label(Some("tfa"), Some("alice")), "alice");
        assert_eq!(canonical_account_label(None, Some("alice")), "alice");
        assert_eq!(canonical_account_label(Some("Amazon"), None), "Amazon");
        assert_eq!(canonical_account_label(None, None), "imported_account");
    }

    #[test]
    fn test_are_secrets_equivalent_cases() {
        let sec1 = "JBSWY3DPEHPK3PXP";
        let sec2 = "GEZDGNBVGY3TQOJQ";
        let url_default = "otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP";
        let url_diff_digits = "otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP&digits=8";
        let url_diff_period = "otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP&period=60";
        let url_diff_algo = "otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP&algorithm=SHA256";

        assert!(are_secrets_equivalent(sec1, sec1));
        assert!(are_secrets_equivalent(sec1, url_default));
        assert!(are_secrets_equivalent(url_default, sec1));
        assert!(!are_secrets_equivalent(sec1, sec2));
        assert!(!are_secrets_equivalent(sec1, url_diff_digits));
        assert!(!are_secrets_equivalent(sec1, url_diff_period));
        assert!(!are_secrets_equivalent(sec1, url_diff_algo));
    }

    #[test]
    fn test_import_collision_tracker_batch_and_auto_suffixing() {
        let store = crate::storage::MemoryStore::new();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();

        let mut tracker = ImportCollisionTracker::new();

        // 1. Existing with identical secret -> SkipDuplicate
        let item1 = ImportItem {
            account: "github".to_string(),
            secret: "JBSWY3DPEHPK3PXP".to_string(),
        };
        let act1 = tracker.resolve_item(&store, &item1, false, false);
        assert_eq!(
            act1,
            ResolvedImportAction::SkipDuplicate {
                account: "github".to_string()
            }
        );

        // 2. Existing with differing secret, default mode -> auto-suffix to github-2
        let item2 = ImportItem {
            account: "github".to_string(),
            secret: "GEZDGNBVGY3TQOJQ".to_string(),
        };
        let act2 = tracker.resolve_item(&store, &item2, false, false);
        assert_eq!(
            act2,
            ResolvedImportAction::ImportSuffixed {
                account: "github-2".to_string(),
                original: "github".to_string()
            }
        );

        // 3. Batch repeat of secret for github-2 -> SkipDuplicate
        let act3 = tracker.resolve_item(&store, &item2, false, false);
        assert_eq!(
            act3,
            ResolvedImportAction::SkipDuplicate {
                account: "github".to_string()
            }
        );

        // 4. Third distinct secret -> auto-suffix to github-3
        let item3 = ImportItem {
            account: "github".to_string(),
            secret: "MZXW6YTBOI======".to_string(),
        };
        let act4 = tracker.resolve_item(&store, &item3, false, false);
        assert_eq!(
            act4,
            ResolvedImportAction::ImportSuffixed {
                account: "github-3".to_string(),
                original: "github".to_string()
            }
        );

        // 5. Overwrite flag
        let act_ow = tracker.resolve_item(&store, &item2, true, false);
        assert_eq!(
            act_ow,
            ResolvedImportAction::UpdateExisting {
                account: "github".to_string()
            }
        );

        // 6. Skip flag
        let mut tracker2 = ImportCollisionTracker::new();
        let act_skip = tracker2.resolve_item(&store, &item2, false, true);
        assert_eq!(
            act_skip,
            ResolvedImportAction::SkipExisting {
                account: "github".to_string()
            }
        );
    }
}
