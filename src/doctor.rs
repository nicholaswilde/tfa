use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub category: String,
    pub severity: CheckSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub total_checks: usize,
    pub ok: usize,
    pub warnings: usize,
    pub errors: usize,
    pub healthy: bool,
}

impl DoctorReport {
    pub fn new(checks: Vec<DoctorCheck>) -> Self {
        let ok = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Ok)
            .count();
        let warnings = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Warning)
            .count();
        let errors = checks
            .iter()
            .filter(|c| c.severity == CheckSeverity::Error)
            .count();
        let total_checks = checks.len();
        let healthy = errors == 0;
        Self {
            checks,
            total_checks,
            ok,
            warnings,
            errors,
            healthy,
        }
    }
}

/// Run all doctor diagnostic checks
pub fn run_doctor(
    active_backend: Option<&str>,
    credentials_path: Option<&Path>,
    age_path: Option<&Path>,
    config_paths: &[PathBuf],
    ntp_server: Option<&str>,
) -> DoctorReport {
    let mut checks = Vec::new();

    checks.extend(check_keyring(active_backend));
    checks.extend(check_storage_and_permissions(
        active_backend,
        credentials_path,
        age_path,
        config_paths,
    ));
    checks.extend(check_clock_drift(ntp_server));
    checks.extend(check_clipboard());
    checks.extend(check_age_prerequisites(active_backend));
    checks.extend(check_terminal_capabilities());

    DoctorReport::new(checks)
}

/// Check keyring service responsiveness
pub fn check_keyring(active_backend: Option<&str>) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();
    let is_keyring_active = active_backend == Some("keyring");

    match keyring::Entry::new("tfa", "__tfa_doctor_probe__") {
        Ok(entry) => match entry.get_password() {
            Ok(_) | Err(keyring::Error::NoEntry) => {
                checks.push(DoctorCheck {
                    category: "keyring".into(),
                    severity: CheckSeverity::Ok,
                    message: "Keyring service is responsive and reachable".into(),
                    path: None,
                });
            }
            Err(e) => {
                checks.push(DoctorCheck {
                    category: "keyring".into(),
                    severity: if is_keyring_active {
                        CheckSeverity::Error
                    } else {
                        CheckSeverity::Warning
                    },
                    message: format!(
                        "Keyring service returned error: {e}{}",
                        if is_keyring_active {
                            ""
                        } else {
                            " (fallback to file storage is active)"
                        }
                    ),
                    path: None,
                });
            }
        },
        Err(e) => {
            checks.push(DoctorCheck {
                category: "keyring".into(),
                severity: if is_keyring_active {
                    CheckSeverity::Error
                } else {
                    CheckSeverity::Warning
                },
                message: format!(
                    "Failed initializing keyring probe: {e}{}",
                    if is_keyring_active {
                        ""
                    } else {
                        " (fallback to file storage is active)"
                    }
                ),
                path: None,
            });
        }
    }

    checks
}

