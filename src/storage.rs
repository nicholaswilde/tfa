use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

/// Local file-backed storage (e.g. for headless sessions or missing Secret Service)
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> Result<PathBuf> {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            if !xdg.is_empty() {
                return Ok(PathBuf::from(xdg).join("tfa").join("credentials.json"));
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("tfa")
                .join("credentials.json"));
        }
        bail!("Unable to determine user data directory (HOME / XDG_DATA_HOME not set)");
    }

    fn load(&self) -> Result<HashMap<String, String>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(&self.path)?;
        let map: HashMap<String, String> = serde_json::from_str(&content).unwrap_or_default();
        Ok(map)
    }

    fn save(&self, data: &HashMap<String, String>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
            #[cfg(unix)]
            {
                let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
            }
        }
        let json = serde_json::to_string_pretty(data)?;
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        options.mode(0o600);

        let mut file = options.open(&self.path)?;
        file.write_all(json.as_bytes())?;
        #[cfg(unix)]
        {
            let _ = std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

impl AccountStore for FileStore {
    fn set_secret(&self, account: &str, secret: &str) -> Result<()> {
        let mut data = self.load()?;
        data.insert(account.to_string(), secret.to_string());
        self.save(&data)
    }

    fn get_secret(&self, account: &str) -> Result<String> {
        let data = self.load()?;
        data.get(account)
            .cloned()
            .with_context(|| format!("Account '{account}' not found in file store"))
    }

    fn list_accounts(&self) -> Result<Vec<String>> {
        let data = self.load()?;
        let mut list: Vec<String> = data.keys().cloned().collect();
        list.sort();
        Ok(list)
    }

    fn delete_account(&self, account: &str) -> Result<()> {
        let mut data = self.load()?;
        if data.remove(account).is_none() {
            bail!("Account '{account}' not found in file store");
        }
        self.save(&data)
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

/// Hybrid storage that attempts KeyringStore first and falls back to FileStore on platform errors
pub struct AutoStore {
    keyring: KeyringStore,
    file: FileStore,
}

impl AutoStore {
    pub fn new(keyring: KeyringStore, file: FileStore) -> Self {
        Self { keyring, file }
    }

    pub fn default_auto() -> Result<Self> {
        let file_path = FileStore::default_path()?;
        Ok(Self::new(
            KeyringStore::default(),
            FileStore::new(file_path),
        ))
    }
}

impl AccountStore for AutoStore {
    fn set_secret(&self, account: &str, secret: &str) -> Result<()> {
        match self.keyring.set_secret(account, secret) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("[notice] Keyring unavailable ({e}); saving to local file store");
                self.file.set_secret(account, secret)
            }
        }
    }

    fn get_secret(&self, account: &str) -> Result<String> {
        if let Ok(secret) = self.keyring.get_secret(account) {
            return Ok(secret);
        }
        self.file.get_secret(account)
    }

    fn list_accounts(&self) -> Result<Vec<String>> {
        let mut accounts = self.keyring.list_accounts().unwrap_or_default();
        let file_accounts = self.file.list_accounts().unwrap_or_default();
        for acc in file_accounts {
            if !accounts.contains(&acc) {
                accounts.push(acc);
            }
        }
        accounts.sort();
        Ok(accounts)
    }

    fn delete_account(&self, account: &str) -> Result<()> {
        let k_res = self.keyring.delete_account(account);
        let f_res = self.file.delete_account(account);
        if k_res.is_ok() || f_res.is_ok() {
            Ok(())
        } else {
            bail!("Account '{account}' not found in keyring or file storage")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
    fn test_file_store_lifecycle() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sub").join("creds.json");
        let store = FileStore::new(file_path.clone());

        assert_eq!(store.list_accounts().unwrap(), Vec::<String>::new());
        store.set_secret("alice", "SECRET_A").unwrap();
        store.set_secret("bob", "SECRET_B").unwrap();

        assert_eq!(store.list_accounts().unwrap(), vec!["alice", "bob"]);
        assert_eq!(store.get_secret("alice").unwrap(), "SECRET_A");

        #[cfg(unix)]
        {
            let metadata = std::fs::metadata(&file_path).unwrap();
            let mode = metadata.permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }

        store.delete_account("alice").unwrap();
        assert_eq!(store.list_accounts().unwrap(), vec!["bob"]);
        assert!(store.delete_account("alice").is_err());
    }

    #[test]
    fn test_keyring_store_instantiation() {
        let store = KeyringStore::new("tfa-test");
        assert_eq!(store.service, "tfa-test");
        let default_store = KeyringStore::default();
        assert_eq!(default_store.service, KeyringStore::DEFAULT_SERVICE);
    }
}
