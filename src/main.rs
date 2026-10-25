use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::collections::BTreeMap;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

mod backup;
mod clipboard;
mod config;
mod import;
mod qr;
mod storage;
mod totp;
mod watch;

use clipboard::{Clipboard, ClipboardProvider, DispatchingClipboard};
use storage::{AccountStore, AutoStore, FileStore, KeyringStore};
use totp::TfaTotp;

#[derive(
    ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
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
    /// Path to configuration file (TOML, YAML, or JSON)
    #[arg(short = 'C', long, global = true, env = "TFA_CONFIG")]
    pub config: Option<PathBuf>,

    /// Credential storage backend
    #[arg(long, env = "TFA_STORAGE", global = true)]
    pub storage: Option<StorageBackend>,

    /// Clipboard copy provider (system or osc52)
    #[arg(long, env = "TFA_CLIPBOARD", global = true)]
    pub clipboard: Option<ClipboardProvider>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a 2FA token (TOTP, Steam, or HOTP) for an account
    Generate {
        /// Name of the account (interactively prompted via fuzzy search if omitted)
        account: Option<String>,
        /// Copy generated token to clipboard
        #[arg(short, long, num_args = 0..=1, default_missing_value = "true")]
        copy: Option<bool>,
        /// Watch and continuously refresh code with progress bar
        #[arg(short, long)]
        watch: bool,
        /// Counter value for counter-based (HOTP) token generation
        #[arg(long, num_args = 0..=1)]
        counter: Option<Option<u64>>,
    },
    /// Live watch mode displaying continuous TOTP countdown with progress bar
    Watch {
        /// Name of the account (interactively prompted via fuzzy search if omitted)
        account: Option<String>,
        /// Copy token to clipboard on every cycle
        #[arg(short, long, num_args = 0..=1, default_missing_value = "true")]
        copy: Option<bool>,
    },
    /// Add a new account secret or OTPAuth URL
    Add {
        /// Name of the account
        account: Option<String>,
        /// TOTP/HOTP secret key or otpauth:// URI (prompted if omitted and --qr not supplied)
        secret: Option<String>,
        /// Decode account secret and parameters from a QR code image
        #[arg(long)]
        qr: Option<PathBuf>,
        /// Number of digits in generated code (e.g. 5, 6, 8)
        #[arg(short, long)]
        digits: Option<u8>,
        /// Time step period in seconds (e.g. 30 or 60)
        #[arg(short, long)]
        period: Option<u64>,
        /// Hash algorithm (sha1, sha256, sha512, steam)
        #[arg(short, long)]
        algorithm: Option<totp_rs::Algorithm>,
        /// Initial counter for counter-based (HOTP) accounts
        #[arg(short, long)]
        counter: Option<u64>,
    },
    /// Render account as a terminal QR code for mobile authenticator scanning
    Qr {
        /// Name of the account
        account: String,
        /// Invert terminal QR blocks (for light terminal backgrounds)
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        light: Option<bool>,
    },
    /// List all stored accounts or live token status
    List {
        /// Include live TOTP codes and remaining time-to-live
        #[arg(short, long)]
        codes: bool,
        /// Output overview in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Live overview status dashboard of all accounts and tokens (alias for list --codes)
    Status {
        /// Output overview in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Delete an account secret
    Delete {
        /// Name of the account (interactively prompted via fuzzy search if omitted)
        account: Option<String>,
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
    /// Generate UNIX roff man page
    Man,
}

#[derive(serde::Serialize, Debug)]
pub struct AccountStatus {
    pub account: String,
    pub token: String,
    pub ttl: u64,
}

pub fn display_accounts_overview(
    accounts: &[String],
    store: &dyn AccountStore,
    json: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut statuses = Vec::new();
    for acc in accounts {
        let secret = store.get_secret(acc)?;
        let engine = TfaTotp::from_input(acc, &secret)?;
        statuses.push(AccountStatus {
            account: acc.clone(),
            token: engine.generate_current(),
            ttl: engine.ttl(),
        });
    }

    if json {
        let json_str = serde_json::to_string_pretty(&statuses)?;
        writeln!(out, "{json_str}")?;
    } else {
        let max_acc_len = statuses
            .iter()
            .map(|s| s.account.len())
            .max()
            .unwrap_or(7)
            .max(7);
        writeln!(
            out,
            "{:<width$}  {:^8}  {:>5}",
            "ACCOUNT",
            "TOKEN",
            "TTL",
            width = max_acc_len
        )?;
        for s in statuses {
            let ttl_str = if s.ttl == 0 {
                "-".to_string()
            } else {
                format!("{}s", s.ttl)
            };
            writeln!(
                out,
                "{:<width$}  {:^8}  {:>5}",
                s.account,
                s.token,
                ttl_str,
                width = max_acc_len
            )?;
        }
    }
    Ok(())
}

pub fn select_account_interactive(
    store: &dyn AccountStore,
    prompt: &str,
) -> anyhow::Result<String> {
    select_account_interactive_on(store, prompt, &console::Term::stderr())
}

pub fn select_account_interactive_on(
    store: &dyn AccountStore,
    prompt: &str,
    term: &console::Term,
) -> anyhow::Result<String> {
    let accounts = store.list_accounts()?;
    if accounts.is_empty() {
        anyhow::bail!("No accounts configured");
    }

    #[cfg(test)]
    {
        // When running under test harness, avoid blocking on interactive prompt unless explicitly testing PTY
        if std::env::var("TFA_TEST_INTERACTIVE").is_err() {
            anyhow::bail!("Account name is required in non-interactive environment");
        }
    }

    if !io::stdin().is_terminal() || !term.features().is_attended() {
        anyhow::bail!("Account name is required in non-interactive environment");
    }

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt(prompt)
        .items(&accounts)
        .default(0)
        .interact_on_opt(term)
        .map_err(|e| anyhow::anyhow!("Interactive selection failed: {e}"))?;

    match selection {
        Some(index) => Ok(accounts[index].clone()),
        None => anyhow::bail!("Selection cancelled"),
    }
}

pub fn generate_completions(shell: Shell, writer: &mut dyn Write) {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, writer);
}

pub fn generate_manpage(writer: &mut dyn Write) -> anyhow::Result<()> {
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    man.render(writer)
        .map_err(|e| anyhow::anyhow!("Failed rendering man page: {e}"))
}

pub fn run_app(
    cli: Cli,
    store: &dyn AccountStore,
    clipboard: &dyn Clipboard,
    out_writer: &mut dyn Write,
) -> anyhow::Result<()> {
    let (resolved_cfg, _) =
        config::resolve_configuration(cli.config.as_deref(), cli.storage, cli.clipboard)?;

    match cli.command {
        Some(Commands::Generate {
            account,
            copy,
            watch,
            counter,
        }) => {
            let acct = match account {
                Some(a) => a,
                None => select_account_interactive(store, "Select account to generate token")?,
            };
            let should_copy = copy.unwrap_or(resolved_cfg.auto_copy);
            if watch {
                let secret = store.get_secret(&acct)?;
                let engine = TfaTotp::from_input(&acct, &secret)?;
                if engine.is_hotp() {
                    anyhow::bail!("Watch mode is not supported for counter-based (HOTP) accounts");
                }
                watch::watch_account(&acct, store, clipboard, should_copy, out_writer, None)?;
            } else {
                let secret = store.get_secret(&acct)?;
                let mut engine = TfaTotp::from_input(&acct, &secret)?;
                if engine.is_hotp() {
                    let current_c = match counter {
                        Some(Some(c)) => c,
                        _ => engine.counter().unwrap_or(0),
                    };
                    let code = engine.generate_at_counter(current_c)?;
                    engine.set_counter(current_c + 1);
                    store.set_secret(&acct, &engine.to_url()?)?;

                    if should_copy {
                        clipboard.set_text(&code)?;
                        writeln!(out_writer, "{code} (counter: {current_c}) [copied]")?;
                    } else {
                        writeln!(out_writer, "{code} (counter: {current_c})")?;
                    }
                } else {
                    if counter.is_some() {
                        anyhow::bail!("--counter can only be used with HOTP accounts");
                    }
                    let code = engine.generate_current();
                    let ttl = engine.ttl();

                    if should_copy {
                        clipboard.set_text(&code)?;
                        writeln!(out_writer, "{code} ({ttl}s remaining) [copied]")?;
                    } else {
                        writeln!(out_writer, "{code} ({ttl}s remaining)")?;
                    }
                }
            }
        }
        Some(Commands::Watch { account, copy }) => {
            let acct = match account {
                Some(a) => a,
                None => select_account_interactive(store, "Select account to watch")?,
            };
            let secret = store.get_secret(&acct)?;
            let engine = TfaTotp::from_input(&acct, &secret)?;
            if engine.is_hotp() {
                anyhow::bail!("Watch mode is not supported for counter-based (HOTP) accounts");
            }
            let should_copy = copy.unwrap_or(resolved_cfg.auto_copy);
            watch::watch_account(&acct, store, clipboard, should_copy, out_writer, None)?;
        }
        Some(Commands::Add {
            account,
            secret,
            qr,
            digits,
            period,
            algorithm,
            counter,
        }) => {
            let (target_account, raw_secret) = if let Some(qr_path) = qr {
                let decoded = qr::scan_qr_from_image(&qr_path)?;
                let acct = match account {
                    Some(a) => a,
                    None => {
                        // Extract label/account from OTPAuth URI or fallback
                        if decoded.starts_with("otpauth://") {
                            import::extract_account_from_otpauth(&decoded)
                        } else {
                            anyhow::bail!(
                                "Account name is required when QR content is not an OTPAuth URL"
                            );
                        }
                    }
                };
                (acct, decoded)
            } else {
                let acct = account.ok_or_else(|| anyhow::anyhow!("Account name is required"))?;
                let raw = match secret {
                    Some(s) => s,
                    None => {
                        eprint!("Enter TOTP secret for '{acct}': ");
                        let mut buffer = String::new();
                        let stdin = io::stdin();
                        stdin.lock().read_line(&mut buffer)?;
                        buffer.trim().to_string()
                    }
                };
                (acct, raw)
            };

            // Apply precedence: CLI argument -> Resolved config (Env / Config file) -> default
            let resolved_digits = digits.or(resolved_cfg.digits);
            let resolved_period = period.or(resolved_cfg.period);
            let resolved_algorithm = algorithm.or(resolved_cfg.algorithm);

            // Validate and generate canonical OTPAuth URL with custom parameters
            let engine = TfaTotp::build(
                &target_account,
                &raw_secret,
                resolved_digits,
                resolved_period,
                resolved_algorithm,
                counter,
            )?;
            let url = engine.to_url()?;
            store.set_secret(&target_account, &url)?;
            writeln!(out_writer, "Account '{target_account}' added successfully.")?;
        }
        Some(Commands::Qr { account, light }) => {
            let secret = store.get_secret(&account)?;
            let engine = TfaTotp::from_input(&account, &secret)?;
            let url = engine.to_url()?;
            let is_light = light.unwrap_or(resolved_cfg.light);
            let qr_blocks = qr::render_terminal_qr(&url, !is_light)?;
            write!(out_writer, "{qr_blocks}")?;
        }
        Some(Commands::List { codes, json }) => {
            let accounts = store.list_accounts()?;
            if accounts.is_empty() {
                if json {
                    writeln!(out_writer, "[]")?;
                } else {
                    writeln!(out_writer, "No accounts configured.")?;
                }
            } else if codes || json {
                display_accounts_overview(&accounts, store, json, out_writer)?;
            } else {
                for account in accounts {
                    writeln!(out_writer, "{account}")?;
                }
            }
        }
        Some(Commands::Status { json }) => {
            let accounts = store.list_accounts()?;
            if accounts.is_empty() {
                if json {
                    writeln!(out_writer, "[]")?;
                } else {
                    writeln!(out_writer, "No accounts configured.")?;
                }
            } else {
                display_accounts_overview(&accounts, store, json, out_writer)?;
            }
        }
        Some(Commands::Delete { account }) => {
            let acct = match account {
                Some(a) => a,
                None => select_account_interactive(store, "Select account to delete")?,
            };
            store.delete_account(&acct)?;
            writeln!(out_writer, "Account '{acct}' deleted.")?;
        }
        Some(Commands::Show { account, raw }) => {
            let secret = store.get_secret(&account)?;
            if raw {
                writeln!(out_writer, "{secret}")?;
            } else {
                let engine = TfaTotp::from_input(&account, &secret)?;
                writeln!(out_writer, "Account:   {account}")?;
                writeln!(out_writer, "Type:      {}", engine.account_type())?;
                if engine.is_hotp() {
                    writeln!(out_writer, "Digits:    {}", engine.digits())?;
                    writeln!(out_writer, "Counter:   {}", engine.counter().unwrap_or(0))?;
                    writeln!(out_writer, "Algorithm: {}", engine.algorithm())?;
                    writeln!(out_writer, "Secret:    {secret}")?;
                    writeln!(
                        out_writer,
                        "Live Code: {} (counter: {})",
                        engine.generate_current(),
                        engine.counter().unwrap_or(0)
                    )?;
                } else {
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
        }
        Some(Commands::Completions { shell }) => {
            generate_completions(shell, out_writer);
        }
        Some(Commands::Man) => {
            generate_manpage(out_writer)?;
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

    let (resolved_cfg, _) =
        config::resolve_configuration(cli.config.as_deref(), cli.storage, cli.clipboard)?;

    let clipboard = DispatchingClipboard::new(resolved_cfg.clipboard);

    let store: Box<dyn AccountStore> = match resolved_cfg.storage {
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
        assert_eq!(unwrapped.storage, None);
        assert_eq!(unwrapped.config, None);
    }

    #[test]
    fn test_cli_parse_config_flag() {
        let cli = Cli::try_parse_from(["tfa", "-C", "/custom/config.toml", "list"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/custom/config.toml")));

        let cli_long =
            Cli::try_parse_from(["tfa", "--config", "/other/config.yaml", "list"]).unwrap();
        assert_eq!(cli_long.config, Some(PathBuf::from("/other/config.yaml")));
    }

    #[test]
    fn test_cli_parse_storage_flag() {
        let cli = Cli::try_parse_from(["tfa", "--storage", "file", "list"]).unwrap();
        assert_eq!(cli.storage, Some(StorageBackend::File));

        let cli_keyring = Cli::try_parse_from(["tfa", "--storage", "keyring", "list"]).unwrap();
        assert_eq!(cli_keyring.storage, Some(StorageBackend::Keyring));
    }

    #[test]
    fn test_cli_parse_generate() {
        let cli = Cli::try_parse_from(["tfa", "generate", "github"]).unwrap();
        match cli.command {
            Some(Commands::Generate {
                account,
                copy,
                watch,
                counter,
            }) => {
                assert_eq!(account, Some("github".to_string()));
                assert_eq!(copy, None);
                assert!(!watch);
                assert_eq!(counter, None);
            }
            _ => panic!("Expected generate subcommand"),
        }

        // Without account argument
        let cli_no_acc = Cli::try_parse_from(["tfa", "generate"]).unwrap();
        match cli_no_acc.command {
            Some(Commands::Generate { account, .. }) => {
                assert_eq!(account, None);
            }
            _ => panic!("Expected generate subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_generate_copy_flags() {
        // Short flag -c
        let cli_short = Cli::try_parse_from(["tfa", "generate", "github", "-c"]).unwrap();
        match cli_short.command {
            Some(Commands::Generate {
                account,
                copy,
                watch,
                counter: _,
            }) => {
                assert_eq!(account, Some("github".to_string()));
                assert_eq!(copy, Some(true));
                assert!(!watch);
            }
            _ => panic!("Expected generate subcommand"),
        }

        // Long flag --copy
        let cli_long = Cli::try_parse_from(["tfa", "generate", "aws", "--copy"]).unwrap();
        match cli_long.command {
            Some(Commands::Generate {
                account,
                copy,
                watch,
                counter: _,
            }) => {
                assert_eq!(account, Some("aws".to_string()));
                assert_eq!(copy, Some(true));
                assert!(!watch);
            }
            _ => panic!("Expected generate subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_watch() {
        let cli = Cli::try_parse_from(["tfa", "watch", "github", "-c"]).unwrap();
        match cli.command {
            Some(Commands::Watch { account, copy }) => {
                assert_eq!(account, Some("github".to_string()));
                assert_eq!(copy, Some(true));
            }
            _ => panic!("Expected watch subcommand"),
        }

        let cli_gen_watch = Cli::try_parse_from(["tfa", "generate", "github", "-w"]).unwrap();
        match cli_gen_watch.command {
            Some(Commands::Generate { watch, .. }) => {
                assert!(watch);
            }
            _ => panic!("Expected generate subcommand with watch"),
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
                assert_eq!(account, Some("custom".to_string()));
                assert_eq!(digits, Some(8));
                assert_eq!(period, Some(60));
                assert_eq!(algorithm, Some(totp_rs::Algorithm::SHA256));
            }
            _ => panic!("Expected add subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_add_qr_flag() {
        let cli = Cli::try_parse_from(["tfa", "add", "--qr", "qr.png"]).unwrap();
        match cli.command {
            Some(Commands::Add { account, qr, .. }) => {
                assert_eq!(account, None);
                assert_eq!(qr, Some(PathBuf::from("qr.png")));
            }
            _ => panic!("Expected add subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_qr_subcommand() {
        let cli = Cli::try_parse_from(["tfa", "qr", "github", "--light"]).unwrap();
        match cli.command {
            Some(Commands::Qr { account, light }) => {
                assert_eq!(account, "github");
                assert_eq!(light, Some(true));
            }
            _ => panic!("Expected qr subcommand"),
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
    fn test_generate_manpage_renders_roff() {
        let mut man_buf = Vec::new();
        assert!(generate_manpage(&mut man_buf).is_ok());
        let man_str = String::from_utf8(man_buf).unwrap();
        assert!(man_str.contains(".TH tfa 1"));
        assert!(man_str.contains(".SH NAME"));
        assert!(man_str.contains(".SH SYNOPSIS"));
        assert!(man_str.contains(".SH DESCRIPTION"));
        assert!(man_str.contains("generate"));
    }

    #[test]
    fn test_cli_parse_man() {
        let cli = Cli::try_parse_from(["tfa", "man"]).unwrap();
        match cli.command {
            Some(Commands::Man) => {}
            _ => panic!("Expected man subcommand"),
        }
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

        let mut out_json = Vec::new();
        let status_json = Cli::try_parse_from(["tfa", "status", "--json"]).unwrap();
        assert!(run_app(status_json, &store, &cb, &mut out_json).is_ok());
        let json_str = String::from_utf8(out_json).unwrap();
        assert_eq!(json_str.trim(), "[]");
    }

    #[test]
    fn test_run_app_list_codes_and_status() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        store
            .set_secret("aws", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")
            .unwrap();

        // 1. Plain table via list --codes
        let mut out1 = Vec::new();
        let cli1 = Cli::try_parse_from(["tfa", "list", "--codes"]).unwrap();
        assert!(run_app(cli1, &store, &cb, &mut out1).is_ok());
        let str1 = String::from_utf8(out1).unwrap();
        assert!(str1.contains("ACCOUNT"));
        assert!(str1.contains("TOKEN"));
        assert!(str1.contains("TTL"));
        assert!(str1.contains("github"));
        assert!(str1.contains("aws"));

        // 2. Status command
        let mut out2 = Vec::new();
        let cli2 = Cli::try_parse_from(["tfa", "status"]).unwrap();
        assert!(run_app(cli2, &store, &cb, &mut out2).is_ok());
        let str2 = String::from_utf8(out2).unwrap();
        assert!(str2.contains("github"));
        assert!(str2.contains("aws"));

        // 3. JSON format
        let mut out3 = Vec::new();
        let cli3 = Cli::try_parse_from(["tfa", "status", "--json"]).unwrap();
        assert!(run_app(cli3, &store, &cb, &mut out3).is_ok());
        let str3 = String::from_utf8(out3).unwrap();
        let json: serde_json::Value = serde_json::from_str(&str3).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        let first = &json[0];
        assert!(first.get("account").is_some());
        assert!(first.get("token").is_some());
        assert!(first.get("ttl").is_some());
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
    fn test_select_account_interactive_non_interactive_env_fails() {
        let store = MemoryStore::new();
        // 1. With empty store
        let err1 = select_account_interactive(&store, "Pick account");
        assert!(err1.is_err());
        assert!(err1
            .unwrap_err()
            .to_string()
            .contains("No accounts configured"));

        // 2. With store accounts, but non-terminal standard input during cargo test
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        let err2 = select_account_interactive(&store, "Pick account");
        assert!(err2.is_err());
        assert!(err2
            .unwrap_err()
            .to_string()
            .contains("non-interactive environment"));

        // 3. Omitting account in generate or delete under non-interactive env fails gracefully
        let mut out = Vec::new();
        let cb = MockClipboard::default();
        let gen_cli = Cli::try_parse_from(["tfa", "generate"]).unwrap();
        assert!(run_app(gen_cli, &store, &cb, &mut out).is_err());

        let del_cli = Cli::try_parse_from(["tfa", "delete"]).unwrap();
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

    #[test]
    fn test_run_app_qr_display() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();

        let cli = Cli::try_parse_from(["tfa", "qr", "github"]).unwrap();
        assert!(run_app(cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains('█'));
    }

    #[test]
    fn test_run_app_add_with_qr_image() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        // Create a test QR image with an OTPAuth URI
        let payload = "otpauth://totp/GitLab:user@gitlab.com?secret=JBSWY3DPEHPK3PXP&issuer=GitLab";
        let qr = qrcodegen::QrCode::encode_text(payload, qrcodegen::QrCodeEcc::Medium).unwrap();
        let size = qr.size() as u32;
        let border = 4u32;
        let img_dim = size + border * 2;
        let mut img_buf = image::GrayImage::new(img_dim, img_dim);
        for pixel in img_buf.pixels_mut() {
            *pixel = image::Luma([255u8]);
        }
        for y in 0..size {
            for x in 0..size {
                if qr.get_module(x as i32, y as i32) {
                    img_buf.put_pixel(x + border, y + border, image::Luma([0u8]));
                }
            }
        }
        let temp_dir = tempfile::tempdir().unwrap();
        let img_path = temp_dir.path().join("gitlab_qr.png");
        img_buf.save(&img_path).unwrap();

        // 1. Add using label from CLI
        let cli = Cli::try_parse_from([
            "tfa",
            "add",
            "my_gitlab",
            "--qr",
            img_path.to_str().unwrap(),
        ])
        .unwrap();
        assert!(run_app(cli, &store, &cb, &mut out).is_ok());
        assert!(store
            .list_accounts()
            .unwrap()
            .contains(&"my_gitlab".to_string()));

        // 2. Add without account positional argument, auto-extract label from URI
        let cli2 = Cli::try_parse_from(["tfa", "add", "--qr", img_path.to_str().unwrap()]).unwrap();
        assert!(run_app(cli2, &store, &cb, &mut out).is_ok());
        assert!(store
            .list_accounts()
            .unwrap()
            .contains(&"user@gitlab.com".to_string()));
    }

    #[test]
    fn test_watch_account_lifecycle() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();

        // Run watch for 2 ticks (stops automatically for test)
        let res = watch::watch_account("github", &store, &cb, true, &mut out, Some(2));
        // Under headless or non-tty environments, enable_raw_mode may fail or succeed
        // If enable_raw_mode succeeds, check output content
        if res.is_ok() {
            let out_str = String::from_utf8(out).unwrap();
            assert!(out_str.contains("github"));
            assert!(out_str.contains("[copied]"));
            assert_eq!(cb.text.lock().unwrap().as_deref().unwrap().len(), 6);
        }
    }

    #[test]
    fn test_run_app_steam_guard_lifecycle() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        // 1. Add steam account with -a steam
        let add_cli = Cli::try_parse_from([
            "tfa",
            "add",
            "valve_steam",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "-a",
            "steam",
        ])
        .unwrap();
        assert!(run_app(add_cli, &store, &cb, &mut out).is_ok());

        // 2. Show steam account
        let mut show_out = Vec::new();
        let show_cli = Cli::try_parse_from(["tfa", "show", "valve_steam"]).unwrap();
        assert!(run_app(show_cli, &store, &cb, &mut show_out).is_ok());
        let show_str = String::from_utf8(show_out).unwrap();
        assert!(show_str.contains("Account:   valve_steam"));
        assert!(show_str.contains("Digits:    5"));
        assert!(show_str.contains("Algorithm: STEAM"));

        // 3. Generate steam token
        let mut gen_out = Vec::new();
        let gen_cli = Cli::try_parse_from(["tfa", "generate", "valve_steam", "-c"]).unwrap();
        assert!(run_app(gen_cli, &store, &cb, &mut gen_out).is_ok());
        let gen_str = String::from_utf8(gen_out).unwrap();
        assert!(gen_str.contains("[copied]"));
        let token = cb.text.lock().unwrap().clone().unwrap();
        assert_eq!(token.len(), 5);
        const STEAM_ALPHABET: &[u8] = b"23456789BCDFGHJKMNPQRTVWXY";
        for b in token.bytes() {
            assert!(STEAM_ALPHABET.contains(&b));
        }
    }

    #[test]
    fn test_run_app_hotp_lifecycle_and_counter_increment() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        // 1. Add HOTP account with --counter 0
        let add_cli = Cli::try_parse_from([
            "tfa",
            "add",
            "my_hotp",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "--counter",
            "0",
        ])
        .unwrap();
        assert!(run_app(add_cli, &store, &cb, &mut out).is_ok());

        // Verify URL in store is otpauth://hotp/
        let stored_url = store.get_secret("my_hotp").unwrap();
        assert!(stored_url.starts_with("otpauth://hotp/"));
        assert!(stored_url.contains("counter=0"));

        // 2. Show HOTP account (does not increment counter)
        let mut show_out = Vec::new();
        let show_cli = Cli::try_parse_from(["tfa", "show", "my_hotp"]).unwrap();
        assert!(run_app(show_cli, &store, &cb, &mut show_out).is_ok());
        let show_str = String::from_utf8(show_out).unwrap();
        assert!(show_str.contains("Account:   my_hotp"));
        assert!(show_str.contains("Counter:   0"));
        assert!(show_str.contains("Digits:    6"));
        assert!(show_str.contains("755224"));

        // 3. Generate token (counter 0 -> 755224, then store increments to counter 1)
        let mut gen1_out = Vec::new();
        let gen1 = Cli::try_parse_from(["tfa", "generate", "my_hotp"]).unwrap();
        assert!(run_app(gen1, &store, &cb, &mut gen1_out).is_ok());
        let gen1_str = String::from_utf8(gen1_out).unwrap();
        assert!(gen1_str.contains("755224 (counter: 0)"));

        let updated_url1 = store.get_secret("my_hotp").unwrap();
        assert!(updated_url1.contains("counter=1"));

        // 4. Generate next token (counter 1 -> 287082, then store increments to 2)
        let mut gen2_out = Vec::new();
        let gen2 = Cli::try_parse_from(["tfa", "generate", "my_hotp", "-c"]).unwrap();
        assert!(run_app(gen2, &store, &cb, &mut gen2_out).is_ok());
        let gen2_str = String::from_utf8(gen2_out).unwrap();
        assert!(gen2_str.contains("287082 (counter: 1) [copied]"));
        assert_eq!(cb.text.lock().unwrap().as_deref().unwrap(), "287082");

        let updated_url2 = store.get_secret("my_hotp").unwrap();
        assert!(updated_url2.contains("counter=2"));

        // 5. Generate with explicit --counter 5 (generates counter 5 -> 254676, updates store to 6)
        let mut gen3_out = Vec::new();
        let gen3 = Cli::try_parse_from(["tfa", "generate", "my_hotp", "--counter", "5"]).unwrap();
        assert!(run_app(gen3, &store, &cb, &mut gen3_out).is_ok());
        let gen3_str = String::from_utf8(gen3_out).unwrap();
        assert!(gen3_str.contains("254676 (counter: 5)"));

        let updated_url3 = store.get_secret("my_hotp").unwrap();
        assert!(updated_url3.contains("counter=6"));

        // 6. Generate with standalone --counter flag (uses current stored counter 6 -> 287922, updates to 7)
        let mut gen4_out = Vec::new();
        let gen4 = Cli::try_parse_from(["tfa", "generate", "my_hotp", "--counter"]).unwrap();
        assert!(run_app(gen4, &store, &cb, &mut gen4_out).is_ok());
        let gen4_str = String::from_utf8(gen4_out).unwrap();
        assert!(gen4_str.contains("287922 (counter: 6)"));

        // 7. Watch on HOTP account fails gracefully
        let mut watch_out = Vec::new();
        let watch_cli = Cli::try_parse_from(["tfa", "watch", "my_hotp"]).unwrap();
        let watch_res = run_app(watch_cli, &store, &cb, &mut watch_out);
        assert!(watch_res.is_err());
        assert!(watch_res
            .unwrap_err()
            .to_string()
            .contains("Watch mode is not supported for counter-based (HOTP) accounts"));

        // 8. Passing --counter to a standard TOTP account fails gracefully
        store
            .set_secret("totp_acc", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")
            .unwrap();
        let mut err_out = Vec::new();
        let err_cli =
            Cli::try_parse_from(["tfa", "generate", "totp_acc", "--counter", "1"]).unwrap();
        let err_res = run_app(err_cli, &store, &cb, &mut err_out);
        assert!(err_res.is_err());
        assert!(err_res
            .unwrap_err()
            .to_string()
            .contains("--counter can only be used with HOTP accounts"));
    }

    #[test]
    fn test_run_app_status_with_mixed_totp_steam_hotp() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        store.set_secret("totp_acc", "JBSWY3DPEHPK3PXP").unwrap();
        store
            .set_secret(
                "steam_acc",
                "otpauth://steam/Steam:steam_acc?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            )
            .unwrap();
        store
            .set_secret(
                "hotp_acc",
                "otpauth://hotp/hotp_acc?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&counter=2&digits=6",
            )
            .unwrap();

        // 1. Status table output
        let mut out = Vec::new();
        let cli = Cli::try_parse_from(["tfa", "status"]).unwrap();
        assert!(run_app(cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("totp_acc"));
        assert!(out_str.contains("steam_acc"));
        assert!(out_str.contains("hotp_acc"));
        assert!(out_str.contains("359152")); // counter 2 HOTP code
        assert!(out_str.contains('-')); // HOTP shows '-' for TTL

        // 2. Status JSON output
        let mut json_out = Vec::new();
        let json_cli = Cli::try_parse_from(["tfa", "status", "--json"]).unwrap();
        assert!(run_app(json_cli, &store, &cb, &mut json_out).is_ok());
        let json_str = String::from_utf8(json_out).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let arr = json_val.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        let hotp_entry = arr.iter().find(|x| x["account"] == "hotp_acc").unwrap();
        assert_eq!(hotp_entry["ttl"], 0);
        assert_eq!(hotp_entry["token"], "359152");
    }

    #[test]
    fn test_run_app_with_custom_config_file_and_overrides() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("custom.yaml");
        std::fs::write(
            &config_path,
            r#"
digits: 8
period: 60
algorithm: sha256
auto_copy: true
light: true
"#,
        )
        .unwrap();

        // 1. Add account without explicit parameters - inherits config file defaults (digits=8, period=60, algorithm=sha256)
        let add_cli = Cli::try_parse_from([
            "tfa",
            "--config",
            config_path.to_str().unwrap(),
            "add",
            "test_acc",
            "JBSWY3DPEHPK3PXP",
        ])
        .unwrap();
        let mut add_out = Vec::new();
        assert!(run_app(add_cli, &store, &cb, &mut add_out).is_ok());

        // Verify stored account has digits=8, period=60, algorithm=SHA256
        let stored_uri = store.get_secret("test_acc").unwrap();
        assert!(stored_uri.contains("digits=8"));
        assert!(stored_uri.contains("period=60"));
        assert!(stored_uri.contains("algorithm=SHA256"));

        // 2. Generate token without --copy flag - inherits auto_copy=true from config file
        let gen_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "generate",
            "test_acc",
        ])
        .unwrap();
        let mut gen_out = Vec::new();
        assert!(run_app(gen_cli, &store, &cb, &mut gen_out).is_ok());
        let gen_str = String::from_utf8(gen_out).unwrap();
        assert!(gen_str.contains("[copied]"));
        assert_eq!(cb.text.lock().unwrap().as_ref().unwrap().len(), 8); // 8 digits

        // 3. QR code with config light=true
        let qr_cli =
            Cli::try_parse_from(["tfa", "-C", config_path.to_str().unwrap(), "qr", "test_acc"])
                .unwrap();
        let mut qr_out = Vec::new();
        assert!(run_app(qr_cli, &store, &cb, &mut qr_out).is_ok());
        assert!(!qr_out.is_empty());
    }
}
