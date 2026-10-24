use anyhow::{bail, Result};

#[cfg(test)]
use anyhow::Context;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

pub trait AccountStore {
    fn set_secret(&self, account: &str, secret: &str) -> Result<()>;
    fn get_secret(&self, account: &str) -> Result<String>;
    fn list_accounts(&self) -> Result<Vec<String>>;
    fn delete_account(&self, account: &str) -> Result<()>;
}

/// In-memory storage for unit testing and headless environments
#[cfg(test)]
#[derive(Default)]
pub struct MemoryStore {
    data: Mutex<HashMap<String, String>>,
}

#[cfg(test)]
impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
impl AccountStore for MemoryStore {
    fn set_secret(&self, account: &str, secret: &str) -> Result<()> {
        let mut lock = self.data.lock().unwrap();
        lock.insert(account.to_string(), secret.to_string());
        Ok(())
    }

    fn get_secret(&self, account: &str) -> Result<String> {
        let lock = self.data.lock().unwrap();
        lock.get(account)
            .cloned()
            .with_context(|| format!("Account '{account}' not found"))
    }

    fn list_accounts(&self) -> Result<Vec<String>> {
        let lock = self.data.lock().unwrap();
        let mut list: Vec<String> = lock.keys().cloned().collect();
        list.sort();
        Ok(list)
    }

    fn delete_account(&self, account: &str) -> Result<()> {
        let mut lock = self.data.lock().unwrap();
        if lock.remove(account).is_none() {
            bail!("Account '{account}' not found");
        }
        Ok(())
    }
}

/// Keyring-backed persistent secure storage
pub struct KeyringStore {
    service: String,
}

impl KeyringStore {
    pub const DEFAULT_SERVICE: &'static str = "tfa";
    const INDEX_KEY: &'static str = "__tfa_accounts_index__";

    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn get_index(&self) -> Result<Vec<String>> {
        let entry = keyring::Entry::new(&self.service, Self::INDEX_KEY)?;
        match entry.get_password() {
            Ok(json_str) => {
                let list: Vec<String> = serde_json::from_str(&json_str).unwrap_or_default();
                Ok(list)
            }
            Err(keyring::Error::NoEntry) => Ok(Vec::new()),
            Err(e) => Err(anyhow::anyhow!("Failed reading account index: {e}")),
        }
    }

    fn save_index(&self, accounts: &[String]) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, Self::INDEX_KEY)?;
        let json_str = serde_json::to_string(accounts)?;
        entry
            .set_password(&json_str)
            .map_err(|e| anyhow::anyhow!("Failed saving account index: {e}"))
    }
}

impl Default for KeyringStore {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SERVICE)
    }
}

impl AccountStore for KeyringStore {
    fn set_secret(&self, account: &str, secret: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, account)?;
        entry
            .set_password(secret)
            .map_err(|e| anyhow::anyhow!("Failed saving secret in keyring: {e}"))?;

        let mut index = self.get_index()?;
        if !index.contains(&account.to_string()) {
            index.push(account.to_string());
            index.sort();
            self.save_index(&index)?;
        }
        Ok(())
    }

    fn get_secret(&self, account: &str) -> Result<String> {
        let entry = keyring::Entry::new(&self.service, account)?;
        match entry.get_password() {
            Ok(secret) => Ok(secret),
            Err(keyring::Error::NoEntry) => bail!("Account '{account}' not found in keyring"),
            Err(e) => Err(anyhow::anyhow!("Failed reading secret from keyring: {e}")),
        }
    }

    fn list_accounts(&self) -> Result<Vec<String>> {
        self.get_index()
    }

    fn delete_account(&self, account: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, account)?;
        let _ = entry.delete_credential();

        let mut index = self.get_index()?;
        if let Some(pos) = index.iter().position(|a| a == account) {
            index.remove(pos);
            self.save_index(&index)?;
            Ok(())
        } else {
            bail!("Account '{account}' not found in index")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_lifecycle() {
        let store = MemoryStore::new();

        // 1. Initial list empty
        assert_eq!(store.list_accounts().unwrap(), Vec::<String>::new());

        // 2. Add accounts
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        store.set_secret("aws", "BASE32SECRET3939").unwrap();

        // 3. List accounts sorted
        assert_eq!(store.list_accounts().unwrap(), vec!["aws", "github"]);

        // 4. Get secret
        assert_eq!(store.get_secret("github").unwrap(), "JBSWY3DPEHPK3PXP");

        // 5. Delete account
        store.delete_account("aws").unwrap();
        assert_eq!(store.list_accounts().unwrap(), vec!["github"]);

        // 6. Delete non-existent account fails
        assert!(store.delete_account("nonexistent").is_err());
    }

    #[test]
    fn test_keyring_store_instantiation() {
        let store = KeyringStore::new("tfa-test");
        assert_eq!(store.service, "tfa-test");
        let default_store = KeyringStore::default();
        assert_eq!(default_store.service, KeyringStore::DEFAULT_SERVICE);
    }
}