/// Check storage file, directory, and config file permissions
pub fn check_storage_and_permissions(
    active_backend: Option<&str>,
    credentials_path: Option<&Path>,
    age_path: Option<&Path>,
    config_paths: &[PathBuf],
) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    // 1. Credentials file permissions & ownership
    if let Some(cred_path) = credentials_path {
        if cred_path.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if let Ok(meta) = std::fs::metadata(cred_path) {
                    let mode = meta.mode() & 0o777;
                    if mode != 0o600 {
                        checks.push(DoctorCheck {
                            category: "storage_permissions".into(),
                            severity: CheckSeverity::Error,
                            message: format!(
                                "Credentials file '{}' has insecure permissions {:04o} (expected 0600)",
                                cred_path.display(),
                                mode
                            ),
                            path: Some(cred_path.to_path_buf()),
                        });
                    } else {
                        checks.push(DoctorCheck {
                            category: "storage_permissions".into(),
                            severity: CheckSeverity::Ok,
                            message: format!(
                                "Credentials file '{}' permissions are secure (0600)",
                                cred_path.display()
                            ),
                            path: Some(cred_path.to_path_buf()),
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
                        checks.push(DoctorCheck {
                            category: "storage_permissions".into(),
                            severity: CheckSeverity::Error,
                            message: format!(
                                "Credentials file '{}' owner UID {file_uid} does not match current user UID {current_uid}",
                                cred_path.display()
                            ),
                            path: Some(cred_path.to_path_buf()),
                        });
                    }
                }
            }
            #[cfg(not(unix))]
            {
                checks.push(DoctorCheck {
                    category: "storage_permissions".into(),
                    severity: CheckSeverity::Ok,
                    message: format!("Credentials file '{}' exists", cred_path.display()),
                    path: Some(cred_path.to_path_buf()),
                });
            }
        } else if active_backend == Some("file") {
            checks.push(DoctorCheck {
                category: "storage_permissions".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "Credentials file '{}' does not exist yet (will be created on first write)",
                    cred_path.display()
                ),
                path: Some(cred_path.to_path_buf()),
            });
        }

        // Parent directory permissions
        if let Some(parent) = cred_path.parent() {
            if parent.exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if let Ok(meta) = std::fs::metadata(parent) {
                        let mode = meta.mode() & 0o777;
                        if (mode & 0o077) != 0 {
                            checks.push(DoctorCheck {
                                category: "storage_permissions".into(),
                                severity: CheckSeverity::Warning,
                                message: format!(
                                    "Storage directory '{}' has permissions {:04o} (expected 0700, accessible to group/world)",
                                    parent.display(),
                                    mode
                                ),
                                path: Some(parent.to_path_buf()),
                            });
                        } else {
                            checks.push(DoctorCheck {
                                category: "storage_permissions".into(),
                                severity: CheckSeverity::Ok,
                                message: format!(
                                    "Storage directory '{}' permissions are secure ({:04o})",
                                    parent.display(),
                                    mode
                                ),
                                path: Some(parent.to_path_buf()),
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Age storage permissions if present
    if let Some(a_path) = age_path {
        if a_path.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if let Ok(meta) = std::fs::metadata(a_path) {
                    let mode = meta.mode() & 0o777;
                    if mode != 0o600 {
                        checks.push(DoctorCheck {
                            category: "storage_permissions".into(),
                            severity: CheckSeverity::Error,
                            message: format!(
                                "Age credentials file '{}' has insecure permissions {:04o} (expected 0600)",
                                a_path.display(),
                                mode
                            ),
                            path: Some(a_path.to_path_buf()),
                        });
                    } else {
                        checks.push(DoctorCheck {
                            category: "storage_permissions".into(),
                            severity: CheckSeverity::Ok,
                            message: format!(
                                "Age credentials file '{}' permissions are secure (0600)",
                                a_path.display()
                            ),
                            path: Some(a_path.to_path_buf()),
                        });
                    }
                }
            }
            #[cfg(not(unix))]
            {
                checks.push(DoctorCheck {
                    category: "storage_permissions".into(),
                    severity: CheckSeverity::Ok,
                    message: format!("Age credentials file '{}' exists", a_path.display()),
                    path: Some(a_path.to_path_buf()),
                });
            }
        } else if active_backend == Some("age") {
            checks.push(DoctorCheck {
                category: "storage_permissions".into(),
                severity: CheckSeverity::Ok,
                message: format!(
                    "Age credentials file '{}' does not exist yet (will be created on first write)",
                    a_path.display()
                ),
                path: Some(a_path.to_path_buf()),
            });
        }
    }

    // 3. Config file & directory permissions
    let mut config_found = false;
    for cfg_path in config_paths {
        if cfg_path.exists() {
            config_found = true;
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if let Ok(meta) = std::fs::metadata(cfg_path) {
                    let mode = meta.mode() & 0o777;
                    if (mode & 0o044) != 0 {
                        checks.push(DoctorCheck {
                            category: "storage_permissions".into(),
                            severity: CheckSeverity::Warning,
                            message: format!(
                                "Config file '{}' has permissions {:04o} (expected 0600, world/group-readable)",
                                cfg_path.display(),
                                mode
                            ),
                            path: Some(cfg_path.clone()),
                        });
                    } else {
                        checks.push(DoctorCheck {
                            category: "storage_permissions".into(),
                            severity: CheckSeverity::Ok,
                            message: format!(
                                "Config file '{}' permissions are secure ({:04o})",
                                cfg_path.display(),
                                mode
                            ),
                            path: Some(cfg_path.clone()),
                        });
                    }
                }
            }
            #[cfg(not(unix))]
            {
                checks.push(DoctorCheck {
                    category: "storage_permissions".into(),
                    severity: CheckSeverity::Ok,
                    message: format!("Config file '{}' found", cfg_path.display()),
                    path: Some(cfg_path.clone()),
                });
            }

            if let Some(parent) = cfg_path.parent() {
                if parent.exists() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        if let Ok(meta) = std::fs::metadata(parent) {
                            let mode = meta.mode() & 0o777;
                            if (mode & 0o077) != 0 {
                                checks.push(DoctorCheck {
                                    category: "storage_permissions".into(),
                                    severity: CheckSeverity::Warning,
                                    message: format!(
                                        "Config directory '{}' has permissions {:04o} (expected 0700, accessible to group/world)",
                                        parent.display(),
                                        mode
                                    ),
                                    path: Some(parent.to_path_buf()),
                                });
                            } else {
                                checks.push(DoctorCheck {
                                    category: "storage_permissions".into(),
                                    severity: CheckSeverity::Ok,
                                    message: format!(
                                        "Config directory '{}' permissions are secure ({:04o})",
                                        parent.display(),
                                        mode
                                    ),
                                    path: Some(parent.to_path_buf()),
                                });
                            }
                        }
                    }
                }
            }
            break;
        }
    }

    if !config_found && !config_paths.is_empty() {
        checks.push(DoctorCheck {
            category: "storage_permissions".into(),
            severity: CheckSeverity::Ok,
            message: "No custom configuration file found (using defaults)".into(),
            path: None,
        });
    }

    checks
}

