use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
#[cfg(test)]
use std::sync::Mutex;

use age::secrecy::SecretString;
use age::{Identity, Recipient};

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
            if !home.is_empty() {
                return Ok(PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("tfa")
                    .join("credentials.json"));
            }
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.is_empty() {
                return Ok(PathBuf::from(local).join("tfa").join("credentials.json"));
            }
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.is_empty() {
                return Ok(PathBuf::from(appdata).join("tfa").join("credentials.json"));
            }
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            if !userprofile.is_empty() {
                return Ok(PathBuf::from(userprofile)
                    .join(".local")
                    .join("share")
                    .join("tfa")
                    .join("credentials.json"));
            }
        }
        bail!("Unable to determine user data directory (HOME / XDG_DATA_HOME / LOCALAPPDATA / USERPROFILE not set)");
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

fn expand_path(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if s == "~" {
        if let Some(h) = home_dirs().into_iter().next() {
            return h;
        }
    } else if let Some(stripped) = s.strip_prefix("~/").or_else(|| s.strip_prefix("~\\")) {
        if let Some(h) = home_dirs().into_iter().next() {
            return h.join(stripped);
        }
    }
    p.to_path_buf()
}

fn home_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            dirs.push(PathBuf::from(home.trim()));
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        if !userprofile.trim().is_empty() {
            let p = PathBuf::from(userprofile.trim());
            if !dirs.contains(&p) {
                dirs.push(p);
            }
        }
    }
    dirs
}

fn add_home_candidates<F>(dir: &Path, add_candidate: &mut F)
where
    F: FnMut(PathBuf),
{
    add_candidate(dir.join(".config").join("age").join("keys.txt"));
    add_candidate(
        dir.join(".config")
            .join("sops")
            .join("age")
            .join("keys.txt"),
    );
    add_candidate(dir.join(".ssh").join("id_ed25519"));
    add_candidate(dir.join(".ssh").join("id_rsa"));

    let ssh_dir = dir.join(".ssh");
    if let Ok(entries) = std::fs::read_dir(&ssh_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let filename = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if filename.ends_with(".pub") {
                    let priv_p = p.with_extension("");
                    if priv_p.is_file() {
                        add_candidate(priv_p);
                    }
                } else if !filename.starts_with("known_hosts")
                    && !filename.starts_with("config")
                    && !filename.starts_with("authorized_keys")
                    && !filename.starts_with('.')
                {
                    add_candidate(p);
                }
            }
        }
    }
}

/// Age-encrypted file storage backend
pub struct AgeStore {
    path: PathBuf,
    key_file: Option<PathBuf>,
    ssh_key_file: Option<PathBuf>,
}

impl AgeStore {
    pub fn new(path: PathBuf, key_file: Option<PathBuf>, ssh_key_file: Option<PathBuf>) -> Self {
        Self {
            path,
            key_file: key_file.map(|p| expand_path(&p)),
            ssh_key_file: ssh_key_file.map(|p| expand_path(&p)),
        }
    }

    /// Default storage location for age-encrypted credentials (`~/.local/share/tfa/credentials.age`)
    pub fn default_path() -> Result<PathBuf> {
        let mut p = FileStore::default_path()?;
        p.set_file_name("credentials.age");
        Ok(p)
    }

    /// Candidate file paths to search for age identity keys
    pub fn candidate_key_paths() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        let mut add_candidate = |p: PathBuf| {
            if !candidates.contains(&p) {
                candidates.push(p);
            }
        };

