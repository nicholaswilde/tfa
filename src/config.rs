use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::clipboard::ClipboardProvider;
use crate::StorageBackend;

/// Format of configuration files supported by `tfa`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

impl ConfigFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "toml" => Some(ConfigFormat::Toml),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "json" => Some(ConfigFormat::Json),
            _ => None,
        }
    }
}

/// Representation of values loaded from a configuration file.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigFile {
    pub storage: Option<StorageBackend>,
    pub clipboard: Option<ClipboardProvider>,
    #[serde(alias = "clipboard_command")]
    pub clip_command: Option<String>,
    pub default_namespace: Option<String>,
    pub digits: Option<u8>,
    #[serde(alias = "step")]
    pub period: Option<u64>,
    pub algorithm: Option<String>,
    pub auto_copy: Option<bool>,
    pub light: Option<bool>,
}

impl ConfigFile {
    /// Parse configuration file content given a format.
    pub fn parse(content: &str, format: ConfigFormat) -> Result<Self> {
        match format {
            ConfigFormat::Toml => {
                toml::from_str(content).context("Failed to parse TOML configuration")
            }
            ConfigFormat::Yaml => {
                serde_yaml_ng::from_str(content).context("Failed to parse YAML configuration")
            }
            ConfigFormat::Json => {
                serde_json::from_str(content).context("Failed to parse JSON configuration")
            }
        }
    }

    /// Load and parse a configuration file from a specified path.
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file '{}'", path.display()))?;

        let format = ConfigFormat::from_path(path).ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported configuration format for '{}'. Expected .toml, .yaml, .yml, or .json",
                path.display()
            )
        })?;

        Self::parse(&content, format)
    }

    /// Discover and load the default configuration file if present at standard paths.
    pub fn load_default() -> Result<Option<(PathBuf, Self)>> {
        for path in default_config_paths() {
            if path.is_file() {
                let cfg = Self::load_from_path(&path)?;
                return Ok(Some((path, cfg)));
            }
        }
        Ok(None)
    }

    /// Save configuration file back to disk in its corresponding format.
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let format = ConfigFormat::from_path(path).ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported configuration format for '{}'. Expected .toml, .yaml, .yml, or .json",
                path.display()
            )
        })?;

        let text = match format {
            ConfigFormat::Toml => {
                toml::to_string_pretty(self).context("Failed serializing TOML")?
            }
            ConfigFormat::Yaml => {
                serde_yaml_ng::to_string(self).context("Failed serializing YAML")?
            }
            ConfigFormat::Json => {
                serde_json::to_string_pretty(self).context("Failed serializing JSON")?
            }
        };

        fs::write(path, text)?;
        Ok(())
    }

    /// Set default namespace in the configuration file and persist it.
    pub fn set_default_namespace_in_file(
        path: Option<&Path>,
        new_default: &str,
    ) -> Result<PathBuf> {
        let (target_path, mut config) = if let Some(p) = path {
            let cfg = if p.is_file() {
                Self::load_from_path(p)?
            } else {
                Self::default()
            };
            (p.to_path_buf(), cfg)
        } else if let Some((p, cfg)) = Self::load_default()? {
            (p, cfg)
        } else {
            let default_paths = default_config_paths();
            let first = default_paths.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!("Unable to determine default config file location")
            })?;
            (first, Self::default())
        };

        config.default_namespace = Some(new_default.to_string());
        config.save_to_path(&target_path)?;
        Ok(target_path)
    }
}

/// Standard candidate locations for default configuration files.
pub fn default_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Check XDG_CONFIG_HOME or HOME/.config
    let config_dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            Some(PathBuf::from(xdg).join("tfa"))
        } else {
            None
        }
    } else {
        None
    };

    let config_dir = config_dir.or_else(|| {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config").join("tfa"))
    });

    if let Some(dir) = config_dir {
        paths.push(dir.join("config.toml"));
        paths.push(dir.join("config.yaml"));
        paths.push(dir.join("config.yml"));
        paths.push(dir.join("config.json"));
    }

    paths
}

