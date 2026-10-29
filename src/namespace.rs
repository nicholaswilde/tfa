use anyhow::{bail, Result};
use std::collections::HashSet;

use crate::storage::AccountStore;

pub const DEFAULT_NAMESPACE: &str = "default";

/// Normalize namespace name (lowercase, trimmed).
pub fn normalize_namespace(ns: &str) -> String {
    let trimmed = ns.trim();
    if trimmed.is_empty() {
        DEFAULT_NAMESPACE.to_string()
    } else {
        trimmed.to_lowercase()
    }
}

/// Split an account identifier if it uses inline `namespace/account` addressing.
pub fn split_inline_account(account: &str) -> Option<(String, String)> {
    let (ns, acct) = account.split_once('/')?;
    let ns_trim = ns.trim();
    let acct_trim = acct.trim();
    if ns_trim.is_empty() || acct_trim.is_empty() {
        return None;
    }
    Some((normalize_namespace(ns_trim), acct_trim.to_string()))
}

/// Qualified key representation in storage:
/// - If namespace is "default", the key is stored plain as `<account>`.
/// - For any other namespace (e.g. "work", "home"), it is stored as `<namespace>:<account>`.
///
/// This guarantees 100% backwards compatibility and clean key separation.
pub fn scoped_key(namespace: &str, account: &str) -> String {
    let ns = normalize_namespace(namespace);
    if ns == DEFAULT_NAMESPACE {
        account.to_string()
    } else {
        format!("{ns}:{account}")
    }
}

/// Parse a raw storage key into `(namespace, account_name)`.
/// Keys without a `:` prefix belong to "default".
pub fn parse_key(key: &str) -> (String, String) {
    if let Some((ns, acct)) = key.split_once(':') {
        (normalize_namespace(ns), acct.to_string())
    } else {
        (DEFAULT_NAMESPACE.to_string(), key.to_string())
    }
}

/// Namespace-scoped wrapper around an underlying `AccountStore`.
pub struct NamespaceStore<'a> {
    store: &'a dyn AccountStore,
    namespace: String,
}

impl<'a> NamespaceStore<'a> {
    pub fn new(store: &'a dyn AccountStore, namespace: &str) -> Self {
        Self {
            store,
            namespace: normalize_namespace(namespace),
        }
    }

    /// List all distinct namespaces currently stored.
    pub fn list_all_namespaces(store: &dyn AccountStore) -> Result<Vec<String>> {
        let all_keys = store.list_accounts()?;
        let mut set = HashSet::new();
        set.insert(DEFAULT_NAMESPACE.to_string());

        for k in all_keys {
            let (ns, _) = parse_key(&k);
            set.insert(ns);
        }

        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();
        Ok(list)
    }

    /// Rename an entire namespace, migrating all its contained accounts to the target namespace.
    pub fn rename_namespace(
        store: &dyn AccountStore,
        old_name: &str,
        new_name: &str,
    ) -> Result<usize> {
        let old_ns = normalize_namespace(old_name);
        let new_ns = normalize_namespace(new_name);

        if old_ns == new_ns {
            bail!("Source and destination namespace are identical ('{old_ns}')");
        }

        let all_keys = store.list_accounts()?;
        let mut migrated = 0;

        for key in all_keys {
            let (ns, acct) = parse_key(&key);
            if ns == old_ns {
                let secret = store.get_secret(&key)?;
                let new_key = scoped_key(&new_ns, &acct);
                store.set_secret(&new_key, &secret)?;
                let _ = store.delete_account(&key);
                migrated += 1;
            }
        }

        Ok(migrated)
    }

