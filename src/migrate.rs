use anyhow::{bail, Result};
use clap::ValueEnum;
use std::io::Write;
use std::path::PathBuf;

use crate::storage::{AccountStore, AgeStore, FileStore, KeyringStore};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MigrateBackend {
    Keyring,
    File,
    Age,
}

impl std::fmt::Display for MigrateBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyring => write!(f, "keyring"),
            Self::File => write!(f, "file"),
            Self::Age => write!(f, "age"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MigrateOptions {
    pub overwrite: bool,
    pub dry_run: bool,
    pub purge_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MigrateStats {
    pub transferred: usize,
    pub updated: usize,
    pub skipped: usize,
    pub purged: usize,
}

pub fn create_store_for_backend(
    backend: MigrateBackend,
    age_key: Option<PathBuf>,
    ssh_key: Option<PathBuf>,
) -> Result<Box<dyn AccountStore>> {
    match backend {
        MigrateBackend::Keyring => Ok(Box::new(KeyringStore::default())),
        MigrateBackend::File => Ok(Box::new(FileStore::new(FileStore::default_path()?))),
        MigrateBackend::Age => Ok(Box::new(AgeStore::new(
            AgeStore::default_path()?,
            age_key,
            ssh_key,
        ))),
    }
}

pub fn migrate_store(
    from_store: &dyn AccountStore,
    to_store: &dyn AccountStore,
    options: MigrateOptions,
    out_writer: &mut dyn Write,
) -> Result<MigrateStats> {
    let mut stats = MigrateStats::default();
    let mut accounts = from_store.list_accounts()?;
    accounts.sort();

    if accounts.is_empty() {
        writeln!(out_writer, "No accounts found in source storage.")?;
        if options.dry_run {
            if options.purge_source {
                writeln!(
                    out_writer,
                    "[dry-run] Summary: 0 to transfer, 0 to update, 0 to skip, 0 to purge."
                )?;
            } else {
                writeln!(
                    out_writer,
                    "[dry-run] Summary: 0 to transfer, 0 to update, 0 to skip."
                )?;
            }
        } else if options.purge_source {
            writeln!(
                out_writer,
                "Summary: 0 transferred, 0 updated, 0 skipped, 0 purged."
            )?;
        } else {
            writeln!(out_writer, "Summary: 0 transferred, 0 updated, 0 skipped.")?;
        }
        return Ok(stats);
    }

    let mut migrated_keys = Vec::new();

    for key in &accounts {
        let secret = from_store.get_secret(key)?;
        let exists = to_store.get_secret(key).is_ok();

        if exists {
            if options.overwrite {
                if !options.dry_run {
                    to_store.set_secret(key, &secret)?;
                }
                stats.updated += 1;
                migrated_keys.push(key.clone());
                if options.dry_run {
                    writeln!(out_writer, "[dry-run] Would update account '{key}'")?;
                } else {
                    writeln!(out_writer, "Updated account '{key}'")?;
                }
            } else {
                stats.skipped += 1;
                if options.dry_run {
                    writeln!(out_writer, "[dry-run] Would skip existing account '{key}'")?;
                } else {
                    writeln!(
                        out_writer,
                        "Skipping existing account '{key}' (use --overwrite to replace)"
                    )?;
                }
            }
        } else {
            if !options.dry_run {
                to_store.set_secret(key, &secret)?;
            }
            stats.transferred += 1;
            migrated_keys.push(key.clone());
            if options.dry_run {
                writeln!(out_writer, "[dry-run] Would transfer account '{key}'")?;
            } else {
                writeln!(out_writer, "Transferred account '{key}'")?;
            }
        }
    }

    if options.purge_source {
        for key in &migrated_keys {
            if options.dry_run {
                stats.purged += 1;
                writeln!(
                    out_writer,
                    "[dry-run] Would purge account '{key}' from source"
                )?;
            } else {
                from_store.delete_account(key)?;
                stats.purged += 1;
                writeln!(out_writer, "Purged account '{key}' from source")?;
            }
        }
    }

    if options.dry_run {
        if options.purge_source {
            writeln!(
                out_writer,
                "[dry-run] Summary: {} to transfer, {} to update, {} to skip, {} to purge.",
                stats.transferred, stats.updated, stats.skipped, stats.purged
            )?;
        } else {
            writeln!(
                out_writer,
                "[dry-run] Summary: {} to transfer, {} to update, {} to skip.",
                stats.transferred, stats.updated, stats.skipped
            )?;
        }
    } else if options.purge_source {
        writeln!(
            out_writer,
            "Summary: {} transferred, {} updated, {} skipped, {} purged.",
            stats.transferred, stats.updated, stats.skipped, stats.purged
        )?;
    } else {
        writeln!(
            out_writer,
            "Summary: {} transferred, {} updated, {} skipped.",
            stats.transferred, stats.updated, stats.skipped
        )?;
    }

    Ok(stats)
}

pub fn run_migrate(
    from: MigrateBackend,
    to: MigrateBackend,
    options: MigrateOptions,
    age_key: Option<PathBuf>,
    ssh_key: Option<PathBuf>,
    out_writer: &mut dyn Write,
) -> Result<MigrateStats> {
    if from == to {
        bail!("Source and destination storage backends cannot be the same");
    }
    let from_store = create_store_for_backend(from, age_key.clone(), ssh_key.clone())?;
    let to_store = create_store_for_backend(to, age_key, ssh_key)?;
    migrate_store(from_store.as_ref(), to_store.as_ref(), options, out_writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::NamespaceStore;
    use crate::storage::MemoryStore;
    use tempfile::tempdir;

    #[test]
    fn test_migrate_backend_display() {
        assert_eq!(MigrateBackend::Keyring.to_string(), "keyring");
        assert_eq!(MigrateBackend::File.to_string(), "file");
        assert_eq!(MigrateBackend::Age.to_string(), "age");
    }

    #[test]
    fn test_migrate_identical_source_and_dest_fails() {
        let mut out = Vec::new();
        let res = run_migrate(
            MigrateBackend::File,
            MigrateBackend::File,
            MigrateOptions::default(),
            None,
            None,
            &mut out,
        );
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Source and destination storage backends cannot be the same"));
    }

    #[test]
    fn test_migrate_empty_source() {
        let from_store = MemoryStore::default();
        let to_store = MemoryStore::default();

        let mut out = Vec::new();
        let stats =
            migrate_store(&from_store, &to_store, MigrateOptions::default(), &mut out).unwrap();

        assert_eq!(stats.transferred, 0);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.purged, 0);

        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("No accounts found in source storage."));
        assert!(out_str.contains("Summary: 0 transferred, 0 updated, 0 skipped."));
    }

    #[test]
    fn test_migrate_empty_source_dry_run_purge() {
        let from_store = MemoryStore::default();
        let to_store = MemoryStore::default();

        let mut out = Vec::new();
        let opts = MigrateOptions {
            overwrite: false,
            dry_run: true,
            purge_source: true,
        };
        let stats = migrate_store(&from_store, &to_store, opts, &mut out).unwrap();
        assert_eq!(stats.transferred, 0);
        assert_eq!(stats.purged, 0);

        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str
            .contains("[dry-run] Summary: 0 to transfer, 0 to update, 0 to skip, 0 to purge."));
    }

    #[test]
    fn test_migrate_basic_in_memory_transfers_all_namespaces() {
        let from_store = MemoryStore::default();
        let to_store = MemoryStore::default();

        from_store.set_secret("github", "SECRET_GITHUB").unwrap();
        from_store.set_secret("work:aws", "SECRET_AWS").unwrap();
        from_store
            .set_secret("personal:google", "SECRET_GOOGLE")
            .unwrap();

        let mut out = Vec::new();
        let stats =
            migrate_store(&from_store, &to_store, MigrateOptions::default(), &mut out).unwrap();

        assert_eq!(stats.transferred, 3);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.purged, 0);

        // Verify direct root store keys
        assert_eq!(to_store.get_secret("github").unwrap(), "SECRET_GITHUB");
        assert_eq!(to_store.get_secret("work:aws").unwrap(), "SECRET_AWS");
        assert_eq!(
            to_store.get_secret("personal:google").unwrap(),
            "SECRET_GOOGLE"
        );

        // Verify namespace-scoped access
        let default_ns = NamespaceStore::new(&to_store, "default");
        assert_eq!(default_ns.list_accounts().unwrap(), vec!["github"]);

        let work_ns = NamespaceStore::new(&to_store, "work");
        assert_eq!(work_ns.list_accounts().unwrap(), vec!["aws"]);
        assert_eq!(work_ns.get_secret("aws").unwrap(), "SECRET_AWS");

        let personal_ns = NamespaceStore::new(&to_store, "personal");
        assert_eq!(personal_ns.list_accounts().unwrap(), vec!["google"]);
        assert_eq!(personal_ns.get_secret("google").unwrap(), "SECRET_GOOGLE");

        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("Transferred account 'github'"));
        assert!(out_str.contains("Transferred account 'personal:google'"));
        assert!(out_str.contains("Transferred account 'work:aws'"));
        assert!(out_str.contains("Summary: 3 transferred, 0 updated, 0 skipped."));
    }

    #[test]
    fn test_migrate_with_conflicts_skip_default() {
        let from_store = MemoryStore::default();
        let to_store = MemoryStore::default();

        from_store.set_secret("existing", "NEW_SECRET").unwrap();
        from_store.set_secret("new_account", "SECRET_NEW").unwrap();

        to_store.set_secret("existing", "ORIGINAL_SECRET").unwrap();

        let mut out = Vec::new();
        let stats = migrate_store(
            &from_store,
            &to_store,
            MigrateOptions {
                overwrite: false,
                dry_run: false,
                purge_source: false,
            },
            &mut out,
        )
        .unwrap();

        assert_eq!(stats.transferred, 1);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.skipped, 1);

        // Original secret remains unchanged
        assert_eq!(to_store.get_secret("existing").unwrap(), "ORIGINAL_SECRET");
        assert_eq!(to_store.get_secret("new_account").unwrap(), "SECRET_NEW");

        let out_str = String::from_utf8(out).unwrap();
        assert!(
            out_str.contains("Skipping existing account 'existing' (use --overwrite to replace)")
        );
        assert!(out_str.contains("Transferred account 'new_account'"));
        assert!(out_str.contains("Summary: 1 transferred, 0 updated, 1 skipped."));
    }

    #[test]
    fn test_migrate_with_conflicts_overwrite() {
        let from_store = MemoryStore::default();
        let to_store = MemoryStore::default();

        from_store.set_secret("existing", "UPDATED_SECRET").unwrap();
        from_store.set_secret("new_account", "SECRET_NEW").unwrap();

        to_store.set_secret("existing", "ORIGINAL_SECRET").unwrap();

        let mut out = Vec::new();
        let stats = migrate_store(
            &from_store,
            &to_store,
            MigrateOptions {
                overwrite: true,
                dry_run: false,
                purge_source: false,
            },
            &mut out,
        )
        .unwrap();

        assert_eq!(stats.transferred, 1);
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.skipped, 0);

        // Secret is overwritten
        assert_eq!(to_store.get_secret("existing").unwrap(), "UPDATED_SECRET");
        assert_eq!(to_store.get_secret("new_account").unwrap(), "SECRET_NEW");

        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("Updated account 'existing'"));
        assert!(out_str.contains("Transferred account 'new_account'"));
        assert!(out_str.contains("Summary: 1 transferred, 1 updated, 0 skipped."));
    }

    #[test]
    fn test_migrate_dry_run_preserves_both_stores() {
        let from_store = MemoryStore::default();
        let to_store = MemoryStore::default();

        from_store.set_secret("github", "SECRET_GITHUB").unwrap();
        from_store.set_secret("aws", "SECRET_AWS").unwrap();
        to_store.set_secret("aws", "OLD_AWS").unwrap();

        let mut out = Vec::new();
        let stats = migrate_store(
            &from_store,
            &to_store,
            MigrateOptions {
                overwrite: true,
                dry_run: true,
                purge_source: true,
            },
            &mut out,
        )
        .unwrap();

        assert_eq!(stats.transferred, 1);
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.purged, 2);

        // Destination was not changed
        assert_eq!(to_store.get_secret("aws").unwrap(), "OLD_AWS");
        assert!(to_store.get_secret("github").is_err());

        // Source was not purged
        assert_eq!(from_store.get_secret("github").unwrap(), "SECRET_GITHUB");
        assert_eq!(from_store.get_secret("aws").unwrap(), "SECRET_AWS");

        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("[dry-run] Would update account 'aws'"));
        assert!(out_str.contains("[dry-run] Would transfer account 'github'"));
        assert!(out_str.contains("[dry-run] Would purge account 'aws' from source"));
        assert!(out_str.contains("[dry-run] Would purge account 'github' from source"));
        assert!(out_str
            .contains("[dry-run] Summary: 1 to transfer, 1 to update, 0 to skip, 2 to purge."));
    }

    #[test]
    fn test_migrate_purge_source_only_deletes_migrated_accounts() {
        let from_store = MemoryStore::default();
        let to_store = MemoryStore::default();

        from_store.set_secret("account1", "SEC1").unwrap();
        from_store.set_secret("account2", "SEC2").unwrap();
        from_store.set_secret("account3", "SEC3").unwrap();

        // account2 already exists in to_store and overwrite is false -> skipped
        to_store.set_secret("account2", "EXISTING_SEC2").unwrap();

        let mut out = Vec::new();
        let stats = migrate_store(
            &from_store,
            &to_store,
            MigrateOptions {
                overwrite: false,
                dry_run: false,
                purge_source: true,
            },
            &mut out,
        )
        .unwrap();

        assert_eq!(stats.transferred, 2);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.purged, 2);

        // account1 and account3 purged from source; account2 remained in source because skipped
        assert_eq!(from_store.list_accounts().unwrap(), vec!["account2"]);
        assert_eq!(from_store.get_secret("account2").unwrap(), "SEC2");

        // Destination has all 3
        assert_eq!(
            to_store.list_accounts().unwrap(),
            vec!["account1", "account2", "account3"]
        );

        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("Purged account 'account1' from source"));
        assert!(out_str.contains("Purged account 'account3' from source"));
        assert!(!out_str.contains("Purged account 'account2' from source"));
        assert!(out_str.contains("Summary: 2 transferred, 0 updated, 1 skipped, 2 purged."));
    }

    #[test]
    fn test_migrate_file_to_age_and_reverse_integration() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("credentials.json");
        let age_path = dir.path().join("credentials.age");
        let key_path = dir.path().join("age-key.txt");

        // Generate age key
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public();
        let secret_str = identity.to_string();
        use age::secrecy::ExposeSecret;
        let key_content = format!("# public: {}\n{}\n", pubkey, secret_str.expose_secret());
        std::fs::write(&key_path, key_content).unwrap();

        let file_store = FileStore::new(file_path.clone());
        let age_store = AgeStore::new(age_path.clone(), Some(key_path.clone()), None);

        // Populate file_store
        file_store
            .set_secret("default_acc", "JBSWY3DPEHPK3PXP")
            .unwrap();
        file_store
            .set_secret("work:corp_vpn", "KRSXG5CTMVRXEZLU")
            .unwrap();

        // 1. Migrate file -> age with purge_source
        let mut out = Vec::new();
        let stats = migrate_store(
            &file_store,
            &age_store,
            MigrateOptions {
                overwrite: false,
                dry_run: false,
                purge_source: true,
            },
            &mut out,
        )
        .unwrap();

        assert_eq!(stats.transferred, 2);
        assert_eq!(stats.purged, 2);

        // File store is now empty
        assert_eq!(file_store.list_accounts().unwrap(), Vec::<String>::new());

        // Age store contains all accounts
        assert_eq!(
            age_store.list_accounts().unwrap(),
            vec!["default_acc", "work:corp_vpn"]
        );
        assert_eq!(
            age_store.get_secret("default_acc").unwrap(),
            "JBSWY3DPEHPK3PXP"
        );
        assert_eq!(
            age_store.get_secret("work:corp_vpn").unwrap(),
            "KRSXG5CTMVRXEZLU"
        );

        // Verify age file on disk is encrypted
        let disk_bytes = std::fs::read(&age_path).unwrap();
        assert!(!String::from_utf8_lossy(&disk_bytes).contains("JBSWY3DPEHPK3PXP"));

        // 2. Migrate age -> file without purge_source
        let mut out_reverse = Vec::new();
        let stats_rev = migrate_store(
            &age_store,
            &file_store,
            MigrateOptions::default(),
            &mut out_reverse,
        )
        .unwrap();

        assert_eq!(stats_rev.transferred, 2);
        assert_eq!(stats_rev.purged, 0);

        // Both stores now have the accounts
        assert_eq!(
            file_store.list_accounts().unwrap(),
            vec!["default_acc", "work:corp_vpn"]
        );
        assert_eq!(
            age_store.list_accounts().unwrap(),
            vec!["default_acc", "work:corp_vpn"]
        );
    }
}
