use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

mod backup;
mod clipboard;
mod import;
mod storage;
mod totp;

use clipboard::{Clipboard, SystemClipboard};
use storage::{AccountStore, AutoStore, FileStore, KeyringStore};
use totp::TfaTotp;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StorageBackend {
    /// Attempt OS keyring first; fallback to local file store if unavailable
    #[default]
    Auto,
    /// Strictly use system keyring (Secret Service, Keychain, Credential Manager)
    Keyring,
    /// Use local file storage (~/.local/share/tfa/credentials.json)
    File,
}

#[derive(Parser, Debug)]
#[command(name = "tfa")]
#[command(author, version = env!("PROJECT_VERSION"), about = "CLI Two-Factor Authentication (2FA) tool", long_about = None)]
pub struct Cli {
    /// Credential storage backend
    #[arg(long, env = "TFA_STORAGE", default_value = "auto", global = true)]
    pub storage: StorageBackend,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a TOTP token for an account
    Generate {
        /// Name of the account
        account: String,
        /// Copy generated token to clipboard
        #[arg(short, long)]
        copy: bool,
    },
    /// Add a new account secret or OTPAuth URL
    Add {
        /// Name of the account
        account: String,
        /// TOTP secret key or otpauth:// URI (prompted if omitted)
        secret: Option<String>,
        /// Number of digits in generated code (e.g. 6 or 8)
        #[arg(short, long)]
        digits: Option<u8>,
        /// Time step period in seconds (e.g. 30 or 60)
        #[arg(short, long)]
        period: Option<u64>,
        /// Hash algorithm (sha1, sha256, sha512)
        #[arg(short, long)]
        algorithm: Option<totp_rs::Algorithm>,
    },
    /// List all stored accounts
    List,
    /// Delete an account secret
    Delete {
        /// Name of the account
        account: String,
    },
    /// Show account details and stored secret
    Show {
        /// Name of the account
        account: String,
        /// Print only the raw secret or URI
        #[arg(short, long)]
        raw: bool,
    },
    /// Export accounts to an encrypted backup file or stdout
    Export {
        /// Destination backup file path (prints to stdout if omitted)
        file: Option<PathBuf>,
        /// Export without encryption (plain JSON)
        #[arg(long)]
        unencrypted: bool,
        /// Master passphrase for encryption (prompted securely if omitted)
        #[arg(long, env = "TFA_PASSWORD")]
        password: Option<String>,
    },
    /// Import accounts from a backup file, migration URI, or JSON payload
    Import {
        /// Path to backup file or raw payload string
        file_or_payload: String,
        /// Preview accounts to import without saving them
        #[arg(long)]
        dry_run: bool,
        /// Overwrite existing accounts if conflicts occur
        #[arg(long, conflicts_with = "skip")]
        overwrite: bool,
        /// Skip existing accounts if conflicts occur (default)
        #[arg(long, conflicts_with = "overwrite")]
        skip: bool,
        /// Master passphrase for encrypted backup (prompted securely if omitted)
        #[arg(long, env = "TFA_PASSWORD")]
        password: Option<String>,
    },
    /// Generate shell autocompletions script
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

pub fn generate_completions(shell: Shell, writer: &mut dyn Write) {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, writer);
}