/// Check clock drift against NTP pool
pub fn check_clock_drift(ntp_server: Option<&str>) -> Vec<DoctorCheck> {
    if std::env::var("TFA_DOCTOR_SKIP_NTP").is_ok() {
        return vec![DoctorCheck {
            category: "clock_drift".into(),
            severity: CheckSeverity::Ok,
            message: "NTP clock drift check skipped by configuration".into(),
            path: None,
        }];
    }

    let host = ntp_server.unwrap_or("pool.ntp.org");
    let server = format!("{host}:123");

    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            return vec![DoctorCheck {
                category: "clock_drift".into(),
                severity: CheckSeverity::Warning,
                message: format!("Unable to bind UDP socket for NTP check: {e}"),
                path: None,
            }];
        }
    };

    let _ = socket.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = socket.set_write_timeout(Some(Duration::from_secs(2)));

    let mut packet = [0u8; 48];
    packet[0] = 0x1B; // LI = 0, VN = 3, Mode = 3 (client)

    if let Err(e) = socket.send_to(&packet, &server) {
        return vec![DoctorCheck {
            category: "clock_drift".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "NTP server '{server}' unreachable ({e}); network may be offline or UDP 123 blocked"
            ),
            path: None,
        }];
    }

    if let Err(e) = socket.recv_from(&mut packet) {
        return vec![DoctorCheck {
            category: "clock_drift".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "NTP response timed out from '{server}' ({e}); network may be offline or UDP 123 blocked"
            ),
            path: None,
        }];
    }

    let secs_1900 = u32::from_be_bytes(packet[40..44].try_into().unwrap_or([0, 0, 0, 0])) as u64;
    let frac = u32::from_be_bytes(packet[44..48].try_into().unwrap_or([0, 0, 0, 0])) as f64
        / 4_294_967_296.0;
    let ntp_time = (secs_1900 as f64 - 2_208_988_800.0) + frac;
    let system_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let drift = (system_time - ntp_time).abs();

    if drift <= 5.0 {
        vec![DoctorCheck {
            category: "clock_drift".into(),
            severity: CheckSeverity::Ok,
            message: format!(
                "System clock synchronized with NTP pool ({host}, drift: {drift:.2}s)"
            ),
            path: None,
        }]
    } else if drift <= 15.0 {
        vec![DoctorCheck {
            category: "clock_drift".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "System clock drift is {drift:.2}s (drift > 5s; TOTP codes may experience boundary skew)"
            ),
            path: None,
        }]
    } else {
        vec![DoctorCheck {
            category: "clock_drift".into(),
            severity: CheckSeverity::Error,
            message: format!(
                "System clock drift is {drift:.2}s (>15s)! TOTP verification will fail. Please synchronize your system clock."
            ),
            path: None,
        }]
    }
}