        if let Ok(path) = std::env::var("TFA_AGE_KEY_FILE") {
            if !path.trim().is_empty() {
                add_candidate(expand_path(&PathBuf::from(path.trim())));
            }
        }
        if let Ok(path) = std::env::var("AGE_KEY_FILE") {
            if !path.trim().is_empty() {
                add_candidate(expand_path(&PathBuf::from(path.trim())));
            }
        }
        if let Ok(path) = std::env::var("SOPS_AGE_KEY_FILE") {
            if !path.trim().is_empty() {
                add_candidate(expand_path(&PathBuf::from(path.trim())));
            }
        }
        if let Ok(path) = std::env::var("TFA_SSH_KEY_FILE") {
            if !path.trim().is_empty() {
                add_candidate(expand_path(&PathBuf::from(path.trim())));
            }
        }
        if let Ok(path) = std::env::var("TFA_SSH_KEY") {
            if !path.trim().is_empty() {
                add_candidate(expand_path(&PathBuf::from(path.trim())));
            }
        }
        if let Ok(path) = std::env::var("TFA_AGE_SSH_KEY") {
            if !path.trim().is_empty() {
                add_candidate(expand_path(&PathBuf::from(path.trim())));
            }
        }

        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            if !config_home.trim().is_empty() {
                let c = PathBuf::from(config_home.trim());
                add_candidate(c.join("age").join("keys.txt"));
                add_candidate(c.join("sops").join("age").join("keys.txt"));
            }
        }

        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                let a = PathBuf::from(appdata.trim());
                add_candidate(a.join("age").join("keys.txt"));
                add_candidate(a.join("sops").join("age").join("keys.txt"));
            }
        }

        for h in home_dirs() {
            add_home_candidates(&h, &mut add_candidate);
        }

        candidates
    }

    /// Load age identities from raw string, files, or environment
    pub fn resolve_identities(&self) -> Result<Vec<Box<dyn Identity>>> {
        let mut identities: Vec<Box<dyn Identity>> = Vec::new();

        // 1. Direct key content in TFA_AGE_KEY environment variable
        if let Ok(raw_key) = std::env::var("TFA_AGE_KEY") {
            let trimmed = raw_key.trim();
            if !trimmed.is_empty() {
                if let Ok(x_id) = age::x25519::Identity::from_str(trimmed) {
                    identities.push(Box::new(x_id));
                    return Ok(identities);
                }
                if let Ok(parsed) = age::IdentityFile::from_buffer(std::io::Cursor::new(trimmed)) {
                    if let Ok(ids) = parsed.into_identities() {
                        if !ids.is_empty() {
                            for id in ids {
                                identities.push(id);
                            }
                            return Ok(identities);
                        }
                    }
                }
            }
        }

        // 2. Explicit age key file configured
        if let Some(ref path) = self.key_file {
            if path.exists() {
                if let Ok(loaded) = Self::load_identities_from_path(path) {
                    if !loaded.is_empty() {
                        identities.extend(loaded);
                        return Ok(identities);
                    }
                }
            } else {
                bail!(
                    "Configured age key file '{}' does not exist",
                    path.display()
                );
            }
        }

        // 3. Explicit SSH key file configured
        if let Some(ref path) = self.ssh_key_file {
            if path.exists() {
                if let Ok(loaded) = Self::load_identities_from_path(path) {
                    if !loaded.is_empty() {
                        identities.extend(loaded);
                        return Ok(identities);
                    }
                }
                bail!(
                    "Failed to load SSH identity from configured key file '{}'",
                    path.display()
                );
            } else {
                bail!(
                    "Configured SSH key file '{}' does not exist",
                    path.display()
                );
            }
        }

        // 4. Fallback candidate paths
        for candidate in Self::candidate_key_paths() {
            if candidate.is_file() {
                if let Ok(loaded) = Self::load_identities_from_path(&candidate) {
                    if !loaded.is_empty() {
                        identities.extend(loaded);
                        return Ok(identities);
                    }
                }
            }
        }

        // 5. Fallback passphrase from TFA_AGE_PASSPHRASE
        if let Ok(pass) = std::env::var("TFA_AGE_PASSPHRASE") {
            if !pass.is_empty() {
                identities.push(Box::new(age::scrypt::Identity::new(SecretString::from(
                    pass,
                ))));
                return Ok(identities);
            }
        }

        bail!(
            "No age or SSH keys found. Checked candidate locations including ~/.config/sops/age/keys.txt, ~/.config/age/keys.txt, ~/.ssh/id_ed25519, and TFA_AGE_KEY/TFA_AGE_KEY_FILE/TFA_SSH_KEY_FILE environment variables."
        )
    }

    /// Load identities from a single file path (age key file or SSH private key)
    fn load_identities_from_path(path: &Path) -> Result<Vec<Box<dyn Identity>>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read key file '{}'", path.display()))?;

        let mut list: Vec<Box<dyn Identity>> = Vec::new();

        // Try standard age identity file format (native x25519 keys)
        if let Ok(parsed) = age::IdentityFile::from_buffer(std::io::Cursor::new(&content)) {
            if let Ok(ids) = parsed.into_identities() {
                for id in ids {
                    list.push(id);
                }
            }
        }

        // Try OpenSSH private key format
        if list.is_empty() {
            if let Ok(ssh_id) =
                age::ssh::Identity::from_buffer(std::io::Cursor::new(&content), None)
            {
                list.push(Box::new(ssh_id));
            }
        }

        Ok(list)
    }

    /// Resolve recipients for encrypting the credentials store
    pub fn resolve_recipients(&self) -> Result<Vec<Box<dyn Recipient + Send>>> {
        let mut recipients: Vec<Box<dyn Recipient + Send>> = Vec::new();

        // 1. Explicit recipient string or SSH pubkey in TFA_AGE_RECIPIENTS
        if let Ok(recs) = std::env::var("TFA_AGE_RECIPIENTS") {
            for token in recs.split([',', '\n']) {
                let token = token.trim();
                if token.is_empty() {
                    continue;
                }
                if let Ok(x_rec) = age::x25519::Recipient::from_str(token) {
                    recipients.push(Box::new(x_rec));
                } else if let Ok(ssh_rec) = age::ssh::Recipient::from_str(token) {
                    recipients.push(Box::new(ssh_rec));
                }
            }
            if !recipients.is_empty() {
                return Ok(recipients);
            }
        }

        // 2. Derive recipient from explicit age key_file if specified
        if let Some(ref path) = self.key_file {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(stripped) = trimmed.strip_prefix("# public key:") {
                            if let Ok(x_rec) = age::x25519::Recipient::from_str(stripped.trim()) {
                                recipients.push(Box::new(x_rec));
                                return Ok(recipients);
                            }
                        } else if trimmed.starts_with("AGE-SECRET-KEY-1") {
                            if let Ok(id) = age::x25519::Identity::from_str(trimmed) {
                                recipients.push(Box::new(id.to_public()));
                                return Ok(recipients);
                            }
                        }
                    }
                }
            } else {
                bail!(
                    "Configured age key file '{}' does not exist",
                    path.display()
                );
            }
        }

        // 3. Derive recipient from explicit ssh_key_file if specified
        if let Some(ref path) = self.ssh_key_file {
            if path.exists() {
                let pub_candidates = [
                    path.clone(),
                    PathBuf::from(format!("{}.pub", path.display())),
                    path.with_extension("pub"),
                ];
                for pub_p in &pub_candidates {
                    if pub_p.is_file() {
                        if let Ok(content) = std::fs::read_to_string(pub_p) {
                            for line in content.lines() {
                                let line = line.trim();
                                if let Ok(ssh_rec) = age::ssh::Recipient::from_str(line) {
                                    recipients.push(Box::new(ssh_rec));
                                    return Ok(recipients);
                                }
                            }
                        }
                    }
                }

                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(ssh_id) =
                        age::ssh::Identity::from_buffer(std::io::Cursor::new(&content), None)
                    {
                        if let Ok(rec) = age::ssh::Recipient::try_from(ssh_id) {
                            recipients.push(Box::new(rec));
                            return Ok(recipients);
                        }
                    }
                }

                bail!(
                    "Failed to derive recipient from configured SSH key file '{}'",
                    path.display()
                );
            } else {
                bail!(
                    "Configured SSH key file '{}' does not exist",
                    path.display()
                );
            }
        }

        // 3. Derive recipient from candidate key files (sops keys, age keys, etc.)
        for candidate in Self::candidate_key_paths() {
            if candidate.is_file() {
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(stripped) = trimmed.strip_prefix("# public key:") {
                            if let Ok(x_rec) = age::x25519::Recipient::from_str(stripped.trim()) {
                                recipients.push(Box::new(x_rec));
                                return Ok(recipients);
                            }
                        } else if trimmed.starts_with("AGE-SECRET-KEY-1") {
                            if let Ok(id) = age::x25519::Identity::from_str(trimmed) {
                                recipients.push(Box::new(id.to_public()));
                                return Ok(recipients);
                            }
                        }
                    }
                }
            }
        }

        // 4. Check SSH public keys in ~/.ssh/
        for h in home_dirs() {
            for ssh_pub in &[
                h.join(".ssh").join("id_ed25519.pub"),
                h.join(".ssh").join("id_rsa.pub"),
            ] {
                if ssh_pub.is_file() {
                    if let Ok(content) = std::fs::read_to_string(ssh_pub) {
                        if let Ok(ssh_rec) = age::ssh::Recipient::from_str(content.trim()) {
                            recipients.push(Box::new(ssh_rec));
                            return Ok(recipients);
                        }
                    }
                }
            }
        }

        // 5. Passphrase recipient if TFA_AGE_PASSPHRASE is provided
        if let Ok(pass) = std::env::var("TFA_AGE_PASSPHRASE") {
            if !pass.is_empty() {
                recipients.push(Box::new(age::scrypt::Recipient::new(SecretString::from(
                    pass,
                ))));
                return Ok(recipients);
            }
        }

        bail!(
            "No age recipients available for encryption. Set TFA_AGE_RECIPIENTS, ensure ~/.ssh/id_ed25519.pub exists, or configure a valid age key file."
        )
    }

    fn load(&self) -> Result<HashMap<String, String>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }

        let encrypted_bytes = std::fs::read(&self.path)
            .with_context(|| format!("Failed to read age store file '{}'", self.path.display()))?;

        if encrypted_bytes.is_empty() {
            return Ok(HashMap::new());
        }

        let identities = self.resolve_identities()?;
        let decryptor = age::Decryptor::new_buffered(&encrypted_bytes[..])
            .context("Failed to parse age encrypted storage header")?;

        let id_refs: Vec<&dyn Identity> = identities.iter().map(|id| &**id).collect();
        let mut reader = decryptor
            .decrypt(id_refs.into_iter())
            .context("Failed to decrypt age store with available identities")?;

        let mut decrypted_bytes = Vec::new();
        reader.read_to_end(&mut decrypted_bytes)?;

        let map: HashMap<String, String> =
            serde_json::from_slice(&decrypted_bytes).unwrap_or_default();
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

        let recipients = self.resolve_recipients()?;
        let rec_refs: Vec<&dyn Recipient> =
            recipients.iter().map(|r| &**r as &dyn Recipient).collect();
        let encryptor = age::Encryptor::with_recipients(rec_refs.into_iter())
            .context("Failed to create age encryptor with resolved recipients")?;

        let json = serde_json::to_string_pretty(data)?;
        let mut encrypted_output = Vec::new();
        let mut writer = encryptor
            .wrap_output(&mut encrypted_output)
            .context("Failed to wrap age encryption output")?;
        writer.write_all(json.as_bytes())?;
        writer.finish()?;

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        options.mode(0o600);

        let mut file = options.open(&self.path)?;
        file.write_all(&encrypted_output)?;
        #[cfg(unix)]
        {
            let _ = std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }
}