pub fn run_app(
    cli: Cli,
    store: &dyn AccountStore,
    clipboard: &dyn Clipboard,
    out_writer: &mut dyn Write,
) -> anyhow::Result<()> {
    match cli.command {
        Some(Commands::Generate { account, copy }) => {
            let secret = store.get_secret(&account)?;
            let engine = TfaTotp::from_input(&account, &secret)?;
            let code = engine.generate_current();
            let ttl = engine.ttl();

            if copy {
                clipboard.set_text(&code)?;
                writeln!(out_writer, "{code} ({ttl}s remaining) [copied]")?;
            } else {
                writeln!(out_writer, "{code} ({ttl}s remaining)")?;
            }
        }
        Some(Commands::Add {
            account,
            secret,
            digits,
            period,
            algorithm,
        }) => {
            let raw_secret = match secret {
                Some(s) => s,
                None => {
                    eprint!("Enter TOTP secret for '{account}': ");
                    let mut buffer = String::new();
                    let stdin = io::stdin();
                    stdin.lock().read_line(&mut buffer)?;
                    buffer.trim().to_string()
                }
            };

            // Validate and generate canonical OTPAuth URL with custom parameters
            let engine = TfaTotp::build(&account, &raw_secret, digits, period, algorithm)?;
            let url = engine.to_url()?;
            store.set_secret(&account, &url)?;
            writeln!(out_writer, "Account '{account}' added successfully.")?;
        }
        Some(Commands::List) => {
            let accounts = store.list_accounts()?;
            if accounts.is_empty() {
                writeln!(out_writer, "No accounts configured.")?;
            } else {
                for account in accounts {
                    writeln!(out_writer, "{account}")?;
                }
            }
        }
        Some(Commands::Delete { account }) => {
            store.delete_account(&account)?;
            writeln!(out_writer, "Account '{account}' deleted.")?;
        }
        Some(Commands::Show { account, raw }) => {
            let secret = store.get_secret(&account)?;
            if raw {
                writeln!(out_writer, "{secret}")?;
            } else {
                let engine = TfaTotp::from_input(&account, &secret)?;
                writeln!(out_writer, "Account:   {account}")?;
                writeln!(out_writer, "Digits:    {}", engine.digits())?;
                writeln!(out_writer, "Period:    {}s", engine.step())?;
                writeln!(out_writer, "Algorithm: {}", engine.algorithm())?;
                writeln!(out_writer, "Secret:    {secret}")?;
                writeln!(
                    out_writer,
                    "Live Code: {} ({}s remaining)",
                    engine.generate_current(),
                    engine.ttl()
                )?;
            }
        }
        Some(Commands::Completions { shell }) => {
            generate_completions(shell, out_writer);
        }
        Some(Commands::Export {
            file,
            unencrypted,
            password,
        }) => {
            let accounts = store.list_accounts()?;
            let mut map = BTreeMap::new();
            for account in &accounts {
                let secret = store.get_secret(account)?;
                map.insert(account.clone(), secret);
            }

            let json_str = serde_json::to_string_pretty(&map)?;
            let output_str = if unencrypted {
                json_str
            } else {
                let pass = get_passphrase(password, true)?;
                backup::encrypt_backup(json_str.as_bytes(), &pass)?
            };

            if let Some(dest_path) = file {
                #[cfg(unix)]
                {
                    use std::fs::OpenOptions;
                    use std::os::unix::fs::OpenOptionsExt;
                    let mut f = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .mode(0o600)
                        .open(&dest_path)?;
                    f.write_all(output_str.as_bytes())?;
                }
                #[cfg(not(unix))]
                {
                    std::fs::write(&dest_path, output_str.as_bytes())?;
                }
                writeln!(
                    out_writer,
                    "Exported {} account(s) to {}",
                    accounts.len(),
                    dest_path.display()
                )?;
            } else {
                writeln!(out_writer, "{output_str}")?;
            }
        }
        Some(Commands::Import {
            file_or_payload,
            dry_run,
            overwrite,
            skip: _,
            password,
        }) => {
            let path = Path::new(&file_or_payload);
            let content = if path.is_file() {
                std::fs::read_to_string(path)
                    .with_context(|| format!("Failed reading backup file '{}'", path.display()))?
            } else if path.exists() {
                anyhow::bail!("Path '{}' is not a file", path.display());
            } else if !file_or_payload.starts_with("otpauth-migration://")
                && !file_or_payload.starts_with("otpauth://")
                && (file_or_payload.ends_with(".json")
                    || file_or_payload.ends_with(".enc")
                    || file_or_payload.ends_with(".bak")
                    || file_or_payload.contains(std::path::MAIN_SEPARATOR))
            {
                anyhow::bail!("File not found: '{}'", file_or_payload);
            } else {
                file_or_payload
            };

            let raw_data = if backup::is_encrypted_backup(&content) {
                let pass = get_passphrase(password, false)?;
                let decrypted = backup::decrypt_backup(&content, &pass)?;
                String::from_utf8(decrypted).context("Decrypted backup is not valid UTF-8")?
            } else {
                content
            };

            let items = import::parse_import_payload(&raw_data)?;
            let mut imported = 0;
            let mut updated = 0;
            let mut skipped = 0;

            for item in items {
                let exists = store.get_secret(&item.account).is_ok();
                if exists {
                    if overwrite {
                        if !dry_run {
                            let url = match TfaTotp::from_input(&item.account, &item.secret) {
                                Ok(engine) => {
                                    engine.to_url().unwrap_or_else(|_| item.secret.clone())
                                }
                                Err(_) => item.secret.clone(),
                            };
                            store.set_secret(&item.account, &url)?;
                        }
                        updated += 1;
                        if dry_run {
                            writeln!(
                                out_writer,
                                "[dry-run] Would update account '{}'",
                                item.account
                            )?;
                        } else {
                            writeln!(out_writer, "Updated account '{}'", item.account)?;
                        }
                    } else {
                        skipped += 1;
                        if dry_run {
                            writeln!(
                                out_writer,
                                "[dry-run] Would skip existing account '{}'",
                                item.account
                            )?;
                        } else {
                            writeln!(
                                out_writer,
                                "Skipping existing account '{}' (use --overwrite to replace)",
                                item.account
                            )?;
                        }
                    }
                } else {
                    if !dry_run {
                        let url = match TfaTotp::from_input(&item.account, &item.secret) {
                            Ok(engine) => engine.to_url().unwrap_or_else(|_| item.secret.clone()),
                            Err(_) => item.secret.clone(),
                        };
                        store.set_secret(&item.account, &url)?;
                    }
                    imported += 1;
                    if dry_run {
                        writeln!(
                            out_writer,
                            "[dry-run] Would import account '{}'",
                            item.account
                        )?;
                    } else {
                        writeln!(out_writer, "Imported account '{}'", item.account)?;
                    }
                }
            }

            if dry_run {
                writeln!(
                    out_writer,
                    "[dry-run] Summary: {imported} to import, {updated} to update, {skipped} to skip."
                )?;
            } else {
                writeln!(
                    out_writer,
                    "Summary: {imported} imported, {updated} updated, {skipped} skipped."
                )?;
            }
        }
        None => {
            writeln!(out_writer, "Use --help for usage instructions.")?;
        }
    }
    Ok(())
}