/// Check system clipboard accessibility
pub fn check_clipboard() -> Vec<DoctorCheck> {
    match arboard::Clipboard::new() {
        Ok(_) => vec![DoctorCheck {
            category: "clipboard".into(),
            severity: CheckSeverity::Ok,
            message: "System clipboard is available and functional via arboard".into(),
            path: None,
        }],
        Err(e) => {
            let has_term = std::env::var("TMUX").is_ok()
                || std::env::var("SSH_CONNECTION").is_ok()
                || std::env::var("TERM").is_ok();
            vec![DoctorCheck {
                category: "clipboard".into(),
                severity: CheckSeverity::Warning,
                message: if has_term {
                    format!(
                        "System clipboard unavailable ({e}); terminal OSC 52 fallback is available"
                    )
                } else {
                    format!(
                        "System clipboard unavailable ({e}); use '--quiet' / '-q' to pipe tokens"
                    )
                },
                path: None,
            }]
        }
    }
}

/// Check age encryption prerequisites and identity keys
pub fn check_age_prerequisites(active_backend: Option<&str>) -> Vec<DoctorCheck> {
    let mut candidates = crate::storage::AgeStore::candidate_key_paths();
    let is_age_active = active_backend == Some("age");

    let existing_files: Vec<PathBuf> = candidates.drain(..).filter(|p| p.exists()).collect();
    let has_env_key = std::env::var("TFA_AGE_KEY").is_ok();
    let has_env_pass = std::env::var("TFA_AGE_PASSPHRASE").is_ok();

    let count =
        existing_files.len() + if has_env_key { 1 } else { 0 } + if has_env_pass { 1 } else { 0 };

    if count > 0 {
        vec![DoctorCheck {
            category: "age_prerequisites".into(),
            severity: CheckSeverity::Ok,
            message: format!(
                "Age identity configured ({count} key source(s) available for age encryption/decryption)"
            ),
            path: existing_files.into_iter().next(),
        }]
    } else if is_age_active {
        vec![DoctorCheck {
            category: "age_prerequisites".into(),
            severity: CheckSeverity::Error,
            message: "No age identity keys found. Age storage requires an identity (set TFA_AGE_KEY_FILE, SOPS_AGE_KEY_FILE, or ~/.config/sops/age/keys.txt)".into(),
            path: None,
        }]
    } else {
        vec![DoctorCheck {
            category: "age_prerequisites".into(),
            severity: CheckSeverity::Ok,
            message: "No age identity keys configured (optional, only required when using age storage or encrypted backups)".into(),
            path: None,
        }]
    }
}

/// Check terminal capabilities (UTF-8 unicode and ANSI color)
pub fn check_terminal_capabilities() -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    // 1. UTF-8 check
    let lang = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_CTYPE"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();

    if lang.to_lowercase().contains("utf-8") || lang.to_lowercase().contains("utf8") {
        checks.push(DoctorCheck {
            category: "terminal_capabilities".into(),
            severity: CheckSeverity::Ok,
            message:
                "Terminal supports UTF-8 Unicode characters (required for ANSI block QR codes)"
                    .into(),
            path: None,
        });
    } else if lang.is_empty() {
        checks.push(DoctorCheck {
            category: "terminal_capabilities".into(),
            severity: CheckSeverity::Ok,
            message: "Terminal locale default assumed UTF-8".into(),
            path: None,
        });
    } else {
        checks.push(DoctorCheck {
            category: "terminal_capabilities".into(),
            severity: CheckSeverity::Warning,
            message: format!(
                "Terminal locale '{lang}' does not explicitly indicate UTF-8; ANSI block QR code rendering may degrade"
            ),
            path: None,
        });
    }

    // 2. Color check
    if std::env::var("NO_COLOR").is_ok() {
        checks.push(DoctorCheck {
            category: "terminal_capabilities".into(),
            severity: CheckSeverity::Warning,
            message: "NO_COLOR environment variable is set; terminal color rendering is disabled"
                .into(),
            path: None,
        });
    } else {
        checks.push(DoctorCheck {
            category: "terminal_capabilities".into(),
            severity: CheckSeverity::Ok,
            message: "Terminal supports ANSI color output".into(),
            path: None,
        });
    }

    checks
}

