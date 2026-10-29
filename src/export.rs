use anyhow::Result;
use clap::ValueEnum;
use qrcodegen::{QrCode, QrCodeEcc};
use std::collections::BTreeMap;

use crate::totp::TfaTotp;

#[derive(
    ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Json,
    Otpauth,
    Yaml,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Otpauth => write!(f, "otpauth"),
            Self::Yaml => write!(f, "yaml"),
        }
    }
}

pub fn format_export_payload(
    map: &BTreeMap<String, String>,
    format: ExportFormat,
) -> Result<String> {
    match format {
        ExportFormat::Json => serde_json::to_string_pretty(map)
            .map_err(|e| anyhow::anyhow!("Failed formatting JSON export: {e}")),
        ExportFormat::Yaml => serde_yaml_ng::to_string(map)
            .map_err(|e| anyhow::anyhow!("Failed formatting YAML export: {e}")),
        ExportFormat::Otpauth => {
            let mut lines = Vec::new();
            for (acct, secret) in map {
                let (ns, clean_name) = match acct.split_once(':') {
                    Some((ns, name)) => (Some(ns), name),
                    None => (None, acct.as_str()),
                };
                let url = match TfaTotp::from_input(clean_name, secret) {
                    Ok(engine) => {
                        let base_url = engine.to_url().unwrap_or_else(|_| secret.clone());
                        if let Some(ns_str) = ns {
                            if let Ok(mut u) = url::Url::parse(&base_url) {
                                let label = format!("{ns_str}:{clean_name}");
                                u.set_path(&label);
                                u.to_string()
                            } else {
                                base_url
                            }
                        } else {
                            base_url
                        }
                    }
                    Err(_) => {
                        if secret.starts_with("otpauth://") {
                            secret.clone()
                        } else {
                            format!("otpauth://totp/{acct}?secret={secret}")
                        }
                    }
                };
                lines.push(url);
            }
            Ok(lines.join("\n"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationAccount {
    pub name: String,
    pub secret: Vec<u8>,
    pub issuer: String,
    pub algorithm: totp_rs::Algorithm,
    pub digits: u8,
    pub is_hotp: bool,
    pub counter: u64,
}

impl MigrationAccount {
    pub fn from_account_and_secret(account: &str, secret: &str) -> Result<Self> {
        let (ns, clean_name) = match account.split_once(':') {
            Some((ns, name)) => (Some(ns), name),
            None => (None, account),
        };
        let engine = TfaTotp::from_input(clean_name, secret)?;
        let secret_bytes = engine.secret_bytes();
        let (issuer, name) = match ns {
            Some(ns_str) => (ns_str.to_string(), format!("{ns_str}:{clean_name}")),
            None => ("tfa".to_string(), clean_name.to_string()),
        };
        Ok(Self {
            name,
            secret: secret_bytes,
            issuer,
            algorithm: engine.algorithm(),
            digits: engine.digits(),
            is_hotp: engine.is_hotp(),
            counter: engine.counter().unwrap_or(0),
        })
    }
}

fn encode_varint(mut val: u64, buf: &mut Vec<u8>) {
    while val >= 0x80 {
        buf.push((val as u8 & 0x7f) | 0x80);
        val >>= 7;
    }
    buf.push(val as u8);
}

fn encode_tag(field_number: u32, wire_type: u8, buf: &mut Vec<u8>) {
    encode_varint(((field_number as u64) << 3) | (wire_type as u64), buf);
}

fn encode_length_delimited(field_number: u32, data: &[u8], buf: &mut Vec<u8>) {
    encode_tag(field_number, 2, buf);
    encode_varint(data.len() as u64, buf);
    buf.extend_from_slice(data);
}

fn encode_varint_field(field_number: u32, val: u64, buf: &mut Vec<u8>) {
    encode_tag(field_number, 0, buf);
    encode_varint(val, buf);
}

fn encode_otp_parameters(acc: &MigrationAccount) -> Vec<u8> {
    let mut buf = Vec::new();

    // 1. secret (bytes)
    encode_length_delimited(1, &acc.secret, &mut buf);

    // 2. name (string)
    encode_length_delimited(2, acc.name.as_bytes(), &mut buf);

    // 3. issuer (string)
    if !acc.issuer.is_empty() {
        encode_length_delimited(3, acc.issuer.as_bytes(), &mut buf);
    }

    // 4. algorithm (Algorithm enum: ALGO_SHA1 = 1)
    let algo_val = match acc.algorithm {
        totp_rs::Algorithm::SHA1 => 1,
        totp_rs::Algorithm::SHA256 => 2,
        totp_rs::Algorithm::SHA512 => 3,
        _ => 1,
    };
    encode_varint_field(4, algo_val, &mut buf);

    // 5. digits (int32: 1 = 6 digits, 2 = 8 digits)
    let digits_val = match acc.digits {
        8 => 2,
        _ => 1,
    };
    encode_varint_field(5, digits_val, &mut buf);

    // 6. type (OtpType enum: OTP_HOTP = 1, OTP_TOTP = 2)
    let type_val = if acc.is_hotp { 1 } else { 2 };
    encode_varint_field(6, type_val, &mut buf);

    // 7. counter (int64)
    if acc.is_hotp && acc.counter > 0 {
        encode_varint_field(7, acc.counter, &mut buf);
    }

    buf
}

fn encode_migration_payload(
    accounts: &[MigrationAccount],
    batch_size: i32,
    batch_index: i32,
    batch_id: i32,
) -> Vec<u8> {
    let mut buf = Vec::new();

    for acc in accounts {
        let param_bytes = encode_otp_parameters(acc);
        encode_length_delimited(1, &param_bytes, &mut buf);
    }

    // version = 1
    encode_varint_field(2, 1, &mut buf);

    // batch_size
    encode_varint_field(3, batch_size as u64, &mut buf);

    // batch_index
    encode_varint_field(4, batch_index as u64, &mut buf);

    // batch_id
    encode_varint_field(5, batch_id as u64, &mut buf);

    buf
}

pub fn generate_migration_uri(
    accounts: &[MigrationAccount],
    batch_size: i32,
    batch_index: i32,
    batch_id: i32,
) -> String {
    let payload = encode_migration_payload(accounts, batch_size, batch_index, batch_id);
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&payload);
    let mut u = url::Url::parse("otpauth-migration://offline").unwrap();
    u.query_pairs_mut().append_pair("data", &b64);
    u.to_string()
}

/// Default maximum accounts per batch to ensure comfortable terminal display and scanning
pub const DEFAULT_MAX_ACCOUNTS_PER_BATCH: usize = 10;

/// Chunk accounts and generate `otpauth-migration://` URIs, ensuring each fits within QR capacity
pub fn generate_migration_uris(
    accounts: &[MigrationAccount],
    max_per_batch: usize,
) -> Result<Vec<String>> {
    if accounts.is_empty() {
        return Ok(Vec::new());
    }

    let batch_id = (rand::random::<u32>() & 0x7fffffff) as i32;
    let initial_chunk_size = if max_per_batch == 0 {
        DEFAULT_MAX_ACCOUNTS_PER_BATCH
    } else {
        max_per_batch
    };

    // Partition accounts into batches, dynamically splitting any batch if QR capacity is exceeded
    let mut batches: Vec<Vec<MigrationAccount>> = Vec::new();
    let mut current_batch = Vec::new();

    for acc in accounts {
        current_batch.push(acc.clone());

        // Check if current batch exceeds chunk size or QR capacity
        let test_uri = generate_migration_uri(&current_batch, 1, 0, batch_id);
        let fits_qr = QrCode::encode_text(&test_uri, QrCodeEcc::Medium).is_ok();

        if current_batch.len() >= initial_chunk_size || !fits_qr {
            if !fits_qr && current_batch.len() > 1 {
                // Remove the last added account and flush previous batch
                let last = current_batch.pop().unwrap();
                batches.push(current_batch);
                current_batch = vec![last];
            } else {
                batches.push(current_batch);
                current_batch = Vec::new();
            }
        }
    }
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    let total_batches = batches.len() as i32;
    let mut uris = Vec::new();

    for (idx, batch) in batches.iter().enumerate() {
        let uri = generate_migration_uri(batch, total_batches, idx as i32, batch_id);
        // Ensure generated URI encodes into a valid QR code
        QrCode::encode_text(&uri, QrCodeEcc::Medium)
            .map_err(|e| anyhow::anyhow!("Migration payload exceeded QR code capacity: {e:?}"))?;
        uris.push(uri);
    }

    Ok(uris)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::parse_import_payload;

    #[test]
    fn test_export_format_display() {
        assert_eq!(ExportFormat::Json.to_string(), "json");
        assert_eq!(ExportFormat::Otpauth.to_string(), "otpauth");
        assert_eq!(ExportFormat::Yaml.to_string(), "yaml");
    }

    #[test]
    fn test_format_export_payload_json_roundtrip() {
        let mut map = BTreeMap::new();
        map.insert("github".to_string(), "JBSWY3DPEHPK3PXP".to_string());
        map.insert(
            "aws".to_string(),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_string(),
        );

        let json_str = format_export_payload(&map, ExportFormat::Json).unwrap();
        let items = parse_import_payload(&json_str).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "aws");
        assert_eq!(items[0].secret, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        assert_eq!(items[1].account, "github");
        assert_eq!(items[1].secret, "JBSWY3DPEHPK3PXP");
    }

    #[test]
    fn test_format_export_payload_yaml_roundtrip() {
        let mut map = BTreeMap::new();
        map.insert("github".to_string(), "JBSWY3DPEHPK3PXP".to_string());
        map.insert(
            "work:aws".to_string(),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_string(),
        );

        let yaml_str = format_export_payload(&map, ExportFormat::Yaml).unwrap();
        let items = parse_import_payload(&yaml_str).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "github");
        assert_eq!(items[0].secret, "JBSWY3DPEHPK3PXP");
        assert_eq!(items[1].account, "work:aws");
        assert_eq!(items[1].secret, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
    }

    #[test]
    fn test_format_export_payload_otpauth_roundtrip() {
        let mut map = BTreeMap::new();
        map.insert("github".to_string(), "JBSWY3DPEHPK3PXP".to_string());
        map.insert(
            "work:gitlab".to_string(),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_string(),
        );

        let otpauth_str = format_export_payload(&map, ExportFormat::Otpauth).unwrap();
        assert!(otpauth_str.contains("otpauth://totp/"));

        let items = parse_import_payload(&otpauth_str).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].account, "github");
        assert_eq!(items[1].account, "gitlab");
    }

    #[test]
    fn test_migration_uri_single_account_roundtrip() {
        let acc = MigrationAccount::from_account_and_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        let uri = generate_migration_uri(&[acc], 1, 0, 12345);

        assert!(uri.starts_with("otpauth-migration://offline?data="));

        // Roundtrip through google_authenticator_converter / parse_import_payload
        let items = parse_import_payload(&uri).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].account, "github");
        assert_eq!(items[0].secret, "JBSWY3DPEHPK3PXP");

        let engine = TfaTotp::from_input(&items[0].account, &items[0].secret).unwrap();
        assert_eq!(engine.generate_current().len(), 6);
    }

    #[test]
    fn test_migration_uris_chunking_and_roundtrip() {
        let mut accounts = Vec::new();
        for i in 0..25 {
            let name = format!("account_{i:02}");
            let acc = MigrationAccount::from_account_and_secret(&name, "JBSWY3DPEHPK3PXP").unwrap();
            accounts.push(acc);
        }

        // Chunk into batches of 10
        let uris = generate_migration_uris(&accounts, 10).unwrap();
        assert_eq!(uris.len(), 3); // 10, 10, 5

        let mut total_imported = Vec::new();
        for uri in uris {
            let mut items = parse_import_payload(&uri).unwrap();
            total_imported.append(&mut items);
        }

        assert_eq!(total_imported.len(), 25);
        for (i, item) in total_imported.iter().enumerate().take(25) {
            assert_eq!(item.account, format!("account_{i:02}"));
            assert_eq!(item.secret, "JBSWY3DPEHPK3PXP");
        }
    }
}