/// Fully resolved configuration values after evaluating the precedence chain:
/// 1. Command Line Arguments
/// 2. Environment Variables
/// 3. Configuration File
/// 4. Hardcoded Defaults
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub storage: StorageBackend,
    pub clipboard: ClipboardProvider,
    pub clip_command: Option<String>,
    pub namespace: String,
    pub digits: Option<u8>,
    pub period: Option<u64>,
    pub algorithm: Option<totp_rs::Algorithm>,
    pub auto_copy: bool,
    pub light: bool,
}

impl Default for ResolvedConfig {
    fn default() -> Self {
        Self {
            storage: StorageBackend::Auto,
            clipboard: ClipboardProvider::System,
            clip_command: None,
            namespace: "default".to_string(),
            digits: None,
            period: None,
            algorithm: None,
            auto_copy: false,
            light: false,
        }
    }
}

pub fn parse_algorithm_str(s: &str) -> Result<totp_rs::Algorithm> {
    match s.trim().to_ascii_lowercase().as_str() {
        "sha1" => Ok(totp_rs::Algorithm::SHA1),
        "sha256" => Ok(totp_rs::Algorithm::SHA256),
        "sha512" => Ok(totp_rs::Algorithm::SHA512),
        "steam" => Ok(totp_rs::Algorithm::Steam),
        other => anyhow::bail!(
            "Unsupported algorithm '{other}'. Expected sha1, sha256, sha512, or steam"
        ),
    }
}

