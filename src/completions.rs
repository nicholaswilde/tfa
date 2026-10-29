use std::io::Write;
use std::path::PathBuf;

#[cfg(test)]
use std::sync::Arc;

use clap_complete::engine::CompletionCandidate;
use clap_complete::env::EnvCompleter;
use clap_complete::Shell;

use crate::namespace::{normalize_namespace, parse_key, NamespaceStore, DEFAULT_NAMESPACE};
use crate::storage::AccountStore;
use crate::StorageBackend;

#[cfg(test)]
thread_local! {
    static TEST_COMPLETION_STORE: std::cell::RefCell<Option<Arc<dyn AccountStore>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub fn set_test_store(store: Option<Arc<dyn AccountStore>>) {
    TEST_COMPLETION_STORE.with(|cell| {
        *cell.borrow_mut() = store;
    });
}

/// Check if master password protection is active and the session is currently locked.
pub fn is_storage_locked() -> bool {
    let (cfg, _) = match crate::config::resolve_configuration(None, None, None, None, None) {
        Ok(res) => res,
        Err(_) => return false,
    };

    if cfg.password_enabled {
        if let Ok(Some(record)) = crate::auth::get_password_record() {
            let token = crate::auth::compute_auth_token(&record);
            let timeout = cfg.password_timeout;
            if !crate::auth::is_session_unlocked(&token, timeout) {
                return true;
            }
        }
    }
    false
}

/// Inspect command line arguments after `--` to detect namespace scoping or flags.
fn detect_cli_namespace_from_args() -> (Option<String>, bool) {
    let args: Vec<String> = std::env::args().collect();
    let mut explicit_ns = None;
    let mut all_ns = false;

    // In dynamic completion, arguments are passed after `--`
    let start_idx = args
        .iter()
        .position(|a| a == "--")
        .map(|i| i + 1)
        .unwrap_or(0);
    let mut iter = args[start_idx..].iter();

    while let Some(arg) = iter.next() {
        if arg == "--all-namespaces" || arg == "-A" {
            all_ns = true;
        } else if arg == "-n" || arg == "--namespace" {
            if let Some(val) = iter.next() {
                explicit_ns = Some(val.clone());
            }
        } else if let Some(val) = arg.strip_prefix("--namespace=") {
            explicit_ns = Some(val.to_string());
        }
    }

    (explicit_ns, all_ns)
}

/// Safely open the active storage backend for autocompletion queries.
pub fn open_default_store() -> anyhow::Result<Box<dyn AccountStore>> {
    #[cfg(test)]
    {
        let test_store = TEST_COMPLETION_STORE.with(|cell| cell.borrow().clone());
        if let Some(store) = test_store {
            struct ArcStoreWrapper(Arc<dyn AccountStore>);
            impl AccountStore for ArcStoreWrapper {
                fn get_secret(&self, account: &str) -> anyhow::Result<String> {
                    self.0.get_secret(account)
                }
                fn set_secret(&self, account: &str, secret: &str) -> anyhow::Result<()> {
                    self.0.set_secret(account, secret)
                }
                fn delete_account(&self, account: &str) -> anyhow::Result<()> {
                    self.0.delete_account(account)
                }
                fn list_accounts(&self) -> anyhow::Result<Vec<String>> {
                    self.0.list_accounts()
                }
            }
            return Ok(Box::new(ArcStoreWrapper(store)));
        }
    }

    let (resolved_cfg, loaded_cfg) =
        crate::config::resolve_configuration(None, None, None, None, None)?;

    let store: Box<dyn AccountStore> = match resolved_cfg.storage {
        StorageBackend::Auto => Box::new(crate::storage::AutoStore::default_auto()?),
        StorageBackend::Keyring => Box::new(crate::storage::KeyringStore::default()),
        StorageBackend::File => Box::new(crate::storage::FileStore::new(
            crate::storage::FileStore::default_path()?,
        )),
        StorageBackend::Age => {
            let age_key = loaded_cfg.as_ref().and_then(|c| c.age_key.clone());
            let ssh_key = std::env::var("TFA_SSH_KEY")
                .ok()
                .map(PathBuf::from)
                .or_else(|| std::env::var("TFA_AGE_SSH_KEY").ok().map(PathBuf::from))
                .or_else(|| loaded_cfg.as_ref().and_then(|c| c.ssh_key.clone()));
            Box::new(crate::storage::AgeStore::new(
                crate::storage::AgeStore::default_path()?,
                age_key,
                ssh_key,
            ))
        }
    };
    Ok(store)
}

/// Generate candidate account names from an `AccountStore`.
pub fn account_candidates_from_store(
    store: &dyn AccountStore,
    target_namespace: Option<&str>,
    all_namespaces: bool,
) -> Vec<CompletionCandidate> {
    let Ok(all_keys) = store.list_accounts() else {
        return Vec::new();
    };

    let mut candidates = Vec::new();

    if let Some(explicit) = target_namespace {
        let norm = normalize_namespace(explicit);
        for key in all_keys {
            let (ns, acct) = parse_key(&key);
            if ns == norm {
                candidates.push(CompletionCandidate::new(acct));
            }
        }
    } else if all_namespaces {
        for key in all_keys {
            let (ns, acct) = parse_key(&key);
            if ns == DEFAULT_NAMESPACE {
                candidates.push(CompletionCandidate::new(acct));
            } else {
                candidates.push(CompletionCandidate::new(format!("{ns}/{acct}")));
            }
        }
    } else {
        let active_ns = crate::config::resolve_configuration(None, None, None, None, None)
            .ok()
            .map(|(r, _)| r.namespace)
            .unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());
        let norm_active = normalize_namespace(&active_ns);

        for key in all_keys {
            let (ns, acct) = parse_key(&key);
            if ns == norm_active {
                candidates.push(CompletionCandidate::new(acct));
            } else {
                candidates.push(CompletionCandidate::new(format!("{ns}/{acct}")));
            }
        }
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

/// Generate candidate namespace names from an `AccountStore`.
pub fn namespace_candidates_from_store(store: &dyn AccountStore) -> Vec<CompletionCandidate> {
    let Ok(namespaces) = NamespaceStore::list_all_namespaces(store) else {
        return Vec::new();
    };

    namespaces
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

/// Dynamic completion candidate generator for account-accepting arguments.
pub fn complete_accounts() -> Vec<CompletionCandidate> {
    if is_storage_locked() {
        return Vec::new();
    }
    let Ok(store) = open_default_store() else {
        return Vec::new();
    };
    let (target_ns, all_ns) = detect_cli_namespace_from_args();
    account_candidates_from_store(store.as_ref(), target_ns.as_deref(), all_ns)
}

/// Dynamic completion candidate generator for namespace arguments.
pub fn complete_namespaces() -> Vec<CompletionCandidate> {
    if is_storage_locked() {
        return Vec::new();
    }
    let Ok(store) = open_default_store() else {
        return Vec::new();
    };
    namespace_candidates_from_store(store.as_ref())
}

/// Generate dynamic registration completion script for the requested shell.
pub fn generate_dynamic_completions(shell: Shell, writer: &mut dyn Write) -> std::io::Result<()> {
    match shell {
        Shell::Bash => {
            clap_complete::env::Bash.write_registration("COMPLETE", "tfa", "tfa", "tfa", writer)
        }
        Shell::Zsh => {
            clap_complete::env::Zsh.write_registration("COMPLETE", "tfa", "tfa", "tfa", writer)
        }
        Shell::Fish => {
            clap_complete::env::Fish.write_registration("COMPLETE", "tfa", "tfa", "tfa", writer)
        }
        Shell::PowerShell => clap_complete::env::Powershell
            .write_registration("COMPLETE", "tfa", "tfa", "tfa", writer),
        Shell::Elvish => {
            clap_complete::env::Elvish.write_registration("COMPLETE", "tfa", "tfa", "tfa", writer)
        }
        _ => Err(std::io::Error::other(format!(
            "Unsupported shell for dynamic completions: {shell}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStore;

    #[test]
    fn test_account_candidates_default_namespace() {
        let store = MemoryStore::new();
        store.set_secret("github", "secret1").unwrap();
        store.set_secret("google", "secret2").unwrap();
        store.set_secret("work:aws", "secret3").unwrap();

        // Active namespace = default
        let candidates = account_candidates_from_store(&store, None, false);
        let vals: Vec<String> = candidates
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();

        assert_eq!(vals, vec!["github", "google", "work/aws"]);
    }

    #[test]
    fn test_account_candidates_explicit_namespace() {
        let store = MemoryStore::new();
        store.set_secret("github", "secret1").unwrap();
        store.set_secret("work:aws", "secret2").unwrap();
        store.set_secret("work:jira", "secret3").unwrap();

        let candidates = account_candidates_from_store(&store, Some("work"), false);
        let vals: Vec<String> = candidates
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();

        assert_eq!(vals, vec!["aws", "jira"]);
    }

    #[test]
    fn test_account_candidates_all_namespaces() {
        let store = MemoryStore::new();
        store.set_secret("github", "secret1").unwrap();
        store.set_secret("work:aws", "secret2").unwrap();

        let candidates = account_candidates_from_store(&store, None, true);
        let vals: Vec<String> = candidates
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();

        assert_eq!(vals, vec!["github", "work/aws"]);
    }

    #[test]
    fn test_graceful_failure_when_store_unreachable() {
        struct ErrorStore;
        impl AccountStore for ErrorStore {
            fn get_secret(&self, _account: &str) -> anyhow::Result<String> {
                anyhow::bail!("Storage unreachable")
            }
            fn set_secret(&self, _account: &str, _secret: &str) -> anyhow::Result<()> {
                anyhow::bail!("Storage unreachable")
            }
            fn delete_account(&self, _account: &str) -> anyhow::Result<()> {
                anyhow::bail!("Storage unreachable")
            }
            fn list_accounts(&self) -> anyhow::Result<Vec<String>> {
                anyhow::bail!("Keyring locked or storage error")
            }
        }

        let err_store = ErrorStore;
        let accts = account_candidates_from_store(&err_store, None, false);
        assert!(accts.is_empty());

        let namespaces = namespace_candidates_from_store(&err_store);
        assert!(namespaces.is_empty());

        // Also test through complete_accounts with test store set to ErrorStore
        set_test_store(Some(Arc::new(err_store)));
        assert!(complete_accounts().is_empty());
        assert!(complete_namespaces().is_empty());
        set_test_store(None);
    }

    #[test]
    fn test_namespace_candidates() {
        let store = MemoryStore::new();
        store.set_secret("github", "secret1").unwrap();
        store.set_secret("work:aws", "secret2").unwrap();
        store.set_secret("personal:bank", "secret3").unwrap();

        let candidates = namespace_candidates_from_store(&store);
        let vals: Vec<String> = candidates
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();

        assert_eq!(vals, vec!["default", "personal", "work"]);
    }

    #[test]
    fn test_namespace_candidates_empty_store_returns_default() {
        let store = MemoryStore::new();
        let candidates = namespace_candidates_from_store(&store);
        let vals: Vec<String> = candidates
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();

        assert_eq!(vals, vec!["default"]);
    }

    #[test]
    fn test_generate_dynamic_completions_all_shells() {
        let shells = [
            (Shell::Bash, "_clap_complete_tfa"),
            (Shell::Zsh, "_clap_dynamic_completer_tfa"),
            (Shell::Fish, "COMPLETE=fish"),
            (Shell::PowerShell, "Register-ArgumentCompleter"),
            (Shell::Elvish, "edit:completion:arg-completer"),
        ];

        for (shell, expected_snippet) in shells {
            let mut buf = Vec::new();
            assert!(generate_dynamic_completions(shell, &mut buf).is_ok());
            let out = String::from_utf8(buf).unwrap();
            assert!(
                out.contains(expected_snippet),
                "Shell {shell:?} missing snippet {expected_snippet}"
            );
        }
    }

    #[test]
    fn test_complete_accounts_and_namespaces_via_test_store() {
        let store = Arc::new(MemoryStore::new());
        store.set_secret("github", "secret1").unwrap();
        store.set_secret("work:aws", "secret2").unwrap();
        set_test_store(Some(store));

        let acct_candidates = complete_accounts();
        let acct_vals: Vec<String> = acct_candidates
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();
        assert!(acct_vals.contains(&"github".to_string()));
        assert!(acct_vals.contains(&"work/aws".to_string()));

        let ns_candidates = complete_namespaces();
        let ns_vals: Vec<String> = ns_candidates
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();
        assert!(ns_vals.contains(&"default".to_string()));
        assert!(ns_vals.contains(&"work".to_string()));

        set_test_store(None);
    }

    #[test]
    fn test_clap_complete_engine_complete_subcommands_and_args() {
        use clap::CommandFactory;

        let store = Arc::new(MemoryStore::new());
        store.set_secret("github", "secret1").unwrap();
        store.set_secret("google", "secret2").unwrap();
        store.set_secret("work:aws", "secret3").unwrap();
        set_test_store(Some(store));

        let mut cmd = crate::Cli::command();

        // 1. Complete subcommands from "tfa "
        let res = clap_complete::engine::complete(&mut cmd, vec!["tfa".into(), "".into()], 1, None)
            .unwrap();
        let subcmds: Vec<String> = res
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();
        assert!(subcmds.contains(&"generate".to_string()));
        assert!(subcmds.contains(&"watch".to_string()));
        assert!(subcmds.contains(&"delete".to_string()));
        assert!(subcmds.contains(&"namespace".to_string()));

        // 2. Complete accounts for "tfa generate "
        let mut cmd = crate::Cli::command();
        let res = clap_complete::engine::complete(
            &mut cmd,
            vec!["tfa".into(), "generate".into(), "".into()],
            2,
            None,
        )
        .unwrap();
        let acct_vals: Vec<String> = res
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();
        assert!(acct_vals.contains(&"github".to_string()));
        assert!(acct_vals.contains(&"google".to_string()));
        assert!(acct_vals.contains(&"work/aws".to_string()));

        // 3. Complete accounts for "tfa move "
        let mut cmd = crate::Cli::command();
        let res = clap_complete::engine::complete(
            &mut cmd,
            vec!["tfa".into(), "move".into(), "".into()],
            2,
            None,
        )
        .unwrap();
        let move_accts: Vec<String> = res
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();
        assert!(move_accts.contains(&"github".to_string()));

        // 4. Complete namespaces for "tfa move github --to "
        let mut cmd = crate::Cli::command();
        let res = clap_complete::engine::complete(
            &mut cmd,
            vec![
                "tfa".into(),
                "move".into(),
                "github".into(),
                "--to".into(),
                "".into(),
            ],
            4,
            None,
        )
        .unwrap();
        let ns_vals: Vec<String> = res
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();
        assert!(ns_vals.contains(&"default".to_string()));
        assert!(ns_vals.contains(&"work".to_string()));

        // 5. Complete namespaces for "tfa -n "
        let mut cmd = crate::Cli::command();
        let res = clap_complete::engine::complete(
            &mut cmd,
            vec!["tfa".into(), "-n".into(), "".into()],
            2,
            None,
        )
        .unwrap();
        let global_ns_vals: Vec<String> = res
            .into_iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect();
        assert!(global_ns_vals.contains(&"default".to_string()));
        assert!(global_ns_vals.contains(&"work".to_string()));

        set_test_store(None);
    }
}
