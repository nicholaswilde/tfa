use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::namespace::parse_key;
use crate::storage::AccountStore;
use crate::totp::TfaTotp;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditReport {
    pub accounts_scanned: usize,
    pub findings: Vec<AuditFinding>,
    pub summary: AuditSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditFinding {
    pub rule: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditSummary {
    pub weak_secrets: usize,
    pub duplicate_secrets: usize,
    pub legacy_algorithms: usize,
    pub file_issues: usize,
    pub total_issues: usize,
}

struct ScannedAccount {
    namespace: String,
    account: String,
    full_key: String,
    clean_secret: String,
    algorithm: totp_rs::Algorithm,
}

/// Run security audit on accounts in storage and optional credentials file
pub fn run_security_audit(
    store: &dyn AccountStore,
    credentials_path: Option<&Path>,
) -> Result<AuditReport> {
    let (accounts_scanned, mut findings) = audit_accounts(store)?;

    if let Some(path) = credentials_path {
        if path.exists() {
            let mut file_findings = audit_credentials_file(path);
            findings.append(&mut file_findings);
        }
    }

    findings.sort_by(|a, b| {
        (&a.namespace, &a.account, &a.path, &a.rule, &a.message).cmp(&(
            &b.namespace,
            &b.account,
            &b.path,
            &b.rule,
            &b.message,
        ))
    });

    let mut weak_secrets = 0;
    let mut duplicate_secrets = 0;
    let mut legacy_algorithms = 0;
    let mut file_issues = 0;

    for f in &findings {
        match f.rule.as_str() {
            "weak_secret" => weak_secrets += 1,
            "duplicate_secret" => duplicate_secrets += 1,
            "legacy_algorithm" => legacy_algorithms += 1,
            "file_permissions" | "file_ownership" => file_issues += 1,
            _ => {}
        }
    }

    let summary = AuditSummary {
        weak_secrets,
        duplicate_secrets,
        legacy_algorithms,
        file_issues,
        total_issues: findings.len(),
    };

    Ok(AuditReport {
        accounts_scanned,
        findings,
        summary,
    })
}

/// Audit all accounts across namespaces for security vulnerabilities
pub fn audit_accounts(store: &dyn AccountStore) -> Result<(usize, Vec<AuditFinding>)> {
    let mut all_keys = store.list_accounts()?;
    all_keys.sort();
    let accounts_scanned = all_keys.len();

    let mut scanned = Vec::new();
    let mut findings = Vec::new();

    for key in &all_keys {
        let (ns, acct) = parse_key(key);
        let secret = store.get_secret(key)?;
        match TfaTotp::from_input(&acct, &secret) {
            Ok(engine) => {
                let clean_b32 = engine.secret_base32().trim_end_matches('=').to_uppercase();
                scanned.push(ScannedAccount {
                    namespace: ns,
                    account: acct,
                    full_key: key.clone(),
                    clean_secret: clean_b32,
                    algorithm: engine.algorithm(),
                });
            }
            Err(e) => {
                findings.push(AuditFinding {
                    rule: "corrupted_account".to_string(),
                    severity: "high".to_string(),
                    namespace: Some(ns),
                    account: Some(acct),
                    path: None,
                    message: format!("Failed to parse account secret: {e}"),
                });
            }
        }
    }

    // Check for weak secrets (< 128-bit / 16 Base32 characters)
    for acc in &scanned {
        if acc.clean_secret.len() < 16 {
            findings.push(AuditFinding {
                rule: "weak_secret".to_string(),
                severity: "high".to_string(),
                namespace: Some(acc.namespace.clone()),
                account: Some(acc.account.clone()),
                path: None,
                message: format!(
                    "Weak secret with {} Base32 characters (< 128-bit / 16 Base32 characters)",
                    acc.clean_secret.len()
                ),
            });
        }
    }

    // Check for legacy SHA-1 hash algorithms
    for acc in &scanned {
        if acc.algorithm == totp_rs::Algorithm::SHA1 {
            findings.push(AuditFinding {
                rule: "legacy_algorithm".to_string(),
                severity: "warning".to_string(),
                namespace: Some(acc.namespace.clone()),
                account: Some(acc.account.clone()),
                path: None,
                message: "Legacy SHA-1 algorithm in use (SHA-256 or SHA-512 recommended)"
                    .to_string(),
            });
        }
    }

    // Check for duplicate secret seeds shared across distinct accounts
    let mut seed_map: HashMap<&str, Vec<&ScannedAccount>> = HashMap::new();
    for acc in &scanned {
        seed_map.entry(&acc.clean_secret).or_default().push(acc);
    }

    for acc in &scanned {
        if let Some(group) = seed_map.get(acc.clean_secret.as_str()) {
            if group.len() > 1 {
                let mut others: Vec<String> = group
                    .iter()
                    .filter(|o| o.full_key != acc.full_key)
                    .map(|o| format!("{}:{}", o.namespace, o.account))
                    .collect();
                others.sort();
                findings.push(AuditFinding {
                    rule: "duplicate_secret".to_string(),
                    severity: "high".to_string(),
                    namespace: Some(acc.namespace.clone()),
                    account: Some(acc.account.clone()),
                    path: None,
                    message: format!(
                        "Duplicate secret seed shared with {}",
                        others
                            .iter()
                            .map(|o| format!("'{o}'"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                });
            }
        }
    }

    Ok((accounts_scanned, findings))
}

/// Audit permissions (0600) and ownership of a credentials file
pub fn audit_credentials_file(path: &Path) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    if !path.exists() {
        return findings;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.mode() & 0o777;
            // 0600 is user rw only; flag world or group readable permissions (0o044) or non-0600
            if (mode & 0o044) != 0 || mode != 0o600 {
                findings.push(AuditFinding {
                    rule: "file_permissions".to_string(),
                    severity: "high".to_string(),
                    namespace: None,
                    account: None,
                    path: Some(path.to_string_lossy().to_string()),
                    message: format!(
                        "Insecure file permissions '{mode:04o}' (expected '0600', world/group-readable)"
                    ),
                });
            }

            let file_uid = meta.uid();
            let current_uid = unsafe {
                extern "C" {
                    fn getuid() -> u32;
                }
                getuid()
            };
            if file_uid != current_uid {
                findings.push(AuditFinding {
                    rule: "file_ownership".to_string(),
                    severity: "high".to_string(),
                    namespace: None,
                    account: None,
                    path: Some(path.to_string_lossy().to_string()),
                    message: format!(
                        "Credentials file owner UID {file_uid} does not match current user UID {current_uid}"
                    ),
                });
            }
        }
    }

    findings
}

/// Render human-readable security audit report
pub fn format_audit_report(report: &AuditReport) -> String {
    let mut out = String::new();
    out.push_str("=== Security Audit Report ===\n\n");
    out.push_str(&format!("Accounts Scanned: {}\n", report.accounts_scanned));
    out.push_str(&format!("Issues Found: {}\n", report.summary.total_issues));

    if report.findings.is_empty() {
        out.push_str("\nNo security vulnerabilities detected.\n");
    } else {
        out.push_str("\nFindings:\n");
        for f in &report.findings {
            let target = if let (Some(ns), Some(acc)) = (&f.namespace, &f.account) {
                format!("{ns}:{acc}")
            } else if let Some(path) = &f.path {
                path.clone()
            } else {
                "system".to_string()
            };
            let sev_tag = match f.severity.to_lowercase().as_str() {
                "high" => "[HIGH]",
                "warning" | "warn" => "[WARN]",
                _ => "[INFO]",
            };
            out.push_str(&format!("  {sev_tag} {target} - {}\n", f.message));
        }

        out.push_str("\nSummary:\n");
        out.push_str(&format!(
            "  - Weak secrets: {}\n",
            report.summary.weak_secrets
        ));
        out.push_str(&format!(
            "  - Duplicate secrets: {}\n",
            report.summary.duplicate_secrets
        ));
        out.push_str(&format!(
            "  - Legacy algorithms: {}\n",
            report.summary.legacy_algorithms
        ));
        out.push_str(&format!(
            "  - File permission/ownership issues: {}\n",
            report.summary.file_issues
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStore;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_audit_empty_store() {
        let store = MemoryStore::new();
        let report = run_security_audit(&store, None).unwrap();
        assert_eq!(report.accounts_scanned, 0);
        assert_eq!(report.summary.total_issues, 0);
        assert!(report.findings.is_empty());

        let text = format_audit_report(&report);
        assert!(text.contains("No security vulnerabilities detected."));
    }

    #[test]
    fn test_audit_strong_account_no_issues() {
        let store = MemoryStore::new();
        // SHA-256 algorithm with 32 Base32 characters (160 bits)
        let engine = TfaTotp::build(
            "strong",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            Some(6),
            Some(30),
            Some(totp_rs::Algorithm::SHA256),
            None,
        )
        .unwrap();
        store
            .set_secret("strong", &engine.to_url().unwrap())
            .unwrap();

        let report = run_security_audit(&store, None).unwrap();
        assert_eq!(report.accounts_scanned, 1);
        assert_eq!(report.summary.total_issues, 0);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_audit_weak_secret_and_legacy_algorithm() {
        let store = MemoryStore::new();
        // 8-character Base32 secret (< 16 Base32 characters) with default SHA-1
        let engine = TfaTotp::build(
            "weak_acc",
            "GEZDGNBV",
            Some(6),
            Some(30),
            Some(totp_rs::Algorithm::SHA1),
            None,
        )
        .unwrap();
        store
            .set_secret("weak_acc", &engine.to_url().unwrap())
            .unwrap();

        let report = run_security_audit(&store, None).unwrap();
        assert_eq!(report.accounts_scanned, 1);
        assert_eq!(report.summary.total_issues, 2);
        assert_eq!(report.summary.weak_secrets, 1);
        assert_eq!(report.summary.legacy_algorithms, 1);

        let weak_finding = report
            .findings
            .iter()
            .find(|f| f.rule == "weak_secret")
            .unwrap();
        assert_eq!(weak_finding.severity, "high");
        assert!(weak_finding
            .message
            .contains("Weak secret with 8 Base32 characters"));

        let alg_finding = report
            .findings
            .iter()
            .find(|f| f.rule == "legacy_algorithm")
            .unwrap();
        assert_eq!(alg_finding.severity, "warning");
        assert!(alg_finding.message.contains("Legacy SHA-1 algorithm"));

        let text = format_audit_report(&report);
        assert!(text.contains("[HIGH] default:weak_acc - Weak secret"));
        assert!(text.contains("[WARN] default:weak_acc - Legacy SHA-1"));
    }

    #[test]
    fn test_audit_duplicate_secret_seeds_across_namespaces() {
        let store = MemoryStore::new();
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

        // Account 1 in default namespace (SHA-256)
        let eng1 = TfaTotp::build(
            "acc1",
            secret,
            Some(6),
            Some(30),
            Some(totp_rs::Algorithm::SHA256),
            None,
        )
        .unwrap();
        store.set_secret("acc1", &eng1.to_url().unwrap()).unwrap();

        // Account 2 in work namespace with same seed (SHA-256)
        let eng2 = TfaTotp::build(
            "acc2",
            secret,
            Some(6),
            Some(30),
            Some(totp_rs::Algorithm::SHA256),
            None,
        )
        .unwrap();
        store
            .set_secret("work:acc2", &eng2.to_url().unwrap())
            .unwrap();

        let report = run_security_audit(&store, None).unwrap();
        assert_eq!(report.accounts_scanned, 2);
        assert_eq!(report.summary.total_issues, 2);
        assert_eq!(report.summary.duplicate_secrets, 2);

        let dup1 = report
            .findings
            .iter()
            .find(|f| f.account.as_deref() == Some("acc1"))
            .unwrap();
        assert_eq!(dup1.rule, "duplicate_secret");
        assert!(dup1.message.contains("'work:acc2'"));

        let dup2 = report
            .findings
            .iter()
            .find(|f| f.account.as_deref() == Some("acc2"))
            .unwrap();
        assert_eq!(dup2.rule, "duplicate_secret");
        assert!(dup2.message.contains("'default:acc1'"));
    }

    #[test]
    #[cfg(unix)]
    fn test_audit_credentials_file_permissions() {
        let tmp = tempdir().unwrap();
        let creds_file = tmp.path().join("credentials.json");
        std::fs::write(&creds_file, b"{}").unwrap();

        // Set permissions to 0644 (world readable)
        std::fs::set_permissions(&creds_file, std::fs::Permissions::from_mode(0o644)).unwrap();

        let findings = audit_credentials_file(&creds_file);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "file_permissions");
        assert_eq!(findings[0].severity, "high");
        assert!(findings[0].message.contains("0644"));

        // Set permissions to 0600 (secure)
        std::fs::set_permissions(&creds_file, std::fs::Permissions::from_mode(0o600)).unwrap();
        let findings_ok = audit_credentials_file(&creds_file);
        assert!(findings_ok.is_empty());
    }

    #[test]
    fn test_audit_nonexistent_credentials_file_ignored() {
        let tmp = tempdir().unwrap();
        let creds_file = tmp.path().join("does_not_exist.json");
        let findings = audit_credentials_file(&creds_file);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_audit_corrupted_account() {
        let store = MemoryStore::new();
        store
            .set_secret("broken", "invalid_base32_secret!#@$")
            .unwrap();
        let report = run_security_audit(&store, None).unwrap();
        assert_eq!(report.accounts_scanned, 1);
        assert_eq!(report.summary.total_issues, 1);
        let finding = &report.findings[0];
        assert_eq!(finding.rule, "corrupted_account");
        assert_eq!(finding.severity, "high");
    }
}