/// Resolves configuration taking into account:
/// 1. Explicit CLI arguments (if passed)
/// 2. Environment variables (`TFA_STORAGE`, `TFA_CLIPBOARD`, `TFA_CLIP_COMMAND`, `TFA_NAMESPACE`, `TFA_DIGITS`, `TFA_PERIOD`, `TFA_ALGORITHM`, `TFA_AUTO_COPY`, `TFA_LIGHT`)
/// 3. Configuration file (either explicit path via `--config` / `TFA_CONFIG`, or discovered default in `~/.config/tfa/config.*`)
/// 4. Hardcoded defaults
pub fn resolve_configuration(
    cli_config_path: Option<&Path>,
    cli_storage: Option<StorageBackend>,
    cli_clipboard: Option<ClipboardProvider>,
    cli_clip_command: Option<String>,
    cli_namespace: Option<String>,
) -> Result<(ResolvedConfig, Option<ConfigFile>)> {
    // Determine config file path: explicit CLI / TFA_CONFIG or default discovery
    let config_file = if let Some(path) = cli_config_path {
        Some(ConfigFile::load_from_path(path)?)
    } else if let Ok(env_path) = std::env::var("TFA_CONFIG") {
        if !env_path.trim().is_empty() {
            Some(ConfigFile::load_from_path(Path::new(&env_path))?)
        } else {
            ConfigFile::load_default()?.map(|(_, cfg)| cfg)
        }
    } else {
        ConfigFile::load_default()?.map(|(_, cfg)| cfg)
    };

    // 1. Storage precedence: CLI > Env > Config File > Default (Auto)
    let storage = if let Some(s) = cli_storage {
        s
    } else if let Ok(s_env) = std::env::var("TFA_STORAGE") {
        match s_env.to_lowercase().as_str() {
            "auto" => StorageBackend::Auto,
            "keyring" => StorageBackend::Keyring,
            "file" => StorageBackend::File,
            other => anyhow::bail!("Invalid storage backend '{other}' from TFA_STORAGE"),
        }
    } else if let Some(s) = config_file.as_ref().and_then(|c| c.storage) {
        s
    } else {
        StorageBackend::Auto
    };

    // 2. Clipboard precedence: CLI > Env > Config File > Default (System)
    let clipboard = if let Some(cp) = cli_clipboard {
        cp
    } else if let Ok(cp_env) = std::env::var("TFA_CLIPBOARD") {
        match cp_env.to_lowercase().as_str() {
            "system" => ClipboardProvider::System,
            "osc52" => ClipboardProvider::Osc52,
            other => anyhow::bail!("Invalid clipboard provider '{other}' from TFA_CLIPBOARD. Expected 'system' or 'osc52'"),
        }
    } else if let Some(cp) = config_file.as_ref().and_then(|c| c.clipboard) {
        cp
    } else {
        ClipboardProvider::System
    };

    // 3. Custom clip_command precedence: CLI > Env > Config File
    let clip_command = if let Some(cmd) = cli_clip_command {
        Some(cmd)
    } else if let Ok(cmd_env) = std::env::var("TFA_CLIP_COMMAND") {
        if !cmd_env.trim().is_empty() {
            Some(cmd_env)
        } else {
            None
        }
    } else {
        config_file.as_ref().and_then(|c| c.clip_command.clone())
    };

    // 4. Namespace precedence: CLI > Env > Config File (default_namespace) > Default ("default")
    let namespace = if let Some(ns) = cli_namespace {
        ns.trim().to_lowercase()
    } else if let Ok(ns_env) = std::env::var("TFA_NAMESPACE") {
        if !ns_env.trim().is_empty() {
            ns_env.trim().to_lowercase()
        } else {
            "default".to_string()
        }
    } else if let Some(ns_cfg) = config_file
        .as_ref()
        .and_then(|c| c.default_namespace.as_ref())
    {
        ns_cfg.trim().to_lowercase()
    } else {
        "default".to_string()
    };

    // 2. Digits precedence: Env > Config File
    let digits = if let Ok(d_env) = std::env::var("TFA_DIGITS") {
        Some(d_env.parse::<u8>().context("Invalid TFA_DIGITS value")?)
    } else {
        config_file.as_ref().and_then(|c| c.digits)
    };

    // 3. Period precedence: Env > Config File
    let period = if let Ok(p_env) = std::env::var("TFA_PERIOD") {
        Some(p_env.parse::<u64>().context("Invalid TFA_PERIOD value")?)
    } else if let Ok(s_env) = std::env::var("TFA_STEP") {
        Some(s_env.parse::<u64>().context("Invalid TFA_STEP value")?)
    } else {
        config_file.as_ref().and_then(|c| c.period)
    };

    // 4. Algorithm precedence: Env > Config File
    let algorithm = if let Ok(a_env) = std::env::var("TFA_ALGORITHM") {
        Some(parse_algorithm_str(&a_env)?)
    } else if let Some(a_cfg) = config_file.as_ref().and_then(|c| c.algorithm.as_ref()) {
        Some(parse_algorithm_str(a_cfg)?)
    } else {
        None
    };

    // 5. Auto-copy precedence: Env > Config File > Default (false)
    let auto_copy = if let Ok(ac_env) = std::env::var("TFA_AUTO_COPY") {
        match ac_env.to_lowercase().as_str() {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            other => anyhow::bail!("Invalid boolean for TFA_AUTO_COPY: '{other}'"),
        }
    } else {
        config_file
            .as_ref()
            .and_then(|c| c.auto_copy)
            .unwrap_or_default()
    };

    // 6. Light precedence: Env > Config File > Default (false)
    let light = if let Ok(l_env) = std::env::var("TFA_LIGHT") {
        match l_env.to_lowercase().as_str() {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            other => anyhow::bail!("Invalid boolean for TFA_LIGHT: '{other}'"),
        }
    } else {
        config_file
            .as_ref()
            .and_then(|c| c.light)
            .unwrap_or_default()
    };

    Ok((
        ResolvedConfig {
            storage,
            clipboard,
            clip_command,
            namespace,
            digits,
            period,
            algorithm,
            auto_copy,
            light,
        },
        config_file,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_format_detection() {
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.toml")),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yaml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.json")),
            Some(ConfigFormat::Json)
        );
        assert_eq!(ConfigFormat::from_path(Path::new("config.ini")), None);
    }

    #[test]
    fn test_parse_toml_config() {
        let content = r#"
storage = "file"
digits = 8
period = 60
algorithm = "sha256"
auto_copy = true
light = true
"#;
        let cfg = ConfigFile::parse(content, ConfigFormat::Toml).unwrap();
        assert_eq!(cfg.storage, Some(StorageBackend::File));
        assert_eq!(cfg.digits, Some(8));
        assert_eq!(cfg.period, Some(60));
        assert_eq!(cfg.algorithm.as_deref(), Some("sha256"));
        assert_eq!(cfg.auto_copy, Some(true));
        assert_eq!(cfg.light, Some(true));
    }

    #[test]
    fn test_parse_yaml_config() {
        let content = r#"
storage: keyring
digits: 6
step: 30
algorithm: steam
auto_copy: false
light: false
"#;
        let cfg = ConfigFile::parse(content, ConfigFormat::Yaml).unwrap();
        assert_eq!(cfg.storage, Some(StorageBackend::Keyring));
        assert_eq!(cfg.digits, Some(6));
        assert_eq!(cfg.period, Some(30)); // aliased from step
        assert_eq!(cfg.algorithm.as_deref(), Some("steam"));
        assert_eq!(cfg.auto_copy, Some(false));
        assert_eq!(cfg.light, Some(false));
    }

    #[test]
    fn test_parse_json_config() {
        let content = r#"{
    "storage": "auto",
    "digits": 7,
    "period": 45,
    "algorithm": "sha512",
    "auto_copy": true,
    "light": false
}"#;
        let cfg = ConfigFile::parse(content, ConfigFormat::Json).unwrap();
        assert_eq!(cfg.storage, Some(StorageBackend::Auto));
        assert_eq!(cfg.digits, Some(7));
        assert_eq!(cfg.period, Some(45));
        assert_eq!(cfg.algorithm.as_deref(), Some("sha512"));
        assert_eq!(cfg.auto_copy, Some(true));
        assert_eq!(cfg.light, Some(false));
    }

    #[test]
    fn test_resolve_precedence_cli_over_env_over_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_file_path = dir.path().join("config.toml");
        fs::write(
            &config_file_path,
            r#"
storage = "keyring"
clip_command = "wl-copy"
default_namespace = "work"
digits = 8
period = 60
auto_copy = false
"#,
        )
        .unwrap();

        // 1. File loaded only
        let (resolved, _) =
            resolve_configuration(Some(&config_file_path), None, None, None, None).unwrap();
        assert_eq!(resolved.storage, StorageBackend::Keyring);
        assert_eq!(resolved.clip_command.as_deref(), Some("wl-copy"));
        assert_eq!(resolved.namespace, "work");
        assert_eq!(resolved.digits, Some(8));
        assert_eq!(resolved.period, Some(60));
        assert!(!resolved.auto_copy);

        // 2. CLI overrides file
        let (resolved_cli, _) = resolve_configuration(
            Some(&config_file_path),
            Some(StorageBackend::File),
            None,
            Some("xclip".to_string()),
            Some("personal".to_string()),
        )
        .unwrap();
        assert_eq!(resolved_cli.storage, StorageBackend::File);
        assert_eq!(resolved_cli.clip_command.as_deref(), Some("xclip"));
        assert_eq!(resolved_cli.namespace, "personal");
    }

    #[test]
    fn test_hierarchical_precedence_full_chain() {
        // Test hierarchy: CLI > ENV > FILE > DEFAULT
        // We will isolate by temporary environment variables
        let dir = tempfile::tempdir().unwrap();
        let config_file_path = dir.path().join("config.json");
        fs::write(
            &config_file_path,
            r#"{
                "storage": "file",
                "clipboard": "system",
                "clip_command": "xsel",
                "default_namespace": "home",
                "digits": 6,
                "period": 30,
                "algorithm": "sha1",
                "auto_copy": false,
                "light": false
            }"#,
        )
        .unwrap();

        // Level 3: Config file only (no env, no CLI)
        let (res_file, _) =
            resolve_configuration(Some(&config_file_path), None, None, None, None).unwrap();
        assert_eq!(res_file.storage, StorageBackend::File);
        assert_eq!(res_file.clipboard, ClipboardProvider::System);
        assert_eq!(res_file.clip_command.as_deref(), Some("xsel"));
        assert_eq!(res_file.namespace, "home");
        assert_eq!(res_file.digits, Some(6));
        assert_eq!(res_file.period, Some(30));
        assert_eq!(res_file.algorithm, Some(totp_rs::Algorithm::SHA1));
        assert!(!res_file.auto_copy);
        assert!(!res_file.light);

        // Level 2: Environment variables override Config file
        std::env::set_var("TFA_STORAGE", "keyring");
        std::env::set_var("TFA_CLIPBOARD", "osc52");
        std::env::set_var("TFA_CLIP_COMMAND", "pbcopy");
        std::env::set_var("TFA_NAMESPACE", "ops");
        std::env::set_var("TFA_DIGITS", "8");
        std::env::set_var("TFA_PERIOD", "60");
        std::env::set_var("TFA_ALGORITHM", "sha256");
        std::env::set_var("TFA_AUTO_COPY", "true");
        std::env::set_var("TFA_LIGHT", "true");

        let (res_env, _) =
            resolve_configuration(Some(&config_file_path), None, None, None, None).unwrap();
        assert_eq!(res_env.storage, StorageBackend::Keyring);
        assert_eq!(res_env.clipboard, ClipboardProvider::Osc52);
        assert_eq!(res_env.clip_command.as_deref(), Some("pbcopy"));
        assert_eq!(res_env.namespace, "ops");
        assert_eq!(res_env.digits, Some(8));
        assert_eq!(res_env.period, Some(60));
        assert_eq!(res_env.algorithm, Some(totp_rs::Algorithm::SHA256));
        assert!(res_env.auto_copy);
        assert!(res_env.light);

        // Level 1: CLI arguments override Environment variables
        let (res_cli, _) = resolve_configuration(
            Some(&config_file_path),
            Some(StorageBackend::File),
            Some(ClipboardProvider::System),
            Some("custom-pipe".to_string()),
            Some("dev".to_string()),
        )
        .unwrap();
        assert_eq!(res_cli.storage, StorageBackend::File);
        assert_eq!(res_cli.clipboard, ClipboardProvider::System);
        assert_eq!(res_cli.clip_command.as_deref(), Some("custom-pipe"));
        assert_eq!(res_cli.namespace, "dev");

        // Clean up environment variables
        std::env::remove_var("TFA_STORAGE");
        std::env::remove_var("TFA_CLIPBOARD");
        std::env::remove_var("TFA_CLIP_COMMAND");
        std::env::remove_var("TFA_NAMESPACE");
        std::env::remove_var("TFA_DIGITS");
        std::env::remove_var("TFA_PERIOD");
        std::env::remove_var("TFA_ALGORITHM");
        std::env::remove_var("TFA_AUTO_COPY");
        std::env::remove_var("TFA_LIGHT");
    }

    #[test]
    fn test_set_default_namespace_persists_to_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_file_path = dir.path().join("config.toml");

        let saved_path =
            ConfigFile::set_default_namespace_in_file(Some(&config_file_path), "production")
                .unwrap();
        assert_eq!(saved_path, config_file_path);

        let reloaded = ConfigFile::load_from_path(&config_file_path).unwrap();
        assert_eq!(reloaded.default_namespace.as_deref(), Some("production"));
    }
}