fn get_passphrase(password: Option<String>, confirm: bool) -> anyhow::Result<String> {
    if let Some(p) = password {
        if p.is_empty() {
            anyhow::bail!("Passphrase cannot be empty");
        }
        return Ok(p);
    }

    let pass = rpassword::prompt_password("Enter master passphrase: ")?;
    if pass.is_empty() {
        anyhow::bail!("Passphrase cannot be empty");
    }

    if confirm {
        let confirm_pass = rpassword::prompt_password("Confirm master passphrase: ")?;
        if pass != confirm_pass {
            anyhow::bail!("Passphrases do not match");
        }
    }

    Ok(pass)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let clipboard = SystemClipboard;

    let store: Box<dyn AccountStore> = match cli.storage {
        StorageBackend::Auto => Box::new(AutoStore::default_auto()?),
        StorageBackend::Keyring => Box::new(KeyringStore::default()),
        StorageBackend::File => Box::new(FileStore::new(FileStore::default_path()?)),
    };

    run_app(cli, store.as_ref(), &clipboard, &mut io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipboard::MockClipboard;
    use storage::MemoryStore;

    #[test]
    fn test_cli_parse_default() {
        let cli = Cli::try_parse_from(["tfa"]);
        assert!(cli.is_ok());
        let unwrapped = cli.unwrap();
        assert!(unwrapped.command.is_none());
        assert_eq!(unwrapped.storage, StorageBackend::Auto);
    }

    #[test]
    fn test_cli_parse_storage_flag() {
        let cli = Cli::try_parse_from(["tfa", "--storage", "file", "list"]).unwrap();
        assert_eq!(cli.storage, StorageBackend::File);

        let cli_keyring = Cli::try_parse_from(["tfa", "--storage", "keyring", "list"]).unwrap();
        assert_eq!(cli_keyring.storage, StorageBackend::Keyring);
    }

    #[test]
    fn test_cli_parse_generate() {
        let cli = Cli::try_parse_from(["tfa", "generate", "github"]).unwrap();
        match cli.command {
            Some(Commands::Generate { account, copy }) => {
                assert_eq!(account, "github");
                assert!(!copy);
            }
            _ => panic!("Expected generate subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_generate_copy_flags() {
        // Short flag -c
        let cli_short = Cli::try_parse_from(["tfa", "generate", "github", "-c"]).unwrap();
        match cli_short.command {
            Some(Commands::Generate { account, copy }) => {
                assert_eq!(account, "github");
                assert!(copy);
            }
            _ => panic!("Expected generate subcommand"),
        }

        // Long flag --copy
        let cli_long = Cli::try_parse_from(["tfa", "generate", "aws", "--copy"]).unwrap();
        match cli_long.command {
            Some(Commands::Generate { account, copy }) => {
                assert_eq!(account, "aws");
                assert!(copy);
            }
            _ => panic!("Expected generate subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_add_custom_flags() {
        let cli = Cli::try_parse_from([
            "tfa",
            "add",
            "custom",
            "JBSWY3DPEHPK3PXP",
            "--digits",
            "8",
            "--period",
            "60",
            "--algorithm",
            "sha256",
        ])
        .unwrap();

        match cli.command {
            Some(Commands::Add {
                account,
                digits,
                period,
                algorithm,
                ..
            }) => {
                assert_eq!(account, "custom");
                assert_eq!(digits, Some(8));
                assert_eq!(period, Some(60));
                assert_eq!(algorithm, Some(totp_rs::Algorithm::SHA256));
            }
            _ => panic!("Expected add subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_completions() {
        let shells = [
            ("bash", Shell::Bash),
            ("zsh", Shell::Zsh),
            ("fish", Shell::Fish),
            ("powershell", Shell::PowerShell),
            ("elvish", Shell::Elvish),
        ];

        for (shell_str, expected) in shells {
            let cli = Cli::try_parse_from(["tfa", "completions", shell_str]).unwrap();
            match cli.command {
                Some(Commands::Completions { shell }) => assert_eq!(shell, expected),
                _ => panic!("Expected completions subcommand"),
            }
        }
    }

    #[test]
    fn test_generate_completions_scripts() {
        // Bash
        let mut bash_buf = Vec::new();
        generate_completions(Shell::Bash, &mut bash_buf);
        let bash_out = String::from_utf8(bash_buf).unwrap();
        assert!(bash_out.contains("complete -F _tfa -o nosort -o bashdefault -o default tfa"));
        assert!(bash_out.contains("generate"));
        assert!(bash_out.contains("completions"));

        // Zsh
        let mut zsh_buf = Vec::new();
        generate_completions(Shell::Zsh, &mut zsh_buf);
        let zsh_out = String::from_utf8(zsh_buf).unwrap();
        assert!(zsh_out.contains("#compdef tfa"));

        // Fish
        let mut fish_buf = Vec::new();
        generate_completions(Shell::Fish, &mut fish_buf);
        let fish_out = String::from_utf8(fish_buf).unwrap();
        assert!(fish_out.contains("complete -c tfa"));
    }

    #[test]
    fn test_run_app_add_and_generate_flow() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        // Add
        let add_cli =
            Cli::try_parse_from(["tfa", "add", "test_acc", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"])
                .unwrap();
        assert!(run_app(add_cli, &store, &cb, &mut out).is_ok());

        // List
        let list_cli = Cli::try_parse_from(["tfa", "list"]).unwrap();
        assert!(run_app(list_cli, &store, &cb, &mut out).is_ok());

        // Generate without copy
        let gen_cli = Cli::try_parse_from(["tfa", "generate", "test_acc"]).unwrap();
        assert!(run_app(gen_cli, &store, &cb, &mut out).is_ok());
        assert!(cb.text.lock().unwrap().is_none());

        // Generate with copy
        let gen_copy_cli = Cli::try_parse_from(["tfa", "generate", "test_acc", "-c"]).unwrap();
        assert!(run_app(gen_copy_cli, &store, &cb, &mut out).is_ok());
        let copied = cb.text.lock().unwrap().clone();
        assert!(copied.is_some());
        let code = copied.unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));

        // Delete
        let del_cli = Cli::try_parse_from(["tfa", "delete", "test_acc"]).unwrap();
        assert!(run_app(del_cli, &store, &cb, &mut out).is_ok());

        // Generate deleted account should fail
        let gen_deleted = Cli::try_parse_from(["tfa", "generate", "test_acc"]).unwrap();
        assert!(run_app(gen_deleted, &store, &cb, &mut out).is_err());
    }

    #[test]
    fn test_run_app_add_custom_parameters_flow() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let add_cli = Cli::try_parse_from([
            "tfa",
            "add",
            "custom8",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "--digits",
            "8",
            "--period",
            "60",
            "--algorithm",
            "sha256",
        ])
        .unwrap();
        assert!(run_app(add_cli, &store, &cb, &mut out).is_ok());

        // Generate token and verify 8 digits
        let gen_cli = Cli::try_parse_from(["tfa", "generate", "custom8"]).unwrap();
        let mut gen_out = Vec::new();
        assert!(run_app(gen_cli, &store, &cb, &mut gen_out).is_ok());
        let gen_str = String::from_utf8(gen_out).unwrap();
        let token = gen_str.split_whitespace().next().unwrap();
        assert_eq!(token.len(), 8);
        assert!(token.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_run_app_list_empty() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();
        let list_cli = Cli::try_parse_from(["tfa", "list"]).unwrap();
        assert!(run_app(list_cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("No accounts configured."));
    }

    #[test]
    fn test_run_app_delete_missing_fails() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();
        let del_cli = Cli::try_parse_from(["tfa", "delete", "missing"]).unwrap();
        assert!(run_app(del_cli, &store, &cb, &mut out).is_err());
    }

    #[test]
    fn test_run_app_none_command() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();
        let none_cli = Cli::try_parse_from(["tfa"]).unwrap();
        assert!(run_app(none_cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("Use --help for usage instructions."));
    }

    #[test]
    fn test_run_app_completions() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();
        let cli = Cli::try_parse_from(["tfa", "completions", "bash"]).unwrap();
        assert!(run_app(cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("complete -F _tfa"));
    }

    #[test]
    fn test_cli_parse_show() {
        let cli = Cli::try_parse_from(["tfa", "show", "github"]).unwrap();
        match cli.command {
            Some(Commands::Show { account, raw }) => {
                assert_eq!(account, "github");
                assert!(!raw);
            }
            _ => panic!("Expected show subcommand"),
        }

        let cli_raw = Cli::try_parse_from(["tfa", "show", "github", "-r"]).unwrap();
        match cli_raw.command {
            Some(Commands::Show { account, raw }) => {
                assert_eq!(account, "github");
                assert!(raw);
            }
            _ => panic!("Expected show subcommand"),
        }
    }

    #[test]
    fn test_run_app_show() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        store
            .set_secret("my_acc", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")
            .unwrap();

        // Normal show
        let show_cli = Cli::try_parse_from(["tfa", "show", "my_acc"]).unwrap();
        assert!(run_app(show_cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("Account:   my_acc"));
        assert!(out_str.contains("Digits:    6"));
        assert!(out_str.contains("Period:    30s"));
        assert!(out_str.contains("Algorithm: SHA1"));
        assert!(out_str.contains("Live Code:"));

        // Raw show
        let mut raw_out = Vec::new();
        let show_raw = Cli::try_parse_from(["tfa", "show", "my_acc", "--raw"]).unwrap();
        assert!(run_app(show_raw, &store, &cb, &mut raw_out).is_ok());
        let raw_str = String::from_utf8(raw_out).unwrap();
        assert_eq!(raw_str.trim(), "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");

        // Missing account fails
        let mut err_out = Vec::new();
        let show_missing = Cli::try_parse_from(["tfa", "show", "missing"]).unwrap();
        assert!(run_app(show_missing, &store, &cb, &mut err_out).is_err());
    }

    #[test]
    fn test_cli_parse_export() {
        let cli = Cli::try_parse_from(["tfa", "export"]).unwrap();
        match cli.command {
            Some(Commands::Export {
                file,
                unencrypted,
                password,
            }) => {
                assert!(file.is_none());
                assert!(!unencrypted);
                assert!(password.is_none());
            }
            _ => panic!("Expected export subcommand"),
        }

        let cli_file = Cli::try_parse_from([
            "tfa",
            "export",
            "backup.enc",
            "--unencrypted",
            "--password",
            "pass123",
        ])
        .unwrap();
        match cli_file.command {
            Some(Commands::Export {
                file,
                unencrypted,
                password,
            }) => {
                assert_eq!(file, Some(PathBuf::from("backup.enc")));
                assert!(unencrypted);
                assert_eq!(password, Some("pass123".to_string()));
            }
            _ => panic!("Expected export subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_import() {
        let cli = Cli::try_parse_from(["tfa", "import", "backup.enc"]).unwrap();
        match cli.command {
            Some(Commands::Import {
                file_or_payload,
                dry_run,
                overwrite,
                skip,
                password,
            }) => {
                assert_eq!(file_or_payload, "backup.enc");
                assert!(!dry_run);
                assert!(!overwrite);
                assert!(!skip);
                assert!(password.is_none());
            }
            _ => panic!("Expected import subcommand"),
        }

        let cli_flags = Cli::try_parse_from([
            "tfa",
            "import",
            "backup.enc",
            "--dry-run",
            "--overwrite",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli_flags.command {
            Some(Commands::Import {
                dry_run,
                overwrite,
                password,
                ..
            }) => {
                assert!(dry_run);
                assert!(overwrite);
                assert_eq!(password, Some("secret".to_string()));
            }
            _ => panic!("Expected import subcommand"),
        }
    }

    #[test]
    fn test_run_app_export_and_import_encrypted_flow() {
        let store1 = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        store1
            .set_secret("acc1", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")
            .unwrap();
        store1.set_secret("acc2", "JBSWY3DPEHPK3PXP").unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let backup_file = temp_dir.path().join("backup.json");

        // Export encrypted
        let export_cli = Cli::try_parse_from([
            "tfa",
            "export",
            backup_file.to_str().unwrap(),
            "--password",
            "masterpwd",
        ])
        .unwrap();
        assert!(run_app(export_cli, &store1, &cb, &mut out).is_ok());

        let file_contents = std::fs::read_to_string(&backup_file).unwrap();
        assert!(backup::is_encrypted_backup(&file_contents));

        // Dry-run import into new store
        let store2 = MemoryStore::new();
        let mut dry_out = Vec::new();
        let import_dry = Cli::try_parse_from([
            "tfa",
            "import",
            backup_file.to_str().unwrap(),
            "--dry-run",
            "--password",
            "masterpwd",
        ])
        .unwrap();
        assert!(run_app(import_dry, &store2, &cb, &mut dry_out).is_ok());
        let dry_str = String::from_utf8(dry_out).unwrap();
        assert!(dry_str.contains("[dry-run] Summary: 2 to import, 0 to update, 0 to skip."));
        assert!(store2.list_accounts().unwrap().is_empty());

        // Actual import into new store
        let mut import_out = Vec::new();
        let import_cli = Cli::try_parse_from([
            "tfa",
            "import",
            backup_file.to_str().unwrap(),
            "--password",
            "masterpwd",
        ])
        .unwrap();
        assert!(run_app(import_cli, &store2, &cb, &mut import_out).is_ok());
        let import_str = String::from_utf8(import_out).unwrap();
        assert!(import_str.contains("Summary: 2 imported, 0 updated, 0 skipped."));

        let accounts = store2.list_accounts().unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(store2.get_secret("acc1").is_ok());
        assert!(store2.get_secret("acc2").is_ok());
    }

    #[test]
    fn test_run_app_import_conflict_skip_and_overwrite() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        store.set_secret("existing", "OLDSECRET").unwrap();

        let json_payload = r#"{"existing":"NEWSECRET","brand_new":"SOMESECRET"}"#;

        // Import with default / skip
        let import_skip = Cli::try_parse_from(["tfa", "import", json_payload, "--skip"]).unwrap();
        assert!(run_app(import_skip, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out.clone()).unwrap();
        assert!(out_str.contains("Skipping existing account 'existing'"));
        assert!(out_str.contains("Imported account 'brand_new'"));
        assert_eq!(store.get_secret("existing").unwrap(), "OLDSECRET");

        // Import with overwrite
        let mut out_overwrite = Vec::new();
        let import_over =
            Cli::try_parse_from(["tfa", "import", json_payload, "--overwrite"]).unwrap();
        assert!(run_app(import_over, &store, &cb, &mut out_overwrite).is_ok());
        let out_over_str = String::from_utf8(out_overwrite).unwrap();
        assert!(out_over_str.contains("Updated account 'existing'"));
        assert_ne!(store.get_secret("existing").unwrap(), "OLDSECRET");
    }

    #[test]
    fn test_run_app_export_unencrypted() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();

        let export_cli = Cli::try_parse_from(["tfa", "export", "--unencrypted"]).unwrap();
        assert!(run_app(export_cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains(r#""github": "JBSWY3DPEHPK3PXP""#));
        assert!(!backup::is_encrypted_backup(&out_str));
    }

    #[test]
    fn test_run_app_import_google_migration_payload() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let uri = "otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8SGFRlc3QxOnRlc3QxQGV4YW1wbGUxLmNvbRoFVGVzdDEgASgBMAIKMwoKSGVsbG8h3q2%2B8BIYVGVzdDI6dGVzdDJAZXhhbXBsZTIuY29tGgVUZXN0MiABKAEwAgozCgpIZWxsbyHerb7xEhhUZXN0Mzp0ZXN0M0BleGFtcGxlMy5jb20aBVRlc3QzIAEoATACEAEYASAAKI3orYEE";
        let import_cli = Cli::try_parse_from(["tfa", "import", uri]).unwrap();
        assert!(run_app(import_cli, &store, &cb, &mut out).is_ok());

        let accounts = store.list_accounts().unwrap();
        assert_eq!(accounts.len(), 3);
        assert!(accounts.contains(&"Test1:test1@example1.com".to_string()));
    }

    #[test]
    fn test_run_app_import_file_not_found_fails() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let cli = Cli::try_parse_from(["tfa", "import", "non_existent_file.json"]).unwrap();
        assert!(run_app(cli, &store, &cb, &mut out).is_err());
    }

    #[test]
    fn test_run_app_import_directory_fails() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let temp_dir = tempfile::tempdir().unwrap();
        let cli =
            Cli::try_parse_from(["tfa", "import", temp_dir.path().to_str().unwrap()]).unwrap();
        assert!(run_app(cli, &store, &cb, &mut out).is_err());
    }

    #[test]
    fn test_get_passphrase_empty_flag_fails() {
        let err = get_passphrase(Some("".to_string()), false);
        assert!(err.is_err());
    }
}