    /// Delete all accounts within a namespace.
    pub fn delete_namespace(store: &dyn AccountStore, namespace: &str) -> Result<usize> {
        let target_ns = normalize_namespace(namespace);
        let all_keys = store.list_accounts()?;
        let mut deleted = 0;

        for key in all_keys {
            let (ns, _) = parse_key(&key);
            if ns == target_ns {
                store.delete_account(&key)?;
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    /// Rename an account within a specific namespace.
    pub fn rename_account(
        store: &dyn AccountStore,
        namespace: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<()> {
        let old_name = old_name.trim();
        let new_name = new_name.trim();
        if old_name.is_empty() || new_name.is_empty() {
            bail!("Account name cannot be empty");
        }
        if old_name == new_name {
            bail!("Source and destination account names are identical ('{old_name}')");
        }

        let ns = normalize_namespace(namespace);
        let ns_store = NamespaceStore::new(store, &ns);

        let secret = ns_store.get_secret(old_name)?;

        if ns_store.get_secret(new_name).is_ok() {
            bail!("Account '{new_name}' already exists in namespace '{ns}'");
        }

        ns_store.set_secret(new_name, &secret)?;
        if let Err(e) = ns_store.delete_account(old_name) {
            let _ = ns_store.delete_account(new_name);
            return Err(e);
        }

        Ok(())
    }

    /// Move an account from one namespace to another.
    pub fn move_account(
        store: &dyn AccountStore,
        from_namespace: &str,
        account: &str,
        to_namespace: &str,
    ) -> Result<()> {
        let account = account.trim();
        if account.is_empty() {
            bail!("Account name cannot be empty");
        }

        let from_ns = normalize_namespace(from_namespace);
        let to_ns = normalize_namespace(to_namespace);

        if from_ns == to_ns {
            bail!("Source and destination namespace are identical ('{from_ns}')");
        }

        let from_store = NamespaceStore::new(store, &from_ns);
        let to_store = NamespaceStore::new(store, &to_ns);

        let secret = from_store.get_secret(account)?;

        if to_store.get_secret(account).is_ok() {
            bail!("Account '{account}' already exists in namespace '{to_ns}'");
        }

        to_store.set_secret(account, &secret)?;
        if let Err(e) = from_store.delete_account(account) {
            let _ = to_store.delete_account(account);
            return Err(e);
        }

        Ok(())
    }
}

impl<'a> AccountStore for NamespaceStore<'a> {
    fn set_secret(&self, account: &str, secret: &str) -> Result<()> {
        let key = scoped_key(&self.namespace, account);
        self.store.set_secret(&key, secret)
    }

    fn get_secret(&self, account: &str) -> Result<String> {
        let key = scoped_key(&self.namespace, account);
        if let Ok(sec) = self.store.get_secret(&key) {
            return Ok(sec);
        }

        bail!(
            "Account '{account}' not found in namespace '{}'",
            self.namespace
        )
    }

    fn list_accounts(&self) -> Result<Vec<String>> {
        let all_keys = self.store.list_accounts()?;
        let mut accounts = Vec::new();

        for key in all_keys {
            let (ns, acct) = parse_key(&key);
            if ns == self.namespace && !accounts.contains(&acct) {
                accounts.push(acct);
            }
        }

        accounts.sort();
        Ok(accounts)
    }

    fn delete_account(&self, account: &str) -> Result<()> {
        let key = scoped_key(&self.namespace, account);
        self.store.delete_account(&key).map_err(|_| {
            anyhow::anyhow!(
                "Account '{account}' not found in namespace '{}'",
                self.namespace
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStore;

    #[test]
    fn test_scoped_key_and_parse() {
        assert_eq!(scoped_key("work", "aws"), "work:aws");
        assert_eq!(scoped_key("DEFAULT", "github"), "github");

        let (ns, acct) = parse_key("personal:google");
        assert_eq!(ns, "personal");
        assert_eq!(acct, "google");

        let (legacy_ns, legacy_acct) = parse_key("github");
        assert_eq!(legacy_ns, "default");
        assert_eq!(legacy_acct, "github");
    }

    #[test]
    fn test_namespace_isolation_and_legacy_compatibility() {
        let root_store = MemoryStore::new();

        // 1. Add legacy account to root store
        root_store.set_secret("legacy_acc", "SEC_LEGACY").unwrap();

        // Default namespace store sees legacy account
        let default_store = NamespaceStore::new(&root_store, "default");
        assert_eq!(
            default_store.get_secret("legacy_acc").unwrap(),
            "SEC_LEGACY"
        );
        assert_eq!(default_store.list_accounts().unwrap(), vec!["legacy_acc"]);

        // Work namespace store does NOT see legacy account
        let work_store = NamespaceStore::new(&root_store, "work");
        assert!(work_store.get_secret("legacy_acc").is_err());
        assert!(work_store.list_accounts().unwrap().is_empty());

        // 2. Add account to work namespace
        work_store.set_secret("aws", "SEC_WORK_AWS").unwrap();
        assert_eq!(work_store.get_secret("aws").unwrap(), "SEC_WORK_AWS");
        assert_eq!(work_store.list_accounts().unwrap(), vec!["aws"]);

        // Default store does not see work account
        assert!(default_store.get_secret("aws").is_err());
    }

    #[test]
    fn test_namespace_list_rename_and_delete() {
        let root_store = MemoryStore::new();
        let home_store = NamespaceStore::new(&root_store, "home");
        home_store.set_secret("router", "SEC_ROUTER").unwrap();

        let work_store = NamespaceStore::new(&root_store, "work");
        work_store.set_secret("jira", "SEC_JIRA").unwrap();
        work_store.set_secret("github", "SEC_GH").unwrap();

        // List namespaces
        let namespaces = NamespaceStore::list_all_namespaces(&root_store).unwrap();
        assert_eq!(namespaces, vec!["default", "home", "work"]);

        // Rename work -> office
        let migrated = NamespaceStore::rename_namespace(&root_store, "work", "office").unwrap();
        assert_eq!(migrated, 2);

        let office_store = NamespaceStore::new(&root_store, "office");
        assert_eq!(office_store.get_secret("jira").unwrap(), "SEC_JIRA");
        assert_eq!(office_store.get_secret("github").unwrap(), "SEC_GH");

        // Old work store is now empty
        let empty_work = NamespaceStore::new(&root_store, "work");
        assert!(empty_work.list_accounts().unwrap().is_empty());

        // Delete office namespace
        let deleted = NamespaceStore::delete_namespace(&root_store, "office").unwrap();
        assert_eq!(deleted, 2);
        assert!(office_store.list_accounts().unwrap().is_empty());
    }

    #[test]
    fn test_namespace_account_rename() {
        let root_store = MemoryStore::new();

        // 1. Rename in default namespace
        let default_store = NamespaceStore::new(&root_store, "default");
        default_store
            .set_secret("github", "SEC_GITHUB_URL")
            .unwrap();
        default_store.set_secret("gitlab", "SEC_GITLAB").unwrap();

        // Error: identical names
        let err_ident =
            NamespaceStore::rename_account(&root_store, "default", "github", "github").unwrap_err();
        assert!(err_ident.to_string().contains("identical"));

        // Error: empty name
        let err_empty =
            NamespaceStore::rename_account(&root_store, "default", "", "gh").unwrap_err();
        assert!(err_empty.to_string().contains("empty"));

        // Error: target already exists
        let err_dup =
            NamespaceStore::rename_account(&root_store, "default", "github", "gitlab").unwrap_err();
        assert!(err_dup.to_string().contains("already exists"));

        // Error: old not found
        let err_missing =
            NamespaceStore::rename_account(&root_store, "default", "missing", "target")
                .unwrap_err();
        assert!(err_missing.to_string().contains("not found"));

        // Success: rename github -> gh
        NamespaceStore::rename_account(&root_store, "default", "github", "gh").unwrap();
        assert_eq!(default_store.get_secret("gh").unwrap(), "SEC_GITHUB_URL");
        assert!(default_store.get_secret("github").is_err());

        // 2. Rename in custom namespace (work)
        let work_store = NamespaceStore::new(&root_store, "work");
        work_store.set_secret("aws", "SEC_AWS_CUSTOM").unwrap();

        NamespaceStore::rename_account(&root_store, "work", "aws", "amazon").unwrap();
        assert_eq!(work_store.get_secret("amazon").unwrap(), "SEC_AWS_CUSTOM");
        assert!(work_store.get_secret("aws").is_err());
    }

    #[test]
    fn test_namespace_account_move() {
        let root_store = MemoryStore::new();

        let default_store = NamespaceStore::new(&root_store, "default");
        default_store.set_secret("router", "SEC_ROUTER").unwrap();

        let work_store = NamespaceStore::new(&root_store, "work");
        work_store.set_secret("jira", "SEC_JIRA").unwrap();

        // Error: source not found
        let err_missing =
            NamespaceStore::move_account(&root_store, "default", "missing", "work").unwrap_err();
        assert!(err_missing.to_string().contains("not found"));

        // Error: empty account name
        let err_empty =
            NamespaceStore::move_account(&root_store, "default", "  ", "work").unwrap_err();
        assert!(err_empty.to_string().contains("empty"));

        // Error: identical namespace
        let err_same_ns =
            NamespaceStore::move_account(&root_store, "default", "router", "default").unwrap_err();
        assert!(err_same_ns.to_string().contains("identical"));

        // Error: target already exists in target namespace
        default_store
            .set_secret("jira", "SEC_DEFAULT_JIRA")
            .unwrap();
        let err_dup =
            NamespaceStore::move_account(&root_store, "default", "jira", "work").unwrap_err();
        assert!(err_dup.to_string().contains("already exists"));
        default_store.delete_account("jira").unwrap();

        // Success: move router from default to work
        NamespaceStore::move_account(&root_store, "default", "router", "work").unwrap();
        assert!(default_store.get_secret("router").is_err());
        assert_eq!(work_store.get_secret("router").unwrap(), "SEC_ROUTER");

        // Success: move router from work to home
        NamespaceStore::move_account(&root_store, "work", "router", "home").unwrap();
        let home_store = NamespaceStore::new(&root_store, "home");
        assert!(work_store.get_secret("router").is_err());
        assert_eq!(home_store.get_secret("router").unwrap(), "SEC_ROUTER");

        // Success: move back from home to default
        NamespaceStore::move_account(&root_store, "home", "router", "default").unwrap();
        assert!(home_store.get_secret("router").is_err());
        assert_eq!(default_store.get_secret("router").unwrap(), "SEC_ROUTER");
    }
}