impl AccountStore for AgeStore {
    fn set_secret(&self, account: &str, secret: &str) -> Result<()> {
        let mut data = self.load()?;
        data.insert(account.to_string(), secret.to_string());
        self.save(&data)
    }

    fn get_secret(&self, account: &str) -> Result<String> {
        let data = self.load()?;
        data.get(account)
            .cloned()
            .with_context(|| format!("Account '{account}' not found in age store"))
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
            bail!("Account '{account}' not found in age store");
        }
        self.save(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    static TEST_STORAGE_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
    fn test_file_store_corrupted_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("creds.json");
        std::fs::write(&file_path, "not valid json").unwrap();
        let store = FileStore::new(file_path);
        let list = store.list_accounts().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_file_store_get_secret_missing() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("creds.json");
        let store = FileStore::new(file_path);
        let res = store.get_secret("missing");
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("not found in file store"));
    }

    #[test]
    fn test_file_store_default_path() {
        let res = FileStore::default_path();
        assert!(res.is_ok());
        let path = res.unwrap();
        assert_eq!(path.file_name().unwrap(), "credentials.json");
        assert_eq!(path.parent().unwrap().file_name().unwrap(), "tfa");
    }

    #[test]
    fn test_keyring_store_instantiation_and_calls() {
        let store = KeyringStore::new("tfa-test");
        assert_eq!(store.service, "tfa-test");
        let default_store = KeyringStore::default();
        assert_eq!(default_store.service, KeyringStore::DEFAULT_SERVICE);

        // In headless/test environment, keyring calls may error; verify paths execute
        let _ = store.get_secret("nonexistent");
        let _ = store.list_accounts();
        let _ = store.delete_account("nonexistent");
        let _ = store.set_secret("test_acc", "test_sec");
    }

    #[test]
    fn test_auto_store_lifecycle() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sub").join("auto_creds.json");
        let file_store = FileStore::new(file_path);
        let keyring_store = KeyringStore::new("tfa-test-auto");
        let auto = AutoStore::new(keyring_store, file_store);

        // 1. Initially empty
        assert_eq!(auto.list_accounts().unwrap(), Vec::<String>::new());

        // 2. Set secret (falls back to file if keyring unavailable)
        auto.set_secret("alice", "SECRET_A").unwrap();
        auto.set_secret("bob", "SECRET_B").unwrap();

        // 3. List accounts
        assert_eq!(auto.list_accounts().unwrap(), vec!["alice", "bob"]);

        // 4. Get secret
        assert_eq!(auto.get_secret("alice").unwrap(), "SECRET_A");
        assert_eq!(auto.get_secret("bob").unwrap(), "SECRET_B");

        // 5. Delete account
        auto.delete_account("alice").unwrap();
        assert_eq!(auto.list_accounts().unwrap(), vec!["bob"]);

        // 6. Delete nonexistent fails
        assert!(auto.delete_account("nonexistent").is_err());
    }

    #[test]
    fn test_auto_store_default_auto() {
        let auto = AutoStore::default_auto();
        assert!(auto.is_ok());
    }

    #[test]
    fn test_age_store_default_path() {
        let p = AgeStore::default_path().unwrap();
        assert_eq!(p.file_name().unwrap(), "credentials.age");
    }

    #[test]
    fn test_age_store_lifecycle_with_generated_identity() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("credentials.age");
        let key_path = dir.path().join("keys.txt");

        // Generate age key
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public();
        let secret_str = identity.to_string();
        use age::secrecy::ExposeSecret;
        let file_content = format!("# public key: {}\n{}\n", pubkey, secret_str.expose_secret());
        std::fs::write(&key_path, file_content).unwrap();

        let store = AgeStore::new(store_path.clone(), Some(key_path.clone()), None);

        // 1. Initially empty
        assert_eq!(store.list_accounts().unwrap(), Vec::<String>::new());

        // 2. Set secrets
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        store.set_secret("aws", "BASE32SECRET3939").unwrap();

        // Ensure file on disk is encrypted (starts with age-encryption header or binary)
        let disk_bytes = std::fs::read(&store_path).unwrap();
        assert!(!disk_bytes.is_empty());
        assert!(!String::from_utf8_lossy(&disk_bytes).contains("JBSWY3DPEHPK3PXP"));

        // 3. List accounts sorted
        assert_eq!(store.list_accounts().unwrap(), vec!["aws", "github"]);

        // 4. Retrieve secret
        assert_eq!(store.get_secret("github").unwrap(), "JBSWY3DPEHPK3PXP");
        assert_eq!(store.get_secret("aws").unwrap(), "BASE32SECRET3939");

        // 5. Delete account
        store.delete_account("github").unwrap();
        assert_eq!(store.list_accounts().unwrap(), vec!["aws"]);

        // 6. Delete nonexistent fails
        assert!(store.delete_account("nonexistent").is_err());
    }

    #[test]
    fn test_age_store_lifecycle_with_passphrase() {
        let _guard = TEST_STORAGE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("credentials.age");
        let store = AgeStore::new(store_path.clone(), None, None);

        std::env::set_var("TFA_AGE_PASSPHRASE", "masterpassword123");
        store.set_secret("my_service", "MY_SUPER_SECRET").unwrap();
        assert_eq!(store.get_secret("my_service").unwrap(), "MY_SUPER_SECRET");
        assert_eq!(store.list_accounts().unwrap(), vec!["my_service"]);
        std::env::remove_var("TFA_AGE_PASSPHRASE");
    }

    #[test]
    fn test_age_store_fallback_candidate_paths() {
        let _guard = TEST_STORAGE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let _temp = if std::env::var("HOME").is_err()
            && std::env::var("USERPROFILE").is_err()
            && std::env::var("APPDATA").is_err()
        {
            let temp = tempdir().unwrap();
            std::env::set_var("HOME", temp.path());
            Some(temp)
        } else {
            None
        };

        let candidates = AgeStore::candidate_key_paths();
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c
            .to_string_lossy()
            .contains(".config/sops/age/keys.txt")
            || c.to_string_lossy().contains("sops")));

        if _temp.is_some() {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn test_age_store_candidate_paths_windows_appdata() {
        let _guard = TEST_STORAGE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let appdata_dir = dir.path().join("AppData").join("Roaming");
        std::env::set_var("APPDATA", &appdata_dir);

        let candidates = AgeStore::candidate_key_paths();
        assert!(candidates
            .iter()
            .any(|c| c.starts_with(&appdata_dir) && c.to_string_lossy().contains("sops")));

        std::env::remove_var("APPDATA");
    }

    #[test]
    fn test_age_store_candidate_paths_windows_userprofile() {
        let _guard = TEST_STORAGE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let profile_dir = dir.path().join("Users").join("testuser");
        std::env::set_var("USERPROFILE", &profile_dir);

        let candidates = AgeStore::candidate_key_paths();
        assert!(candidates
            .iter()
            .any(|c| c.starts_with(&profile_dir) && c.to_string_lossy().contains("sops")));

        std::env::remove_var("USERPROFILE");
    }

    #[test]
    fn test_age_store_missing_identity_fails() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("credentials.age");
        let non_existent_key = dir.path().join("no_such_key.txt");
        let store = AgeStore::new(store_path, Some(non_existent_key), None);

        let res = store.set_secret("test", "secret");
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_age_store_with_ssh_key_location() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("credentials.age");
        let ssh_priv = dir.path().join("id_ed25519");
        let ssh_pub = dir.path().join("id_ed25519.pub");

        let priv_content = concat!(
            "-----BEGIN OPENSSH PRIVATE KEY-----\n",
            "b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\n",
            "QyNTUxOQAAACCOsP2Bak4G1KJl3Q/s764imWfrJLat13s7fO2F+TlXaQAAAJDaUiqR2lIq\n",
            "kQAAAAtzc2gtZWQyNTUxOQAAACCOsP2Bak4G1KJl3Q/s764imWfrJLat13s7fO2F+TlXaQ\n",
            "AAAED2N+OslOMTYGAbV4xHiMZ3Olx8ZRxRw2RbyUTif3QtHY6w/YFqTgbUomXdD+zvriKZ\n",
            "Z+sktq3Xezt87YX5OVdpAAAADHRlc3Qta2V5QHRmYQE=\n",
            "-----END OPENSSH PRIVATE KEY-----\n"
        );
        let pub_content = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAII6w/YFqTgbUomXdD+zvriKZZ+sktq3Xezt87YX5OVdp test-key@tfa\n";

        std::fs::write(&ssh_priv, priv_content).unwrap();
        std::fs::write(&ssh_pub, pub_content).unwrap();

        let store = AgeStore::new(store_path.clone(), None, Some(ssh_priv.clone()));

        // Set and get secret using the explicit SSH key location
        store.set_secret("ssh_service", "SSH_SECRET_987").unwrap();
        assert_eq!(store.get_secret("ssh_service").unwrap(), "SSH_SECRET_987");
        assert_eq!(store.list_accounts().unwrap(), vec!["ssh_service"]);

        // Non-existent SSH key location fails
        let missing_store = AgeStore::new(
            store_path,
            None,
            Some(dir.path().join("non_existent_id_ed25519")),
        );
        assert!(missing_store.get_secret("ssh_service").is_err());
    }

    #[test]
    fn test_age_store_with_ssh_key_location_without_pub_file() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("credentials.age");
        let ssh_priv = dir.path().join("id_ed25519_standalone");

        let priv_content = concat!(
            "-----BEGIN OPENSSH PRIVATE KEY-----\n",
            "b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\n",
            "QyNTUxOQAAACCOsP2Bak4G1KJl3Q/s764imWfrJLat13s7fO2F+TlXaQAAAJDaUiqR2lIq\n",
            "kQAAAAtzc2gtZWQyNTUxOQAAACCOsP2Bak4G1KJl3Q/s764imWfrJLat13s7fO2F+TlXaQ\n",
            "AAAED2N+OslOMTYGAbV4xHiMZ3Olx8ZRxRw2RbyUTif3QtHY6w/YFqTgbUomXdD+zvriKZ\n",
            "Z+sktq3Xezt87YX5OVdpAAAADHRlc3Qta2V5QHRmYQE=\n",
            "-----END OPENSSH PRIVATE KEY-----\n"
        );

        std::fs::write(&ssh_priv, priv_content).unwrap();

        let store = AgeStore::new(store_path, None, Some(ssh_priv));

        store
            .set_secret("ssh_service2", "SECRET_WITHOUT_PUB")
            .unwrap();
        assert_eq!(
            store.get_secret("ssh_service2").unwrap(),
            "SECRET_WITHOUT_PUB"
        );
        assert_eq!(store.list_accounts().unwrap(), vec!["ssh_service2"]);
    }
}