/// Format the doctor report into a human-readable string
pub fn format_doctor_report(report: &DoctorReport, colorize: bool) -> String {
    let mut out = String::new();

    for check in &report.checks {
        let tag = match check.severity {
            CheckSeverity::Ok => {
                if colorize {
                    "\x1b[32m[OK]\x1b[0m   "
                } else {
                    "[OK]   "
                }
            }
            CheckSeverity::Warning => {
                if colorize {
                    "\x1b[33m[WARN]\x1b[0m "
                } else {
                    "[WARN] "
                }
            }
            CheckSeverity::Error => {
                if colorize {
                    "\x1b[31m[ERROR]\x1b[0m"
                } else {
                    "[ERROR]"
                }
            }
        };
        out.push_str(&format!("{} {}\n", tag, check.message));
    }

    out.push_str(&format!(
        "\nDoctor summary: {} passed, {} warning(s), {} error(s).\n",
        report.ok, report.warnings, report.errors
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_report_counts_and_healthy() {
        let checks = vec![
            DoctorCheck {
                category: "test".into(),
                severity: CheckSeverity::Ok,
                message: "all good".into(),
                path: None,
            },
            DoctorCheck {
                category: "test".into(),
                severity: CheckSeverity::Warning,
                message: "heads up".into(),
                path: None,
            },
        ];
        let report = DoctorReport::new(checks);
        assert_eq!(report.total_checks, 2);
        assert_eq!(report.ok, 1);
        assert_eq!(report.warnings, 1);
        assert_eq!(report.errors, 0);
        assert!(report.healthy);

        let with_error = vec![DoctorCheck {
            category: "test".into(),
            severity: CheckSeverity::Error,
            message: "boom".into(),
            path: None,
        }];
        let report_err = DoctorReport::new(with_error);
        assert_eq!(report_err.errors, 1);
        assert!(!report_err.healthy);
    }

    #[test]
    fn test_format_doctor_report() {
        let checks = vec![
            DoctorCheck {
                category: "keyring".into(),
                severity: CheckSeverity::Ok,
                message: "Keyring accessible".into(),
                path: None,
            },
            DoctorCheck {
                category: "clock_drift".into(),
                severity: CheckSeverity::Warning,
                message: "Clock drift high".into(),
                path: None,
            },
            DoctorCheck {
                category: "storage".into(),
                severity: CheckSeverity::Error,
                message: "Permissions bad".into(),
                path: None,
            },
        ];
        let report = DoctorReport::new(checks);
        let uncolored = format_doctor_report(&report, false);
        assert!(uncolored.contains("[OK]    Keyring accessible"));
        assert!(uncolored.contains("[WARN]  Clock drift high"));
        assert!(uncolored.contains("[ERROR] Permissions bad"));
        assert!(uncolored.contains("Doctor summary: 1 passed, 1 warning(s), 1 error(s)."));

        let colored = format_doctor_report(&report, true);
        assert!(colored.contains("\x1b[32m[OK]\x1b[0m"));
        assert!(colored.contains("\x1b[33m[WARN]\x1b[0m"));
        assert!(colored.contains("\x1b[31m[ERROR]\x1b[0m"));
    }

    #[test]
    fn test_check_terminal_capabilities() {
        let checks = check_terminal_capabilities();
        assert!(!checks.is_empty());
        assert!(checks.iter().any(|c| c.category == "terminal_capabilities"));
    }

    #[test]
    fn test_check_clipboard() {
        let checks = check_clipboard();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].category, "clipboard");
    }

    #[test]
    fn test_check_clock_drift_skipped() {
        std::env::set_var("TFA_DOCTOR_SKIP_NTP", "1");
        let checks = check_clock_drift(None);
        std::env::remove_var("TFA_DOCTOR_SKIP_NTP");
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].severity, CheckSeverity::Ok);
        assert!(checks[0].message.contains("skipped"));
    }

    #[test]
    fn test_check_storage_and_permissions_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        let cred_path = tmp.path().join("missing.json");
        let checks = check_storage_and_permissions(Some("file"), Some(&cred_path), None, &[]);
        assert!(checks
            .iter()
            .any(|c| c.message.contains("does not exist yet")));
    }
}
