use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::collections::BTreeMap;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

mod audit;
mod auth;
mod backup;
mod clipboard;
mod config;
mod import;
mod namespace;
mod qr;
mod storage;
mod totp;
mod twofas;
mod watch;

use clipboard::{Clipboard, ClipboardProvider, DispatchingClipboard};
use config::ConfigFile;
use namespace::NamespaceStore;
use storage::{AccountStore, AgeStore, AutoStore, FileStore, KeyringStore};
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
    /// Use age-encrypted file storage (~/.local/share/tfa/credentials.age)
    Age,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "tfa")]
#[command(author, version = env!("PROJECT_VERSION"), about = "CLI Two-Factor Authentication (2FA) tool", long_about = None)]
pub struct Cli {
    /// Path to configuration file (TOML, YAML, or JSON)
    #[arg(short = 'C', long, global = true, env = "TFA_CONFIG")]
    pub config: Option<PathBuf>,

    /// Credential storage backend
    #[arg(long, env = "TFA_STORAGE", global = true)]
    pub storage: Option<StorageBackend>,

    /// Path to age key file for age-encrypted storage
    #[arg(long, env = "TFA_AGE_KEY_FILE", global = true)]
    pub age_key: Option<PathBuf>,

    /// Path to SSH key file for age-encrypted storage
    #[arg(
        long,
        env = "TFA_SSH_KEY_FILE",
        visible_alias = "age-ssh-key",
        visible_alias = "ssh-key-file",
        global = true
    )]
    pub ssh_key: Option<PathBuf>,

    /// Clipboard copy provider (system or osc52)
    #[arg(long, env = "TFA_CLIPBOARD", global = true)]
    pub clipboard: Option<ClipboardProvider>,

    /// Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)
    #[arg(long, env = "TFA_CLIP_COMMAND", global = true)]
    pub clip_command: Option<String>,

    /// Target namespace for account scoping (e.g. personal, work, home)
    #[arg(short = 'n', long, env = "TFA_NAMESPACE", global = true)]
    pub namespace: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Generate a 2FA token (TOTP, Steam, or HOTP) for an account
    #[command(alias = "gen")]
    Generate {
        /// Name of the account (interactively prompted via fuzzy search if omitted and --secret not supplied)
        account: Option<String>,
        /// Generate token instantly from secret, URI, or "-" (stdin) without saving to storage
        #[arg(long)]
        secret: Option<String>,
        /// Copy generated token to clipboard
        #[arg(short, long, num_args = 0..=1, default_missing_value = "true")]
        copy: Option<bool>,
        /// Watch and continuously refresh code with progress bar
        #[arg(short, long)]
        watch: bool,
        /// Counter value for counter-based (HOTP) token generation
        #[arg(long, num_args = 0..=1)]
        counter: Option<Option<u64>>,
        /// Number of digits in generated code (e.g. 5, 6, 8)
        #[arg(short, long)]
        digits: Option<u8>,
        /// Time step period in seconds (e.g. 30 or 60)
        #[arg(short, long)]
        period: Option<u64>,
        /// Hash algorithm (sha1, sha256, sha512, steam)
        #[arg(short, long)]
        algorithm: Option<totp_rs::Algorithm>,
        /// Preview upcoming token for next time step
        #[arg(long, conflicts_with = "watch", conflicts_with = "counter")]
        next: bool,
        /// Output generated token details as JSON
        #[arg(long)]
        json: bool,
    },
    /// Generate a 2FA token instantly from raw secret, URI, or stdin without storage
    #[command(alias = "ins")]
    Instant {
        /// TOTP/HOTP secret key, otpauth:// URI, or "-" to read from stdin
        secret: Option<String>,
        /// Explicit secret key or otpauth:// URI flag
        #[arg(long = "secret")]
        secret_flag: Option<String>,
        /// Decode secret and parameters from a QR code image
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
        /// Initial/current counter for counter-based (HOTP) token generation
        #[arg(long)]
        counter: Option<u64>,
        /// Copy generated token to clipboard
        #[arg(short, long, num_args = 0..=1, default_missing_value = "true")]
        copy: Option<bool>,
        /// Watch and continuously refresh code with progress bar
        #[arg(short, long)]
        watch: bool,
    },
    /// Live watch mode displaying continuous TOTP countdown with progress bar
    Watch {
        #[command(subcommand)]
        command: Option<WatchCommands>,
        /// Name of the account (interactively prompted via fuzzy search if omitted)
        account: Option<String>,
        /// Watch all accounts in the active namespace
        #[arg(short, long)]
        all: bool,
        /// Filter accounts by substring or glob pattern (used with watch all)
        #[arg(short, long)]
        filter: Option<String>,
        /// Copy token to clipboard on every cycle (single account only)
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
    /// Rename an account within the active namespace
    Rename {
        /// Current account name
        old_name: String,
        /// New account name
        new_name: String,
    },
    /// Move an account to a different namespace
    #[command(alias = "mv")]
    Move {
        /// Name of the account to move
        account: String,
        /// Destination namespace
        #[arg(short = 't', long = "to")]
        to: String,
    },
    /// Show account details and stored secret
    Show {
        /// Name of the account
        account: String,
        /// Print only the raw secret or URI
        #[arg(short, long)]
        raw: bool,
        /// Output account details and metadata as JSON
        #[arg(long, conflicts_with = "raw")]
        json: bool,
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
        /// Encrypt backup using GPG with optional recipient key fingerprint, ID, or email
        #[arg(
            long,
            visible_alias = "gpg-recipient",
            visible_alias = "gpg-key",
            value_name = "FINGERPRINT",
            num_args = 0..=1,
            conflicts_with = "unencrypted"
        )]
        gpg: Option<Option<String>>,
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
    /// Manage and isolate accounts across multi-tenant namespaces
    #[command(alias = "ns")]
    Namespace {
        #[command(subcommand)]
        command: NamespaceCommands,
    },
    /// Manage master password protection and unlock sessions
    #[command(alias = "pwd", alias = "pw")]
    Password {
        #[command(subcommand)]
        command: PasswordCommands,
    },
    /// Scan stored accounts and credentials file for security vulnerabilities
    Audit {
        /// Output audit report in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum PasswordCommands {
    /// Set master password and enable password protection
    #[command(alias = "set")]
    Add {
        /// Timeout in seconds before unlock expires (default: 300)
        #[arg(short, long)]
        timeout: Option<u64>,
        /// Master password (prompted securely if omitted)
        #[arg(long, env = "TFA_PASSWORD")]
        password: Option<String>,
    },
    /// Remove master password and disable password protection
    #[command(alias = "rm")]
    Remove {
        /// Current master password (prompted securely if omitted)
        #[arg(long, env = "TFA_PASSWORD")]
        password: Option<String>,
        /// Skip password verification prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Lock the active session immediately
    Lock,
    /// Unlock session for the configured timeout
    Unlock {
        /// Master password (prompted securely if omitted)
        #[arg(long, env = "TFA_PASSWORD")]
        password: Option<String>,
    },
    /// Show current password protection and session unlock status
    Status,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum WatchCommands {
    /// Watch all accounts in the active namespace
    All {
        /// Filter accounts by substring or glob pattern
        #[arg(short, long)]
        filter: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum NamespaceCommands {
    /// List all namespaces and indicate the active/default one
    List,
    /// Create a new namespace
    Create {
        /// Name of the new namespace
        name: String,
    },
    /// Rename an existing namespace, safely migrating all contained accounts
    Rename {
        /// Current namespace name
        old_name: String,
        /// New namespace name
        new_name: String,
    },
    /// Delete a namespace and its contained accounts
    Delete {
        /// Name of the namespace to delete
        name: String,
        /// Force deletion without interactive confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Switch and persist default active namespace in configuration file
    SetDefault {
        /// Name of the namespace to set as default
        name: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AccountStatus {
    pub account: String,
    pub token: String,
    pub ttl: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GenerateResult {
    pub account: String,
    pub token: String,
    pub ttl: u64,
    pub period: u64,
    pub algorithm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AccountDetails {
    pub account: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub digits: u8,
    pub algorithm: String,
    pub secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter: Option<u64>,
    pub live_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
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

/// Resolve instant secret from argument, stdin ("-"), pipe, environment variable, or interactive prompt.
pub fn resolve_instant_secret_with_reader<R: BufRead>(
    secret_arg: Option<String>,
    qr_arg: Option<&Path>,
    mut reader: R,
    is_terminal: bool,
) -> anyhow::Result<String> {
    if let Some(qr_path) = qr_arg {
        let decoded = qr::scan_qr_from_image(qr_path)?;
        let trimmed = decoded.trim().to_string();
        if trimmed.is_empty() {
            anyhow::bail!("QR code contained empty secret");
        }
        return Ok(trimmed);
    }

    if let Some(s) = secret_arg {
        if s == "-" {
            let mut buffer = String::new();
            reader.read_to_string(&mut buffer)?;
            let trimmed = buffer
                .lines()
                .map(|l| l.trim())
                .find(|l| !l.is_empty())
                .unwrap_or("")
                .to_string();
            if trimmed.is_empty() {
                anyhow::bail!("Secret read from stdin is empty");
            }
            return Ok(trimmed);
        } else {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                anyhow::bail!("Secret cannot be empty");
            }
            return Ok(trimmed);
        }
    }

    if let Ok(env_val) = std::env::var("TOTP_TOKEN").or_else(|_| std::env::var("TFA_SECRET")) {
        let trimmed = env_val.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if !is_terminal {
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;
        let trimmed = buffer
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("")
            .to_string();
        if trimmed.is_empty() {
            anyhow::bail!("Secret or URI is required (stdin is empty)");
        }
        return Ok(trimmed);
    }

    eprint!("Enter TOTP secret or URI: ");
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    let trimmed = buffer.trim().to_string();
    if trimmed.is_empty() {
        anyhow::bail!("Secret cannot be empty");
    }
    Ok(trimmed)
}

pub fn resolve_instant_secret(
    secret_arg: Option<String>,
    qr_arg: Option<&Path>,
) -> anyhow::Result<String> {
    #[cfg(test)]
    {
        if std::env::var("TFA_TEST_INTERACTIVE").is_err() {
            // When executing unit tests, avoid locking/reading inherited process stdin
            return resolve_instant_secret_with_reader(
                secret_arg,
                qr_arg,
                io::Cursor::new(Vec::new()),
                false,
            );
        }
    }
    let stdin = io::stdin();
    let is_term = stdin.is_terminal();
    resolve_instant_secret_with_reader(secret_arg, qr_arg, stdin.lock(), is_term)
}

/// Format TOTP token line with validity countdown and optional next token preview
pub fn format_totp_output(code: &str, ttl: u64, next_code: Option<&str>, copied: bool) -> String {
    let mut line = format!("{code} ({ttl}s remaining)");
    if let Some(next) = next_code {
        if ttl <= 5 {
            line.push_str(" [expiring soon]");
        }
        line.push_str(&format!(" (next: {next})"));
    }
    if copied {
        line.push_str(" [copied]");
    }
    line
}

#[derive(Default, Clone, Copy)]
struct EmitOptions {
    counter: Option<u64>,
    next: bool,
    json: bool,
    should_copy: bool,
}

fn emit_instant_token(
    account_name: &str,
    engine: &TfaTotp,
    opts: EmitOptions,
    clipboard: &dyn Clipboard,
    out_writer: &mut dyn Write,
) -> anyhow::Result<()> {
    if engine.is_hotp() {
        if opts.next {
            anyhow::bail!("--next is only supported for time-based (TOTP) accounts");
        }
        let c = opts.counter.or_else(|| engine.counter()).unwrap_or(0);
        let code = engine.generate_at_counter(c)?;
        if opts.should_copy {
            clipboard.set_text(&code)?;
        }
        if opts.json {
            let res = GenerateResult {
                account: account_name.to_string(),
                token: code,
                ttl: 0,
                period: 0,
                algorithm: engine.algorithm().to_string(),
                next_token: None,
                counter: Some(c),
            };
            writeln!(out_writer, "{}", serde_json::to_string_pretty(&res)?)?;
        } else if opts.should_copy {
            writeln!(out_writer, "{code} (counter: {c}) [copied]")?;
        } else {
            writeln!(out_writer, "{code} (counter: {c})")?;
        }
    } else {
        if opts.counter.is_some() {
            anyhow::bail!("--counter can only be used with HOTP accounts");
        }
        let code = engine.generate_current();
        let ttl = engine.ttl();
        let next_code = if opts.next {
            Some(engine.generate_next()?)
        } else {
            None
        };
        if opts.should_copy {
            clipboard.set_text(&code)?;
        }
        if opts.json {
            let res = GenerateResult {
                account: account_name.to_string(),
                token: code,
                ttl,
                period: engine.step(),
                algorithm: engine.algorithm().to_string(),
                next_token: next_code,
                counter: None,
            };
            writeln!(out_writer, "{}", serde_json::to_string_pretty(&res)?)?;
        } else {
            let line = format_totp_output(&code, ttl, next_code.as_deref(), opts.should_copy);
            writeln!(out_writer, "{line}")?;
        }
    }
    Ok(())
}

pub fn run_app(
    mut cli: Cli,
    store: &dyn AccountStore,
    clipboard: &dyn Clipboard,
    out_writer: &mut dyn Write,
) -> anyhow::Result<()> {
    let (resolved_cfg, _) = config::resolve_configuration(
        cli.config.as_deref(),
        cli.storage,
        cli.clipboard,
        cli.clip_command.clone(),
        cli.namespace.clone(),
    )?;

    // If the subcommand is Namespace management, operate on the root store directly
    if let Some(Commands::Namespace {
        command: ref ns_cmd,
    }) = cli.command
    {
        match ns_cmd {
            NamespaceCommands::List => {
                let mut namespaces = NamespaceStore::list_all_namespaces(store)?;
                // Ensure default namespace exists in list
                if !namespaces.contains(&resolved_cfg.namespace) {
                    namespaces.push(resolved_cfg.namespace.clone());
                    namespaces.sort();
                }
                writeln!(out_writer, "{:<20} STATUS", "NAMESPACE")?;
                for ns in namespaces {
                    let is_active = ns == resolved_cfg.namespace;
                    let status = if is_active { "(active)" } else { "" };
                    writeln!(out_writer, "{:<20} {}", ns, status)?;
                }
                return Ok(());
            }
            NamespaceCommands::Create { name } => {
                let norm = namespace::normalize_namespace(name);
                writeln!(out_writer, "Namespace '{norm}' created.")?;
                return Ok(());
            }
            NamespaceCommands::Rename { old_name, new_name } => {
                let migrated = NamespaceStore::rename_namespace(store, old_name, new_name)?;
                writeln!(
                    out_writer,
                    "Renamed namespace '{old_name}' to '{new_name}' ({migrated} accounts migrated)."
                )?;
                return Ok(());
            }
            NamespaceCommands::Delete { name, force } => {
                let norm = namespace::normalize_namespace(name);
                if !force {
                    #[cfg(test)]
                    {
                        if std::env::var("TFA_TEST_INTERACTIVE").is_err() {
                            anyhow::bail!(
                                "Cannot delete namespace without --force in non-interactive mode"
                            );
                        }
                    }
                    // Check if non-interactive environment
                    if !io::stdin().is_terminal() {
                        anyhow::bail!(
                            "Cannot delete namespace without --force in non-interactive mode"
                        );
                    }
                    eprint!("Are you sure you want to delete namespace '{norm}' and all its accounts? (y/N): ");
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    if !input.trim().eq_ignore_ascii_case("y") {
                        writeln!(out_writer, "Namespace deletion cancelled.")?;
                        return Ok(());
                    }
                }
                let deleted = NamespaceStore::delete_namespace(store, &norm)?;
                writeln!(
                    out_writer,
                    "Namespace '{norm}' deleted ({deleted} accounts removed)."
                )?;
                return Ok(());
            }
            NamespaceCommands::SetDefault { name } => {
                let norm = namespace::normalize_namespace(name);
                let config_path =
                    ConfigFile::set_default_namespace_in_file(cli.config.as_deref(), &norm)?;
                writeln!(
                    out_writer,
                    "Default namespace set to '{norm}' in '{}'.",
                    config_path.display()
                )?;
                return Ok(());
            }
        }
    }

    // If the subcommand is Password management, handle it directly
    if let Some(Commands::Password {
        command: ref pwd_cmd,
    }) = cli.command
    {
        match pwd_cmd {
            PasswordCommands::Add { timeout, password } => {
                let target_timeout = timeout.unwrap_or(resolved_cfg.password_timeout);
                // If a password record already exists, ensure authorized
                if let Some(existing_record) = auth::get_password_record()? {
                    let token = auth::compute_auth_token(&existing_record);
                    if !auth::is_session_unlocked(&token, resolved_cfg.password_timeout) {
                        let current_pass =
                            get_password(None, false, "Enter current master password: ")?;
                        if !auth::verify_password(&current_pass, &existing_record) {
                            anyhow::bail!("Incorrect master password");
                        }
                    }
                }

                let new_pass = get_password(password.clone(), true, "Enter new master password: ")?;
                let record = auth::create_password_record(&new_pass)?;
                auth::save_password_record(&record)?;
                let token = auth::compute_auth_token(&record);
                auth::record_unlock(&token)?;

                let config_path = ConfigFile::set_password_config_in_file(
                    cli.config.as_deref(),
                    true,
                    Some(target_timeout),
                )?;
                writeln!(
                    out_writer,
                    "Master password set and unlocked for {}s (config: '{}').",
                    target_timeout,
                    config_path.display()
                )?;
                return Ok(());
            }
            PasswordCommands::Remove { password, force } => {
                if !force {
                    if let Some(existing_record) = auth::get_password_record()? {
                        let token = auth::compute_auth_token(&existing_record);
                        if !auth::is_session_unlocked(&token, resolved_cfg.password_timeout) {
                            let current_pass = get_password(
                                password.clone(),
                                false,
                                "Enter current master password: ",
                            )?;
                            if !auth::verify_password(&current_pass, &existing_record) {
                                anyhow::bail!("Incorrect master password");
                            }
                        }
                    }
                }
                auth::delete_password_record()?;
                let _ = auth::clear_session();
                let config_path =
                    ConfigFile::set_password_config_in_file(cli.config.as_deref(), false, None)?;
                writeln!(
                    out_writer,
                    "Master password removed and password protection disabled (config: '{}').",
                    config_path.display()
                )?;
                return Ok(());
            }
            PasswordCommands::Lock => {
                auth::clear_session()?;
                writeln!(out_writer, "Session locked.")?;
                return Ok(());
            }
            PasswordCommands::Unlock { password } => {
                let Some(record) = auth::get_password_record()? else {
                    anyhow::bail!("No master password configured. Run 'tfa password add' first.");
                };
                let token = auth::compute_auth_token(&record);
                if auth::is_session_unlocked(&token, resolved_cfg.password_timeout) {
                    let remaining =
                        auth::session_status(&token, resolved_cfg.password_timeout).unwrap_or(0);
                    writeln!(
                        out_writer,
                        "Session already unlocked ({}s remaining).",
                        remaining
                    )?;
                    return Ok(());
                }
                let pass = get_password(password.clone(), false, "Enter master password: ")?;
                if !auth::verify_password(&pass, &record) {
                    anyhow::bail!("Incorrect master password");
                }
                auth::record_unlock(&token)?;
                writeln!(
                    out_writer,
                    "Session unlocked for {}s.",
                    resolved_cfg.password_timeout
                )?;
                return Ok(());
            }
            PasswordCommands::Status => {
                let maybe_record = auth::get_password_record()?;
                let (status_str, is_configured) = match &maybe_record {
                    Some(record) => {
                        let token = auth::compute_auth_token(record);
                        match auth::session_status(&token, resolved_cfg.password_timeout) {
                            Some(rem) => (format!("Unlocked ({rem}s remaining)"), true),
                            None => ("Locked".to_string(), true),
                        }
                    }
                    None => ("Locked (no password configured)".to_string(), false),
                };

                writeln!(
                    out_writer,
                    "{:<22} {}",
                    "Password Protection:",
                    if resolved_cfg.password_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                )?;
                writeln!(
                    out_writer,
                    "{:<22} {}",
                    "Password Configured:",
                    if is_configured { "Yes" } else { "No" }
                )?;
                writeln!(
                    out_writer,
                    "{:<22} {}s",
                    "Timeout:", resolved_cfg.password_timeout
                )?;
                writeln!(out_writer, "{:<22} {}", "Session Status:", status_str)?;
                return Ok(());
            }
        }
    }

    // Normalize `tfa generate -` to instant generation from stdin
    if let Some(Commands::Generate {
        ref mut account,
        ref mut secret,
        ..
    }) = cli.command
    {
        if account.as_deref() == Some("-") && secret.is_none() {
            *secret = Some("-".to_string());
            *account = None;
        }
    }

    // If password protection is enabled, enforce unlock for protected commands
    if resolved_cfg.password_enabled {
        match &cli.command {
            Some(Commands::Completions { .. })
            | Some(Commands::Man)
            | Some(Commands::Instant { .. })
            | Some(Commands::Generate {
                secret: Some(_), ..
            })
            | None => {}
            _ => {
                ensure_unlocked(resolved_cfg.password_timeout)?;
            }
        }
    }

    // For all other commands, scope store access to the active namespace
    let root_store = store;
    let scoped_store = NamespaceStore::new(root_store, &resolved_cfg.namespace);
    let store = &scoped_store;

    match cli.command {
        Some(Commands::Generate {
            account,
            secret,
            copy,
            watch,
            counter,
            digits,
            period,
            algorithm,
            next,
            json,
        }) => {
            if let Some(sec) = secret {
                let resolved_digits = digits.or(resolved_cfg.digits);
                let resolved_period = period.or(resolved_cfg.period);
                let resolved_algorithm = algorithm.or(resolved_cfg.algorithm);
                let secret_val = resolve_instant_secret(Some(sec), None)?;
                let c_val = counter.flatten();
                let should_copy = copy.unwrap_or(resolved_cfg.auto_copy);
                let acct_name = account.as_deref().unwrap_or("instant");

                let engine = TfaTotp::build(
                    acct_name,
                    &secret_val,
                    resolved_digits,
                    resolved_period,
                    resolved_algorithm,
                    c_val,
                )?;

                if watch {
                    if engine.is_hotp() {
                        anyhow::bail!(
                            "Watch mode is not supported for counter-based (HOTP) accounts"
                        );
                    }
                    watch::watch_engine(
                        acct_name,
                        &engine,
                        clipboard,
                        should_copy,
                        out_writer,
                        None,
                    )?;
                } else {
                    emit_instant_token(
                        acct_name,
                        &engine,
                        EmitOptions {
                            counter: c_val,
                            next,
                            json,
                            should_copy,
                        },
                        clipboard,
                        out_writer,
                    )?;
                }
                return Ok(());
            }

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
                    if next {
                        anyhow::bail!("--next is only supported for time-based (TOTP) accounts");
                    }
                    let current_c = match counter {
                        Some(Some(c)) => c,
                        _ => engine.counter().unwrap_or(0),
                    };
                    let code = engine.generate_at_counter(current_c)?;
                    engine.set_counter(current_c + 1);
                    store.set_secret(&acct, &engine.to_url()?)?;

                    if should_copy {
                        clipboard.set_text(&code)?;
                    }
                    if json {
                        let res = GenerateResult {
                            account: acct.clone(),
                            token: code,
                            ttl: 0,
                            period: 0,
                            algorithm: engine.algorithm().to_string(),
                            next_token: None,
                            counter: Some(current_c),
                        };
                        writeln!(out_writer, "{}", serde_json::to_string_pretty(&res)?)?;
                    } else if should_copy {
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
                    let next_code = if next {
                        Some(engine.generate_next()?)
                    } else {
                        None
                    };
                    if should_copy {
                        clipboard.set_text(&code)?;
                    }
                    if json {
                        let res = GenerateResult {
                            account: acct.clone(),
                            token: code,
                            ttl,
                            period: engine.step(),
                            algorithm: engine.algorithm().to_string(),
                            next_token: next_code,
                            counter: None,
                        };
                        writeln!(out_writer, "{}", serde_json::to_string_pretty(&res)?)?;
                    } else {
                        let line =
                            format_totp_output(&code, ttl, next_code.as_deref(), should_copy);
                        writeln!(out_writer, "{line}")?;
                    }
                }
            }
        }
        Some(Commands::Instant {
            secret,
            secret_flag,
            qr,
            digits,
            period,
            algorithm,
            counter,
            copy,
            watch,
        }) => {
            let secret_val = resolve_instant_secret(secret.or(secret_flag), qr.as_deref())?;
            let resolved_digits = digits.or(resolved_cfg.digits);
            let resolved_period = period.or(resolved_cfg.period);
            let resolved_algorithm = algorithm.or(resolved_cfg.algorithm);
            let should_copy = copy.unwrap_or(resolved_cfg.auto_copy);

            let engine = TfaTotp::build(
                "instant",
                &secret_val,
                resolved_digits,
                resolved_period,
                resolved_algorithm,
                counter,
            )?;

            if watch {
                if engine.is_hotp() {
                    anyhow::bail!("Watch mode is not supported for counter-based (HOTP) accounts");
                }
                watch::watch_engine("instant", &engine, clipboard, should_copy, out_writer, None)?;
            } else {
                emit_instant_token(
                    "instant",
                    &engine,
                    EmitOptions {
                        counter,
                        next: false,
                        json: false,
                        should_copy,
                    },
                    clipboard,
                    out_writer,
                )?;
            }
        }
        Some(Commands::Watch {
            command,
            account,
            all,
            filter,
            copy,
        }) => {
            let (is_all, cmd_filter) = match &command {
                Some(WatchCommands::All { filter: f }) => (true, f.as_deref()),
                None => (all || account.as_deref() == Some("all"), None),
            };

            if is_all {
                let resolved_filter = cmd_filter.or(filter.as_deref());
                watch::watch_all(store, out_writer, resolved_filter, None)?;
            } else {
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
                        #[cfg(test)]
                        {
                            if std::env::var("TFA_TEST_INTERACTIVE").is_err() {
                                anyhow::bail!(
                                    "TOTP secret is required in non-interactive environment"
                                );
                            }
                        }
                        if !io::stdin().is_terminal() {
                            anyhow::bail!("TOTP secret is required in non-interactive environment");
                        }
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
        Some(Commands::Rename { old_name, new_name }) => {
            NamespaceStore::rename_account(
                root_store,
                &resolved_cfg.namespace,
                &old_name,
                &new_name,
            )?;
            writeln!(out_writer, "Account '{old_name}' renamed to '{new_name}'.")?;
        }
        Some(Commands::Move { account, to }) => {
            let target_ns = namespace::normalize_namespace(&to);
            NamespaceStore::move_account(
                root_store,
                &resolved_cfg.namespace,
                &account,
                &target_ns,
            )?;
            writeln!(
                out_writer,
                "Account '{account}' moved to namespace '{target_ns}'."
            )?;
        }
        Some(Commands::Show { account, raw, json }) => {
            let secret = store.get_secret(&account)?;
            if raw {
                writeln!(out_writer, "{secret}")?;
            } else if json {
                let engine = TfaTotp::from_input(&account, &secret)?;
                let is_hotp = engine.is_hotp();
                let details = AccountDetails {
                    account: account.clone(),
                    account_type: engine.account_type().to_string(),
                    digits: engine.digits(),
                    algorithm: engine.algorithm().to_string(),
                    secret,
                    period: if is_hotp { None } else { Some(engine.step()) },
                    counter: if is_hotp {
                        Some(engine.counter().unwrap_or(0))
                    } else {
                        None
                    },
                    live_code: engine.generate_current(),
                    ttl: if is_hotp { None } else { Some(engine.ttl()) },
                };
                writeln!(out_writer, "{}", serde_json::to_string_pretty(&details)?)?;
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
            gpg,
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
            } else if let Some(gpg_opt) = gpg {
                let recipient = match gpg_opt {
                    Some(fp) => {
                        let trimmed = fp.trim();
                        if trimmed.is_empty() {
                            anyhow::bail!("GPG key fingerprint cannot be empty");
                        }
                        trimmed.to_string()
                    }
                    None => resolved_cfg.gpg_fingerprint.clone().ok_or_else(|| {
                        anyhow::anyhow!(
                            "GPG key fingerprint is required (specify via --gpg <FINGERPRINT>, TFA_GPG_FINGERPRINT, or gpg_fingerprint in config)"
                        )
                    })?,
                };
                backup::encrypt_backup_gpg(json_str.as_bytes(), &recipient)?
            } else if password.is_none()
                && std::env::var("TFA_PASSWORD")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .is_none()
                && resolved_cfg.gpg_fingerprint.is_some()
            {
                let recipient = resolved_cfg.gpg_fingerprint.as_deref().unwrap();
                backup::encrypt_backup_gpg(json_str.as_bytes(), recipient)?
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
                    || file_or_payload.ends_with(".2fas")
                    || file_or_payload.ends_with(".gpg")
                    || file_or_payload.ends_with(".asc")
                    || file_or_payload.ends_with(".pgp")
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
            } else if twofas::is_encrypted_2fas_backup(&content) {
                let pass = get_passphrase(password, false)?;
                twofas::decrypt_2fas_backup(&content, &pass)?
            } else if backup::is_gpg_backup(&content) {
                let decrypted = backup::decrypt_backup_gpg(&content)?;
                String::from_utf8(decrypted).context("Decrypted GPG backup is not valid UTF-8")?
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
        Some(Commands::Namespace { .. }) => unreachable!("Handled above"),
        Some(Commands::Password { .. }) => unreachable!("Handled above"),
        Some(Commands::Audit { json }) => {
            let default_path = FileStore::default_path().ok();
            let report = audit::run_security_audit(root_store, default_path.as_deref())?;
            if json {
                let json_str = serde_json::to_string_pretty(&report)?;
                writeln!(out_writer, "{json_str}")?;
            } else {
                let text = audit::format_audit_report(&report);
                write!(out_writer, "{text}")?;
            }
        }
        None => {
            writeln!(out_writer, "Use --help for usage instructions.")?;
        }
    }
    Ok(())
}

fn get_password(password: Option<String>, confirm: bool, prompt: &str) -> anyhow::Result<String> {
    if let Some(p) = password {
        if p.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }
        return Ok(p);
    }

    if let Ok(env_pass) = std::env::var("TFA_PASSWORD") {
        if !env_pass.is_empty() {
            return Ok(env_pass);
        }
    }

    #[cfg(test)]
    {
        if std::env::var("TFA_TEST_INTERACTIVE").is_err() {
            anyhow::bail!("Password is required in non-interactive environment");
        }
    }

    if !io::stdin().is_terminal() {
        anyhow::bail!("Password is required in non-interactive environment");
    }

    let pass = rpassword::prompt_password(prompt)?;
    if pass.is_empty() {
        anyhow::bail!("Password cannot be empty");
    }

    if confirm {
        let confirm_pass = rpassword::prompt_password("Confirm master password: ")?;
        if pass != confirm_pass {
            anyhow::bail!("Passwords do not match");
        }
    }

    Ok(pass)
}

fn ensure_unlocked(timeout: u64) -> anyhow::Result<()> {
    let Some(record) = auth::get_password_record()? else {
        anyhow::bail!(
            "Password protection is enabled in configuration, but no master password has been configured. Run 'tfa password add' to set one."
        );
    };

    let token = auth::compute_auth_token(&record);
    if auth::is_session_unlocked(&token, timeout) {
        return Ok(());
    }

    let pass = get_password(None, false, "Enter master password: ")?;
    if !auth::verify_password(&pass, &record) {
        anyhow::bail!("Incorrect master password");
    }

    auth::record_unlock(&token)?;
    Ok(())
}

fn get_passphrase(password: Option<String>, confirm: bool) -> anyhow::Result<String> {
    get_password(password, confirm, "Enter master passphrase: ")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let (resolved_cfg, loaded_cfg) = config::resolve_configuration(
        cli.config.as_deref(),
        cli.storage,
        cli.clipboard,
        cli.clip_command.clone(),
        cli.namespace.clone(),
    )?;

    let clipboard =
        DispatchingClipboard::new(resolved_cfg.clipboard, resolved_cfg.clip_command.clone());

    let store: Box<dyn AccountStore> = match resolved_cfg.storage {
        StorageBackend::Auto => Box::new(AutoStore::default_auto()?),
        StorageBackend::Keyring => Box::new(KeyringStore::default()),
        StorageBackend::File => Box::new(FileStore::new(FileStore::default_path()?)),
        StorageBackend::Age => {
            let age_key = cli
                .age_key
                .clone()
                .or_else(|| loaded_cfg.as_ref().and_then(|c| c.age_key.clone()));
            let ssh_key = cli
                .ssh_key
                .clone()
                .or_else(|| std::env::var("TFA_SSH_KEY").ok().map(PathBuf::from))
                .or_else(|| std::env::var("TFA_AGE_SSH_KEY").ok().map(PathBuf::from))
                .or_else(|| loaded_cfg.as_ref().and_then(|c| c.ssh_key.clone()));
            Box::new(AgeStore::new(AgeStore::default_path()?, age_key, ssh_key))
        }
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

        let cli_age = Cli::try_parse_from([
            "tfa",
            "--storage",
            "age",
            "--age-key",
            "/path/to/key.txt",
            "--ssh-key",
            "/path/to/id_ed25519",
            "list",
        ])
        .unwrap();
        assert_eq!(cli_age.storage, Some(StorageBackend::Age));
        assert_eq!(cli_age.age_key, Some(PathBuf::from("/path/to/key.txt")));
        assert_eq!(cli_age.ssh_key, Some(PathBuf::from("/path/to/id_ed25519")));

        let cli_age_alias =
            Cli::try_parse_from(["tfa", "--age-ssh-key", "/path/to/id_rsa", "list"]).unwrap();
        assert_eq!(
            cli_age_alias.ssh_key,
            Some(PathBuf::from("/path/to/id_rsa"))
        );
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
                ..
            }) => {
                assert_eq!(account, Some("github".to_string()));
                assert_eq!(copy, None);
                assert!(!watch);
                assert_eq!(counter, None);
            }
            _ => panic!("Expected generate subcommand"),
        }

        // Test alias "gen"
        let cli_alias = Cli::try_parse_from(["tfa", "gen", "github"]).unwrap();
        match cli_alias.command {
            Some(Commands::Generate { account, .. }) => {
                assert_eq!(account, Some("github".to_string()));
            }
            _ => panic!("Expected generate subcommand via gen alias"),
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
                ..
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
                ..
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
            Some(Commands::Watch {
                account,
                copy,
                command,
                all,
                ..
            }) => {
                assert_eq!(account, Some("github".to_string()));
                assert_eq!(copy, Some(true));
                assert_eq!(command, None);
                assert!(!all);
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
    fn test_format_totp_output() {
        // Without next
        assert_eq!(
            format_totp_output("123456", 20, None, false),
            "123456 (20s remaining)"
        );
        assert_eq!(
            format_totp_output("123456", 20, None, true),
            "123456 (20s remaining) [copied]"
        );

        // With next, ttl > 5
        assert_eq!(
            format_totp_output("123456", 20, Some("654321"), false),
            "123456 (20s remaining) (next: 654321)"
        );
        assert_eq!(
            format_totp_output("123456", 20, Some("654321"), true),
            "123456 (20s remaining) (next: 654321) [copied]"
        );

        // With next, ttl <= 5 (countdown warning indicator)
        assert_eq!(
            format_totp_output("123456", 5, Some("654321"), false),
            "123456 (5s remaining) [expiring soon] (next: 654321)"
        );
        assert_eq!(
            format_totp_output("123456", 2, Some("654321"), true),
            "123456 (2s remaining) [expiring soon] (next: 654321) [copied]"
        );
        assert_eq!(
            format_totp_output("123456", 0, Some("654321"), false),
            "123456 (0s remaining) [expiring soon] (next: 654321)"
        );
    }

    #[test]
    fn test_cli_parse_generate_next() {
        let cli = Cli::try_parse_from(["tfa", "generate", "github", "--next"]).unwrap();
        match cli.command {
            Some(Commands::Generate { account, next, .. }) => {
                assert_eq!(account, Some("github".to_string()));
                assert!(next);
            }
            _ => panic!("Expected generate subcommand with next"),
        }

        // Conflicts with --watch
        assert!(Cli::try_parse_from(["tfa", "generate", "github", "--next", "-w"]).is_err());

        // Conflicts with --counter
        assert!(
            Cli::try_parse_from(["tfa", "generate", "github", "--next", "--counter", "1"]).is_err()
        );
    }

    #[test]
    fn test_run_app_generate_next_flow() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let add_cli =
            Cli::try_parse_from(["tfa", "add", "test_acc", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"])
                .unwrap();
        assert!(run_app(add_cli, &store, &cb, &mut out).is_ok());

        // Generate with --next
        let mut gen_out = Vec::new();
        let gen_cli = Cli::try_parse_from(["tfa", "generate", "test_acc", "--next"]).unwrap();
        assert!(run_app(gen_cli, &store, &cb, &mut gen_out).is_ok());
        let gen_str = String::from_utf8(gen_out).unwrap();
        assert!(gen_str.contains("remaining)"));
        assert!(gen_str.contains("(next: "));

        // Generate with --next and --copy
        let mut copy_out = Vec::new();
        let copy_cli =
            Cli::try_parse_from(["tfa", "generate", "test_acc", "--next", "-c"]).unwrap();
        assert!(run_app(copy_cli, &store, &cb, &mut copy_out).is_ok());
        let copy_str = String::from_utf8(copy_out).unwrap();
        assert!(copy_str.contains("remaining)"));
        assert!(copy_str.contains("(next: "));
        assert!(copy_str.contains("[copied]"));

        let copied = cb.text.lock().unwrap().clone();
        assert!(copied.is_some());
        let code = copied.unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        assert_eq!(copy_str.split_whitespace().next().unwrap(), code);

        // Generate with --json
        let mut json_out = Vec::new();
        let json_cli = Cli::try_parse_from(["tfa", "generate", "test_acc", "--json"]).unwrap();
        assert!(run_app(json_cli, &store, &cb, &mut json_out).is_ok());
        let json_str = String::from_utf8(json_out).unwrap();
        let res: GenerateResult = serde_json::from_str(&json_str).unwrap();
        assert_eq!(res.account, "test_acc");
        assert_eq!(res.token.len(), 6);
        assert!(res.ttl > 0 && res.ttl <= 30);
        assert_eq!(res.period, 30);
        assert_eq!(res.algorithm, "SHA1");
        assert!(res.next_token.is_none());
        assert!(res.counter.is_none());

        // Generate with --json and --next
        let mut json_next_out = Vec::new();
        let json_next_cli =
            Cli::try_parse_from(["tfa", "generate", "test_acc", "--json", "--next"]).unwrap();
        assert!(run_app(json_next_cli, &store, &cb, &mut json_next_out).is_ok());
        let json_next_str = String::from_utf8(json_next_out).unwrap();
        let res_next: GenerateResult = serde_json::from_str(&json_next_str).unwrap();
        assert_eq!(res_next.account, "test_acc");
        assert!(res_next.next_token.is_some());
        assert_eq!(res_next.next_token.as_ref().unwrap().len(), 6);
    }

    #[test]
    fn test_run_app_generate_next_instant() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let cli = Cli::try_parse_from([
            "tfa",
            "generate",
            "--secret",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "--next",
        ])
        .unwrap();
        assert!(run_app(cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("remaining)"));
        assert!(out_str.contains("(next: "));
    }

    #[test]
    fn test_run_app_generate_next_hotp_fails() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

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

        let gen_cli = Cli::try_parse_from(["tfa", "generate", "my_hotp", "--next"]).unwrap();
        let res = run_app(gen_cli, &store, &cb, &mut out);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("only supported for time-based"));
    }

    #[test]
    fn test_cli_parse_watch_all() {
        let cli1 = Cli::try_parse_from(["tfa", "watch", "all"]).unwrap();
        match cli1.command {
            Some(Commands::Watch {
                command,
                account,
                all,
                ..
            }) => {
                assert_eq!(command, Some(WatchCommands::All { filter: None }));
                assert_eq!(account, None);
                assert!(!all);
            }
            _ => panic!("Expected watch all subcommand"),
        }

        let cli1_filter = Cli::try_parse_from(["tfa", "watch", "all", "--filter", "git*"]).unwrap();
        match cli1_filter.command {
            Some(Commands::Watch { command, .. }) => {
                assert_eq!(
                    command,
                    Some(WatchCommands::All {
                        filter: Some("git*".to_string())
                    })
                );
            }
            _ => panic!("Expected watch all with filter"),
        }

        let cli2 = Cli::try_parse_from(["tfa", "watch", "--all", "-f", "aws"]).unwrap();
        match cli2.command {
            Some(Commands::Watch {
                command,
                account,
                all,
                filter,
                ..
            }) => {
                assert_eq!(command, None);
                assert_eq!(account, None);
                assert!(all);
                assert_eq!(filter, Some("aws".to_string()));
            }
            _ => panic!("Expected watch --all"),
        }

        let cli3 = Cli::try_parse_from(["tfa", "watch", "-a"]).unwrap();
        match cli3.command {
            Some(Commands::Watch { all, .. }) => {
                assert!(all);
            }
            _ => panic!("Expected watch -a"),
        }

        let cli4 = Cli::try_parse_from(["tfa", "-n", "work", "watch", "all"]).unwrap();
        assert_eq!(cli4.namespace, Some("work".to_string()));
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
            Some(Commands::Show { account, raw, json }) => {
                assert_eq!(account, "github");
                assert!(!raw);
                assert!(!json);
            }
            _ => panic!("Expected show subcommand"),
        }

        let cli_raw = Cli::try_parse_from(["tfa", "show", "github", "-r"]).unwrap();
        match cli_raw.command {
            Some(Commands::Show { account, raw, json }) => {
                assert_eq!(account, "github");
                assert!(raw);
                assert!(!json);
            }
            _ => panic!("Expected show subcommand"),
        }

        let cli_json = Cli::try_parse_from(["tfa", "show", "github", "--json"]).unwrap();
        match cli_json.command {
            Some(Commands::Show { account, raw, json }) => {
                assert_eq!(account, "github");
                assert!(!raw);
                assert!(json);
            }
            _ => panic!("Expected show subcommand"),
        }

        // Conflicts with --raw
        assert!(Cli::try_parse_from(["tfa", "show", "github", "-r", "--json"]).is_err());
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

        // JSON show
        let mut json_out = Vec::new();
        let show_json = Cli::try_parse_from(["tfa", "show", "my_acc", "--json"]).unwrap();
        assert!(run_app(show_json, &store, &cb, &mut json_out).is_ok());
        let json_str = String::from_utf8(json_out).unwrap();
        let details: AccountDetails = serde_json::from_str(&json_str).unwrap();
        assert_eq!(details.account, "my_acc");
        assert_eq!(details.account_type, "TOTP");
        assert_eq!(details.digits, 6);
        assert_eq!(details.period, Some(30));
        assert_eq!(details.algorithm, "SHA1");
        assert_eq!(details.secret, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        assert_eq!(details.live_code.len(), 6);
        assert!(details.ttl.is_some());
        assert_eq!(details.counter, None);

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
                gpg,
            }) => {
                assert!(file.is_none());
                assert!(!unencrypted);
                assert!(password.is_none());
                assert!(gpg.is_none());
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
                gpg,
            }) => {
                assert_eq!(file, Some(PathBuf::from("backup.enc")));
                assert!(unencrypted);
                assert_eq!(password, Some("pass123".to_string()));
                assert!(gpg.is_none());
            }
            _ => panic!("Expected export subcommand"),
        }

        let cli_gpg =
            Cli::try_parse_from(["tfa", "export", "backup.gpg", "--gpg", "ABCD1234"]).unwrap();
        match cli_gpg.command {
            Some(Commands::Export { file, gpg, .. }) => {
                assert_eq!(file, Some(PathBuf::from("backup.gpg")));
                assert_eq!(gpg, Some(Some("ABCD1234".to_string())));
            }
            _ => panic!("Expected export subcommand"),
        }

        let cli_gpg_alias =
            Cli::try_parse_from(["tfa", "export", "--gpg-recipient", "user@example.com"]).unwrap();
        match cli_gpg_alias.command {
            Some(Commands::Export { file, gpg, .. }) => {
                assert!(file.is_none());
                assert_eq!(gpg, Some(Some("user@example.com".to_string())));
            }
            _ => panic!("Expected export subcommand"),
        }

        let cli_gpg_default =
            Cli::try_parse_from(["tfa", "export", "backup.gpg", "--gpg"]).unwrap();
        match cli_gpg_default.command {
            Some(Commands::Export { file, gpg, .. }) => {
                assert_eq!(file, Some(PathBuf::from("backup.gpg")));
                assert_eq!(gpg, Some(None));
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
    fn test_run_app_export_gpg_cli_validation() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let temp_dir = tempfile::tempdir().unwrap();

        // 1. Error when --gpg is passed with empty key
        let mut err_out = Vec::new();
        let empty_gpg_cli = Cli::try_parse_from(["tfa", "export", "--gpg="]).unwrap();
        assert!(run_app(empty_gpg_cli, &store, &cb, &mut err_out).is_err());

        // 2. Error when --gpg is passed without recipient and without config fingerprint
        let no_cfg_path = temp_dir.path().join("empty_config.toml");
        std::fs::write(&no_cfg_path, "").unwrap();
        let missing_cfg_gpg = Cli::try_parse_from([
            "tfa",
            "-C",
            no_cfg_path.to_str().unwrap(),
            "export",
            "--gpg",
        ])
        .unwrap();
        assert!(run_app(missing_cfg_gpg, &store, &cb, &mut err_out).is_err());
    }

    #[test]
    fn test_run_app_export_and_import_gpg_flow() {
        let _guard = backup::GPG_TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if !backup::is_gpg_available_for_test() {
            eprintln!(
                "Skipping GPG integration test: GnuPG / gpg-agent not available or functional"
            );
            return;
        }
        let store1 = MemoryStore::new();
        let cb = MockClipboard::default();
        store1
            .set_secret("acc1", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")
            .unwrap();
        store1.set_secret("acc2", "JBSWY3DPEHPK3PXP").unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let gpg_dir = temp_dir.path().join("gnupg");
        std::fs::create_dir_all(&gpg_dir).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&gpg_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        std::env::set_var("GNUPGHOME", &gpg_dir);

        // Generate a temporary GPG key in isolated GNUPGHOME
        let status = std::process::Command::new("gpg")
            .args([
                "--batch",
                "--passphrase",
                "",
                "--quick-generate-key",
                "CLI Test <cli-test@example.com>",
                "default",
                "default",
                "never",
            ])
            .status()
            .unwrap();
        assert!(status.success());

        let list_out = std::process::Command::new("gpg")
            .args(["--list-keys", "--with-colons"])
            .output()
            .unwrap();
        let list_str = String::from_utf8_lossy(&list_out.stdout);
        let fpr = list_str
            .lines()
            .find(|l| l.starts_with("fpr:"))
            .and_then(|l| l.split(':').nth(9))
            .expect("Expected fingerprint in gpg --list-keys output")
            .to_string();

        // 1. Export with --gpg <FINGERPRINT>
        let backup_file = temp_dir.path().join("backup.gpg");
        let mut out = Vec::new();
        let export_cli = Cli::try_parse_from([
            "tfa",
            "export",
            backup_file.to_str().unwrap(),
            "--gpg",
            &fpr,
        ])
        .unwrap();
        assert!(run_app(export_cli, &store1, &cb, &mut out).is_ok());

        let file_contents = std::fs::read_to_string(&backup_file).unwrap();
        assert!(backup::is_gpg_backup(&file_contents));

        // 2. Import GPG-encrypted backup (auto-detected without flags)
        let store2 = MemoryStore::new();
        let mut import_out = Vec::new();
        let import_cli =
            Cli::try_parse_from(["tfa", "import", backup_file.to_str().unwrap()]).unwrap();
        assert!(run_app(import_cli, &store2, &cb, &mut import_out).is_ok());
        let import_str = String::from_utf8(import_out).unwrap();
        assert!(import_str.contains("Summary: 2 imported, 0 updated, 0 skipped."));
        assert_eq!(store2.list_accounts().unwrap().len(), 2);

        // 3. Export using default gpg_fingerprint from config file
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, format!("gpg_fingerprint = \"{fpr}\"\n")).unwrap();
        let mut cfg_out = Vec::new();
        let export_cfg_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "export",
            "--gpg",
        ])
        .unwrap();
        assert!(run_app(export_cfg_cli, &store1, &cb, &mut cfg_out).is_ok());
        let cfg_out_str = String::from_utf8(cfg_out).unwrap();
        assert!(backup::is_gpg_backup(&cfg_out_str));

        // 4. Error when --gpg is passed with empty or unconfigured key
        let mut err_out = Vec::new();
        let empty_gpg_cli = Cli::try_parse_from(["tfa", "export", "--gpg="]).unwrap();
        assert!(run_app(empty_gpg_cli, &store1, &cb, &mut err_out).is_err());

        let no_cfg_path = temp_dir.path().join("empty_config.toml");
        std::fs::write(&no_cfg_path, "").unwrap();
        let missing_cfg_gpg = Cli::try_parse_from([
            "tfa",
            "-C",
            no_cfg_path.to_str().unwrap(),
            "export",
            "--gpg",
        ])
        .unwrap();
        assert!(run_app(missing_cfg_gpg, &store1, &cb, &mut err_out).is_err());

        let _ = std::process::Command::new("gpgconf")
            .args(["--homedir", gpg_dir.to_str().unwrap(), "--kill", "all"])
            .status();
        std::env::remove_var("GNUPGHOME");
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
    fn test_run_app_import_2fas_unencrypted_file() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("backup.2fas");
        let payload = r#"{
            "schemaVersion": 4,
            "services": [
                {
                    "name": "GitHub",
                    "secret": "JBSWY3DPEHPK3PXP",
                    "otp": {
                        "account": "alice@example.com",
                        "digits": 6,
                        "period": 30,
                        "algorithm": "SHA1",
                        "tokenType": "TOTP"
                    }
                }
            ]
        }"#;
        std::fs::write(&file_path, payload).unwrap();

        let import_cli =
            Cli::try_parse_from(["tfa", "import", file_path.to_str().unwrap()]).unwrap();
        assert!(run_app(import_cli, &store, &cb, &mut out).is_ok());
        let out_str = String::from_utf8(out).unwrap();
        assert!(out_str.contains("Imported account 'GitHub:alice@example.com'"));

        let secret = store.get_secret("GitHub:alice@example.com").unwrap();
        let engine = TfaTotp::from_input("GitHub:alice@example.com", &secret).unwrap();
        assert_eq!(engine.generate_current().len(), 6);
    }

    #[test]
    fn test_run_app_import_2fas_encrypted_file() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("encrypted.2fas");
        let fixture = r#"{"services":[],"groups":[],"updatedAt":1708958781890,"schemaVersion":4,"appVersionCode":5000017,"appVersionName":"5.3.5","appOrigin":"android","servicesEncrypted":"HhVj7hNTJBdmidKVsCrgK0EafXu2aBshvz3KlBX0dIHHXA/GGCD5Ws8h3NE59DLXl3dgiPQgDYzCgTEYQIqgSXylmMh77odkAYQKDFWGL74etmdPulab480QbrJuUDfGOl6DtZynEn6WS/uyvJ5nrpEHp2UVAXCu6y7EIHuNgtwiY+O2UXEYX8zMKJWVqMeTZ4VyQCQvT9nTBbINJi8lyYIRmIup0cwPbh6JORpzV5yKWG0Jmbi/vdqXsFftYHjRmkwt/KsnjNmNWHTeztBOSZ4RlXpW24pEctUUqkiqz8HkMpOlOxkgR97MpxxoubN7gS/dqn8J6lAgBa0yWlf0QgqNVY5mqhSuMlfug2xTZDeG93jeiWQOqb2cs1bnEb5iUaUzpThw1P8XuzrmlByFQ3K7b71lXzyX6Rdqt05u4SIJMtc/I6CVqAta1fuQ1fcjufjcaIxglGOxKM5Fxy7gLp/Jwhl9Fin1RKC4JRd1h+siC87Lm9qA/WW7XmR81+wlhzlcXQ+xLxoccPl1CNhxibOswR0sQEhMPrOXLQk7Dy5KT9lxTIV0o67e9WgzbfOoIgQp2zxWeT3QjYrROivC2c6OsktUqgonFzzNTAxWxQtxOlUzS0BOmbn6t12KbJYE7Lv9fnv9wQJoKDU9QJai4fqFpcZeXnObN2VIt65QA7bumsr74PvgmS9Y+hu4xYhWEUkag0Q5YTpeLBiWCp71l0cx8uMrvoCw1JPphGnkjuFgVsBpcqKxHudsbi83fDVHBDv1HogO5cyMDDi4/h8N6Cdfz3xCsQg39X8RtvfFMf7kYSRmbWcNlqkmVHVWuIoxlXcsbAHw1LYRgYyodeCFDoR6NMmmP3I2mKAZfG+fXGnhHHb1+dwnNp/hyDFQ/RRFpSNMwAaU+/SUeYbE5yx2ux2dHNUJ2W7j19It1vdAE8xfQMl/qzxMODdq1L1D8lDyrzfwKT1S41wKDXjJBrx7EQAN/Eh5awBGEN+CWyce6l4wOOfvFfYx7iAmH7XDEzjcyRe3wsPlhqTZXr/VMsUKX//xONa2eSjunodKHekr/bI2xGV0qbisWtJmbEZBKkDhsrdUbPBSvADwSqJqlDIpqspljmm+xSpSNW2qPuRD5RIgXI4JkfjjBC9E2hgb+8otanTBI2y8i22GLkz+vr+JmqjDufLzLvvtO6Fj8SPf2B2pMgQNyRw8agUxXbdLlqhRpMPNlaxBKoU3lgitwGoQui7Va+r8kAtKdgZuC0EV/XOPKFY4KUDh+vXbjRsMQbf9xXHnwqkC5tkNk17r+4nVqoMeW4fTfXPNjn2v5euqa7bnUGKx3RhD6dleqH1mJLylyvla3g4EP+eI3sxW925sVK83j6AO8/zJz4zowqKAqDI1KiC1FnlU65TA2qMUU4t2YE1CwzGykWNtcHfjQDhGMOzpnhdhcIylcuJ/qdE2iVZ2ACA4R9VpHPxmoNuqlz4YbtGism6JSjOc43KxPblz+LOT+OXU1XYH1CuaB4Btrty0CG68OS10IXXKJ3UqiDubNfGZ0T0XH+yu87QJkIO0bpwoT45M+Sb9Zps8/zjE8dgVnsncf/7Y27wwemQJK5N4Ndju0/CCfixy4hNPpcGdqQsLRVdU44OLe+nblwA94wLv+SkJXrDYZqu8xbLVxd3Zbi/+nUs084oNSfMwsrq90qhjn3uKmtPTBMqi0IVxAqnmLhLHqEQcBCYFt+uxXJt6Btiehxs76LtMGnuT+t1N5kDpvnuXduXYqvm/lpR1W6rblaYyhnfTdT8S7zecM8BAQ5SN/Z284DkdMUVreYrInXUGc/uJHK8UswrpYfNonvtYpPP9Hc9qrkN/DtdsCmzUd4a1acexYjc31t0fBABl0cihW4yK4vwi77Z2mVyV+6eT2OVUaEGG/zFJtU0UIT/tN50gFXCP63pZu9RdYLD2isIDnlfluQO9gDgcp89KZcbkXB/2cafBypLq5r9RFKcacxY9zM8sEgZv5+p+WEEzP2v/Bk32d6zMJa8mcJcHdeUDsCqAzhsZ7yaJUwYpKqrWY1BUXszLBi2RbmqdVaqgdmYtj6sM9RaW+HHPbMTPwwe6N3MBcQpCFxMMXhY1ivmPzxoj5t/hMaDuDAT6HKT70KHr3ZCA9FqVi2VbEzg4bUBjMKXauZgzb6NZFGucSfV05wcB0Ao/MmBfmUhEb4QhLvhbPu3G4a10YVUsiNol0nypKlbnGsp93mvK7KW60EVOFiQbgsvicIeqFxgO2cZ/PJgbB5GJjPswGs6mMK07CEYJf5spaKBj0FK83NTW+8dg9D0C7ZKIB1huOtPa3rRUGC6RXl1p7BQy5ym9r6zpiTMNLoiJ5zkCUGrr4k88YjZnhcjr89T2T1hQo4qGoEIzKQfGzNobeHvBMj2wq63rTGfeX+uw6mqEPF0P3HG3JninzNQ+gVIWzei0VS1PM3X7dviehhHJm4QJLel5om/vOBtzqtOMf5MV79fmEDUx+FrxJSYiBu+glsB9R50eYaGjM0KDiwtPvDmXN8Haa0TAbegzx8fAFHd7NUoRrOR+p4dTY1Qs9Ut0/m3Ji3V5PxU6ra9GfJhsb6ABSHMauKR1d2jx6dl7hz+iVL2mtJwgERw+31jtktBZSPOcRK9Qsk6UvCHqPsMTcTFTOCxoZkFC5S4AVR/gMTxt+wG3DnYuGv3O2MIsOuVp3vk2vAeSRl8KEPHmwhNwmBPNcWVGNfIp/7rJfQwpwa5vjH5u7Hrr:xsCM/GAwNcyqrDcYodp58e6xxXl+cj0P+1Bh9mH4f7+UYKrQV4cpMAbQRPyNJz5CbsvSsFGYr+Ls1N+GyX6fp8LahIyovloySTRqQZzBI0VgKTKy1g7PlSSVjhedokyK5osUg6lUTimr29SGyvL4r/ornfkKygDZry8gHjyANX06mfxBK46+qomjsw5TErS0VlitPMJ1OWoh5/ZArEZBSczTGSOLjdQ3uMkQGOEUCJAd9wruBViN7td/0tmBAhzkG7EtrOJN7YNCGSLCiRoeLqS+unbaIOmUeKyn2AWd+jT/k4WcxIkHlYPRumy1DzS/REh6NUfagoO/1fPLMUYUug==:5UW4AuvcvsEi0jYe"}"#;
        std::fs::write(&file_path, fixture).unwrap();

        // 1. Dry run
        let mut dry_out = Vec::new();
        let dry_cli = Cli::try_parse_from([
            "tfa",
            "import",
            file_path.to_str().unwrap(),
            "--password",
            "example.com",
            "--dry-run",
        ])
        .unwrap();
        assert!(run_app(dry_cli, &store, &cb, &mut dry_out).is_ok());
        let dry_str = String::from_utf8(dry_out).unwrap();
        assert!(dry_str.contains("[dry-run] Would import account 'Deno:Mason'"));
        assert!(dry_str.contains("[dry-run] Summary: 5 to import, 0 to update, 0 to skip."));
        assert!(store.get_secret("Deno:Mason").is_err());

        // 2. Actual import
        let mut import_out = Vec::new();
        let import_cli = Cli::try_parse_from([
            "tfa",
            "import",
            file_path.to_str().unwrap(),
            "--password",
            "example.com",
        ])
        .unwrap();
        assert!(run_app(import_cli, &store, &cb, &mut import_out).is_ok());
        let import_str = String::from_utf8(import_out).unwrap();
        assert!(import_str.contains("Imported account 'Deno:Mason'"));
        assert!(import_str.contains("Summary: 5 imported, 0 updated, 0 skipped."));

        let accounts = store.list_accounts().unwrap();
        assert_eq!(accounts.len(), 5);

        // 3. Verify Steam token generation (Boeing:Sophia)
        let boeing_sec = store.get_secret("Boeing:Sophia").unwrap();
        let boeing = TfaTotp::from_input("Boeing:Sophia", &boeing_sec).unwrap();
        assert_eq!(boeing.account_type(), "Steam");
        assert_eq!(boeing.digits(), 5);
        let steam_code = boeing.generate_current();
        assert_eq!(steam_code.len(), 5);

        // 4. Verify HOTP token generation (Air Canada:Benjamin)
        let ac_sec = store.get_secret("Air Canada:Benjamin").unwrap();
        let ac = TfaTotp::from_input("Air Canada:Benjamin", &ac_sec).unwrap();
        assert!(ac.is_hotp());
        assert_eq!(ac.digits(), 8);

        // 5. Re-import without overwrite skips all 5
        let mut skip_out = Vec::new();
        let skip_cli = Cli::try_parse_from([
            "tfa",
            "import",
            file_path.to_str().unwrap(),
            "--password",
            "example.com",
            "--skip",
        ])
        .unwrap();
        assert!(run_app(skip_cli, &store, &cb, &mut skip_out).is_ok());
        let skip_str = String::from_utf8(skip_out).unwrap();
        assert!(skip_str.contains("Summary: 0 imported, 0 updated, 5 skipped."));

        // 6. Re-import with overwrite updates all 5
        let mut ow_out = Vec::new();
        let ow_cli = Cli::try_parse_from([
            "tfa",
            "import",
            file_path.to_str().unwrap(),
            "--password",
            "example.com",
            "--overwrite",
        ])
        .unwrap();
        assert!(run_app(ow_cli, &store, &cb, &mut ow_out).is_ok());
        let ow_str = String::from_utf8(ow_out).unwrap();
        assert!(ow_str.contains("Summary: 0 imported, 5 updated, 0 skipped."));
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
    fn test_watch_all_lifecycle() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // 1. Empty store run_app returns Ok and prints "No accounts configured."
        let mut out = Vec::new();
        let cli = Cli::try_parse_from(["tfa", "watch", "all"]).unwrap();
        let res = run_app(cli, &store, &cb, &mut out);
        assert!(res.is_ok());
        assert!(String::from_utf8(out)
            .unwrap()
            .contains("No accounts configured."));

        // 2. Namespace scoping with watch all
        let mut out_work = Vec::new();
        store.set_secret("work:corp", "JBSWY3DPEHPK3PXP").unwrap();
        store.set_secret("personal", "JBSWY3DPEHPK3PXP").unwrap();

        let cli_work = Cli::try_parse_from(["tfa", "-n", "other", "watch", "all"]).unwrap();
        let res_work = run_app(cli_work, &store, &cb, &mut out_work);
        assert!(res_work.is_ok());
        assert!(String::from_utf8(out_work)
            .unwrap()
            .contains("No accounts configured."));

        // 3. Directly test watch_all with max_ticks
        let mut out_watch = Vec::new();
        let res_watch = watch::watch_all(&store, &mut out_watch, None, Some(2));
        if res_watch.is_ok() {
            let out_str = String::from_utf8(out_watch).unwrap();
            assert!(out_str.contains("ACCOUNT"));
            assert!(out_str.contains("TOKEN"));
            assert!(out_str.contains("PROGRESS"));
            assert!(out_str.contains("personal"));
        }

        // 4. Test watch_all with filter
        let mut out_filter = Vec::new();
        let res_filter = watch::watch_all(&store, &mut out_filter, Some("person*"), Some(2));
        if res_filter.is_ok() {
            let out_str = String::from_utf8(out_filter).unwrap();
            assert!(out_str.contains("personal"));
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

    #[test]
    fn test_cli_parse_clip_command_flag() {
        let cli = Cli::try_parse_from(["tfa", "--clip-command", "wl-copy", "generate", "github"])
            .unwrap();
        assert_eq!(cli.clip_command.as_deref(), Some("wl-copy"));
    }

    #[test]
    fn test_cli_parse_namespace_flag_and_alias() {
        let cli = Cli::try_parse_from(["tfa", "-n", "work", "list"]).unwrap();
        assert_eq!(cli.namespace.as_deref(), Some("work"));

        let cli_ns = Cli::try_parse_from(["tfa", "ns", "list"]).unwrap();
        match cli_ns.command {
            Some(Commands::Namespace {
                command: NamespaceCommands::List,
            }) => {}
            _ => panic!("Expected ns alias to map to Namespace command"),
        }
    }

    #[test]
    fn test_run_app_namespace_management_and_isolation() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // 1. Add account in default namespace
        let add_def = Cli::try_parse_from(["tfa", "add", "github", "JBSWY3DPEHPK3PXP"]).unwrap();
        let mut out = Vec::new();
        assert!(run_app(add_def, &store, &cb, &mut out).is_ok());

        // 2. Add account in work namespace
        let add_work = Cli::try_parse_from([
            "tfa",
            "-n",
            "work",
            "add",
            "aws",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        ])
        .unwrap();
        let mut out_work = Vec::new();
        assert!(run_app(add_work, &store, &cb, &mut out_work).is_ok());

        // 3. List in default namespace only shows github
        let list_def = Cli::try_parse_from(["tfa", "list"]).unwrap();
        let mut list_def_out = Vec::new();
        assert!(run_app(list_def, &store, &cb, &mut list_def_out).is_ok());
        let list_str = String::from_utf8(list_def_out).unwrap();
        assert!(list_str.contains("github"));
        assert!(!list_str.contains("aws"));

        // 4. List in work namespace only shows aws
        let list_work = Cli::try_parse_from(["tfa", "-n", "work", "list"]).unwrap();
        let mut list_work_out = Vec::new();
        assert!(run_app(list_work, &store, &cb, &mut list_work_out).is_ok());
        let work_str = String::from_utf8(list_work_out).unwrap();
        assert!(work_str.contains("aws"));
        assert!(!work_str.contains("github"));

        // 5. Namespace list shows default and work
        let ns_list_cli = Cli::try_parse_from(["tfa", "ns", "list"]).unwrap();
        let mut ns_out = Vec::new();
        assert!(run_app(ns_list_cli, &store, &cb, &mut ns_out).is_ok());
        let ns_str = String::from_utf8(ns_out).unwrap();
        assert!(ns_str.contains("default"));
        assert!(ns_str.contains("work"));

        // 6. Namespace rename work -> office
        let ns_rename =
            Cli::try_parse_from(["tfa", "namespace", "rename", "work", "office"]).unwrap();
        let mut rename_out = Vec::new();
        assert!(run_app(ns_rename, &store, &cb, &mut rename_out).is_ok());

        // Now office has aws, work is empty
        let office_list = Cli::try_parse_from(["tfa", "-n", "office", "list"]).unwrap();
        let mut office_out = Vec::new();
        assert!(run_app(office_list, &store, &cb, &mut office_out).is_ok());
        assert!(String::from_utf8(office_out).unwrap().contains("aws"));

        // 7. Delete office namespace with --force
        let ns_del = Cli::try_parse_from(["tfa", "ns", "delete", "office", "--force"]).unwrap();
        let mut del_out = Vec::new();
        assert!(run_app(ns_del, &store, &cb, &mut del_out).is_ok());

        let office_list_after = Cli::try_parse_from(["tfa", "-n", "office", "list"]).unwrap();
        let mut office_after_out = Vec::new();
        assert!(run_app(office_list_after, &store, &cb, &mut office_after_out).is_ok());
        assert!(String::from_utf8(office_after_out)
            .unwrap()
            .contains("No accounts configured."));
    }

    #[test]
    fn test_cli_parse_password_commands() {
        // 1. password add
        let cli_add = Cli::try_parse_from([
            "tfa",
            "password",
            "add",
            "-t",
            "600",
            "--password",
            "secret123",
        ])
        .unwrap();
        match cli_add.command {
            Some(Commands::Password {
                command: PasswordCommands::Add { timeout, password },
            }) => {
                assert_eq!(timeout, Some(600));
                assert_eq!(password, Some("secret123".to_string()));
            }
            _ => panic!("Expected PasswordCommands::Add"),
        }

        // 2. password remove
        let cli_rm = Cli::try_parse_from(["tfa", "password", "remove", "-f"]).unwrap();
        match cli_rm.command {
            Some(Commands::Password {
                command: PasswordCommands::Remove { force, .. },
            }) => {
                assert!(force);
            }
            _ => panic!("Expected PasswordCommands::Remove"),
        }

        // 3. password lock
        let cli_lock = Cli::try_parse_from(["tfa", "password", "lock"]).unwrap();
        match cli_lock.command {
            Some(Commands::Password {
                command: PasswordCommands::Lock,
            }) => {}
            _ => panic!("Expected PasswordCommands::Lock"),
        }

        // 4. password status
        let cli_status = Cli::try_parse_from(["tfa", "password", "status"]).unwrap();
        match cli_status.command {
            Some(Commands::Password {
                command: PasswordCommands::Status,
            }) => {}
            _ => panic!("Expected PasswordCommands::Status"),
        }

        // 5. alias pw
        let cli_pw = Cli::try_parse_from(["tfa", "pw", "status"]).unwrap();
        match cli_pw.command {
            Some(Commands::Password {
                command: PasswordCommands::Status,
            }) => {}
            _ => panic!("Expected PasswordCommands::Status via pw alias"),
        }

        // 6. alias pwd
        let cli_pwd = Cli::try_parse_from(["tfa", "pwd", "status"]).unwrap();
        match cli_pwd.command {
            Some(Commands::Password {
                command: PasswordCommands::Status,
            }) => {}
            _ => panic!("Expected PasswordCommands::Status via pwd alias"),
        }
    }

    #[test]
    fn test_password_add_unlock_lock_flow() {
        let _guard = auth::tests::TEST_AUTH_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let auth_path = dir.path().join("auth.json");
        let session_path = dir.path().join("session.json");

        std::env::remove_var("TFA_PASSWORD");
        std::env::set_var("TFA_AUTH_FILE", &auth_path);
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();

        // 1. Add password
        let add_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "add",
            "-t",
            "300",
            "--password",
            "MySecretPassword",
        ])
        .unwrap();
        let mut out = Vec::new();
        run_app(add_cli, &store, &cb, &mut out).unwrap();

        // Check config updated
        let loaded_cfg = ConfigFile::load_from_path(&config_path).unwrap();
        assert!(loaded_cfg.is_password_enabled());
        assert_eq!(loaded_cfg.password_timeout(), 300);

        // 2. Status shows Unlocked
        let status_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "status",
        ])
        .unwrap();
        let mut status_out = Vec::new();
        assert!(run_app(status_cli.clone(), &store, &cb, &mut status_out).is_ok());
        let status_str = String::from_utf8(status_out).unwrap();
        assert!(status_str.contains("Unlocked"));

        // 3. List command succeeds without prompt because it's currently unlocked
        let list_cli =
            Cli::try_parse_from(["tfa", "-C", config_path.to_str().unwrap(), "list"]).unwrap();
        let mut list_out = Vec::new();
        assert!(run_app(list_cli.clone(), &store, &cb, &mut list_out).is_ok());
        let list_str = String::from_utf8(list_out).unwrap();
        assert!(list_str.contains("github"));

        // 4. Lock session
        let lock_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "lock",
        ])
        .unwrap();
        let mut lock_out = Vec::new();
        assert!(run_app(lock_cli, &store, &cb, &mut lock_out).is_ok());

        // Status shows Locked
        let mut status_out2 = Vec::new();
        assert!(run_app(status_cli, &store, &cb, &mut status_out2).is_ok());
        assert!(String::from_utf8(status_out2).unwrap().contains("Locked"));

        // 5. Running command while locked with correct TFA_PASSWORD succeeds and unlocks
        std::env::set_var("TFA_PASSWORD", "MySecretPassword");
        let mut list_out2 = Vec::new();
        assert!(run_app(list_cli.clone(), &store, &cb, &mut list_out2).is_ok());

        // 6. Running command with wrong TFA_PASSWORD fails
        let _ = auth::clear_session();
        std::env::set_var("TFA_PASSWORD", "WrongPassword");
        let mut list_out3 = Vec::new();
        match run_app(list_cli, &store, &cb, &mut list_out3) {
            Ok(_) => panic!("Expected run_app to fail with wrong password"),
            Err(e) => {
                assert!(
                    e.to_string().contains("Incorrect master password"),
                    "Unexpected error: {e}"
                );
            }
        }

        std::env::remove_var("TFA_AUTH_FILE");
        std::env::remove_var("TFA_SESSION_FILE");
        std::env::remove_var("TFA_PASSWORD");
    }

    #[test]
    fn test_password_remove_flow() {
        let _guard = auth::tests::TEST_AUTH_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let auth_path = dir.path().join("auth.json");
        let session_path = dir.path().join("session.json");

        std::env::remove_var("TFA_PASSWORD");
        std::env::set_var("TFA_AUTH_FILE", &auth_path);
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();

        // Add password first
        let add_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "add",
            "--password",
            "RemoveMe123",
        ])
        .unwrap();
        let mut out = Vec::new();
        assert!(run_app(add_cli, &store, &cb, &mut out).is_ok());

        // Remove password with --force
        let rm_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "remove",
            "-f",
        ])
        .unwrap();
        let mut rm_out = Vec::new();
        assert!(run_app(rm_cli, &store, &cb, &mut rm_out).is_ok());

        // Verify config shows disabled
        let reloaded_cfg = ConfigFile::load_from_path(&config_path).unwrap();
        assert!(!reloaded_cfg.is_password_enabled());

        // Verify list executes without password
        let list_cli =
            Cli::try_parse_from(["tfa", "-C", config_path.to_str().unwrap(), "list"]).unwrap();
        let mut list_out = Vec::new();
        assert!(run_app(list_cli, &store, &cb, &mut list_out).is_ok());
        assert!(String::from_utf8(list_out).unwrap().contains("github"));

        std::env::remove_var("TFA_AUTH_FILE");
        std::env::remove_var("TFA_SESSION_FILE");
        std::env::remove_var("TFA_PASSWORD");
    }

    #[test]
    fn test_password_unlock_command_flow() {
        let _guard = auth::tests::TEST_AUTH_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let auth_path = dir.path().join("auth.json");
        let session_path = dir.path().join("session.json");

        std::env::remove_var("TFA_PASSWORD");
        std::env::set_var("TFA_AUTH_FILE", &auth_path);
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // 1. Unlocking when no password configured fails
        let unlock_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "unlock",
        ])
        .unwrap();
        let mut out1 = Vec::new();
        let err1 = run_app(unlock_cli.clone(), &store, &cb, &mut out1).unwrap_err();
        assert!(err1.to_string().contains("No master password configured"));

        // 2. Add password
        let add_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "add",
            "--password",
            "Passcode99",
        ])
        .unwrap();
        let mut out2 = Vec::new();
        assert!(run_app(add_cli, &store, &cb, &mut out2).is_ok());

        // 3. Unlocking while already unlocked notices active session
        let mut out3 = Vec::new();
        assert!(run_app(unlock_cli.clone(), &store, &cb, &mut out3).is_ok());
        assert!(String::from_utf8(out3)
            .unwrap()
            .contains("already unlocked"));

        // 4. Lock session
        let lock_cli = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "lock",
        ])
        .unwrap();
        let mut out4 = Vec::new();
        assert!(run_app(lock_cli, &store, &cb, &mut out4).is_ok());

        // 5. Unlock with explicit password flag
        let unlock_with_pass = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "unlock",
            "--password",
            "Passcode99",
        ])
        .unwrap();
        let mut out5 = Vec::new();
        assert!(run_app(unlock_with_pass, &store, &cb, &mut out5).is_ok());
        assert!(String::from_utf8(out5)
            .unwrap()
            .contains("Session unlocked"));

        std::env::remove_var("TFA_AUTH_FILE");
        std::env::remove_var("TFA_SESSION_FILE");
        std::env::remove_var("TFA_PASSWORD");
    }

    #[test]
    fn test_password_enabled_without_configured_password_fails() {
        let _guard = auth::tests::TEST_AUTH_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let auth_path = dir.path().join("auth.json");
        let session_path = dir.path().join("session.json");

        std::env::remove_var("TFA_PASSWORD");
        std::env::set_var("TFA_AUTH_FILE", &auth_path);
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // Config has password = true, but no password has been added
        std::fs::write(&config_path, "password = true\n").unwrap();

        let list_cli =
            Cli::try_parse_from(["tfa", "-C", config_path.to_str().unwrap(), "list"]).unwrap();
        let mut out = Vec::new();
        let err = run_app(list_cli, &store, &cb, &mut out).unwrap_err();
        assert!(err
            .to_string()
            .contains("no master password has been configured"));

        std::env::remove_var("TFA_AUTH_FILE");
        std::env::remove_var("TFA_SESSION_FILE");
        std::env::remove_var("TFA_PASSWORD");
    }

    #[test]
    fn test_run_app_namespace_create_and_set_default() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        // 1. Create namespace
        let cli_create = Cli::try_parse_from(["tfa", "namespace", "create", "WorkSpace"]).unwrap();
        let mut out = Vec::new();
        assert!(run_app(cli_create, &store, &cb, &mut out).is_ok());
        assert!(String::from_utf8(out)
            .unwrap()
            .contains("Namespace 'workspace' created."));

        // 2. Set default namespace
        let cli_default = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "namespace",
            "set-default",
            "workspace",
        ])
        .unwrap();
        let mut out_def = Vec::new();
        assert!(run_app(cli_default, &store, &cb, &mut out_def).is_ok());
        assert!(String::from_utf8(out_def)
            .unwrap()
            .contains("Default namespace set to 'workspace'"));

        // 3. Delete namespace without force in non-interactive fails
        let cli_del = Cli::try_parse_from(["tfa", "namespace", "delete", "workspace"]).unwrap();
        let mut out_del = Vec::new();
        let err = run_app(cli_del, &store, &cb, &mut out_del).unwrap_err();
        assert!(err
            .to_string()
            .contains("Cannot delete namespace without --force"));
    }

    #[test]
    fn test_run_app_import_dry_run_full() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        store.set_secret("alice", "JBSWY3DPEHPK3PXP").unwrap();

        let payload = "otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP\notpauth://totp/bob?secret=JBSWY3DPEHPK3PXP\n";

        // 1. Dry run without overwrite: alice skipped, bob imported
        let cli_dry = Cli::try_parse_from(["tfa", "import", "--dry-run", payload]).unwrap();
        let mut out1 = Vec::new();
        assert!(run_app(cli_dry, &store, &cb, &mut out1).is_ok());
        let s1 = String::from_utf8(out1).unwrap();
        assert!(s1.contains("[dry-run] Would skip existing account 'alice'"));
        assert!(s1.contains("[dry-run] Would import account 'bob'"));
        assert!(s1.contains("[dry-run] Summary: 1 to import, 0 to update, 1 to skip."));
        // Check bob was not actually stored
        assert!(store.get_secret("bob").is_err());

        // 2. Dry run with overwrite: alice updated, bob imported
        let cli_dry_ow =
            Cli::try_parse_from(["tfa", "import", "--dry-run", "--overwrite", payload]).unwrap();
        let mut out2 = Vec::new();
        assert!(run_app(cli_dry_ow, &store, &cb, &mut out2).is_ok());
        let s2 = String::from_utf8(out2).unwrap();
        assert!(s2.contains("[dry-run] Would update account 'alice'"));
        assert!(s2.contains("[dry-run] Would import account 'bob'"));
    }

    #[test]
    fn test_run_app_list_and_status_empty_json() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        let cli_list_json = Cli::try_parse_from(["tfa", "list", "--json"]).unwrap();
        let mut out1 = Vec::new();
        assert!(run_app(cli_list_json, &store, &cb, &mut out1).is_ok());
        assert_eq!(String::from_utf8(out1).unwrap().trim(), "[]");

        let cli_status_json = Cli::try_parse_from(["tfa", "status", "--json"]).unwrap();
        let mut out2 = Vec::new();
        assert!(run_app(cli_status_json, &store, &cb, &mut out2).is_ok());
        assert_eq!(String::from_utf8(out2).unwrap().trim(), "[]");

        let cli_status = Cli::try_parse_from(["tfa", "status"]).unwrap();
        let mut out3 = Vec::new();
        assert!(run_app(cli_status, &store, &cb, &mut out3).is_ok());
        assert_eq!(
            String::from_utf8(out3).unwrap().trim(),
            "No accounts configured."
        );
    }

    #[test]
    fn test_run_app_show_hotp_and_manpage() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        store
            .set_secret(
                "my_hotp",
                "otpauth://hotp/my_hotp?secret=JBSWY3DPEHPK3PXP&counter=0",
            )
            .unwrap();

        let cli_show = Cli::try_parse_from(["tfa", "show", "my_hotp"]).unwrap();
        let mut out1 = Vec::new();
        assert!(run_app(cli_show, &store, &cb, &mut out1).is_ok());
        let s1 = String::from_utf8(out1).unwrap();
        assert!(s1.contains("Type:      HOTP"));
        assert!(s1.contains("Counter:   0"));
        assert!(s1.contains("Live Code:"));

        // HOTP show with --json
        let cli_show_json = Cli::try_parse_from(["tfa", "show", "my_hotp", "--json"]).unwrap();
        let mut out_json = Vec::new();
        assert!(run_app(cli_show_json, &store, &cb, &mut out_json).is_ok());
        let s_json = String::from_utf8(out_json).unwrap();
        let details: AccountDetails = serde_json::from_str(&s_json).unwrap();
        assert_eq!(details.account, "my_hotp");
        assert_eq!(details.account_type, "HOTP");
        assert_eq!(details.counter, Some(0));
        assert!(details.period.is_none());
        assert!(details.ttl.is_none());
        assert_eq!(details.live_code.len(), 6);

        // HOTP generate with --json
        let cli_gen_json = Cli::try_parse_from(["tfa", "generate", "my_hotp", "--json"]).unwrap();
        let mut out_gen_json = Vec::new();
        assert!(run_app(cli_gen_json, &store, &cb, &mut out_gen_json).is_ok());
        let s_gen_json = String::from_utf8(out_gen_json).unwrap();
        let gen_res: GenerateResult = serde_json::from_str(&s_gen_json).unwrap();
        assert_eq!(gen_res.account, "my_hotp");
        assert_eq!(gen_res.counter, Some(0));
        assert_eq!(gen_res.ttl, 0);
        assert_eq!(gen_res.period, 0);

        let cli_man = Cli::try_parse_from(["tfa", "man"]).unwrap();
        let mut out2 = Vec::new();
        assert!(run_app(cli_man, &store, &cb, &mut out2).is_ok());
        let s2 = String::from_utf8(out2).unwrap();
        assert!(s2.contains(".TH"));
    }

    #[test]
    fn test_get_password_env_var_and_validation() {
        let _guard = auth::tests::TEST_AUTH_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Empty password passed explicitly
        assert!(get_password(Some("".to_string()), false, "prompt").is_err());

        // Environment variable
        std::env::set_var("TFA_PASSWORD", "secret_env_pass");
        let pass = get_password(None, false, "prompt").unwrap();
        assert_eq!(pass, "secret_env_pass");
        std::env::remove_var("TFA_PASSWORD");

        // Non-interactive without env variable
        assert!(get_password(None, false, "prompt").is_err());
    }

    #[test]
    fn test_run_app_password_edge_flows() {
        let _guard = auth::tests::TEST_AUTH_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let auth_path = dir.path().join("auth.json");
        let session_path = dir.path().join("session.json");

        std::env::remove_var("TFA_PASSWORD");
        std::env::set_var("TFA_AUTH_FILE", &auth_path);
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // 1. Password status when unconfigured
        let cli_status = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "status",
        ])
        .unwrap();
        let mut out1 = Vec::new();
        assert!(run_app(cli_status, &store, &cb, &mut out1).is_ok());
        let s1 = String::from_utf8(out1).unwrap();
        assert!(s1.contains("Password Configured:   No"));

        // 2. Add password
        let cli_add = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "add",
            "--password",
            "Master123!",
        ])
        .unwrap();
        let mut out2 = Vec::new();
        assert!(run_app(cli_add, &store, &cb, &mut out2).is_ok());

        // 3. Unlock when already unlocked
        let cli_unlock_again = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "unlock",
            "--password",
            "Master123!",
        ])
        .unwrap();
        let mut out3 = Vec::new();
        assert!(run_app(cli_unlock_again, &store, &cb, &mut out3).is_ok());
        assert!(String::from_utf8(out3)
            .unwrap()
            .contains("Session already unlocked"));

        // 4. Lock session, then unlock with wrong password
        let cli_lock = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "lock",
        ])
        .unwrap();
        assert!(run_app(cli_lock, &store, &cb, &mut Vec::new()).is_ok());

        let cli_unlock_bad = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "unlock",
            "--password",
            "Wrong123!",
        ])
        .unwrap();
        assert!(run_app(cli_unlock_bad, &store, &cb, &mut Vec::new()).is_err());

        // 5. Remove password with wrong password fails
        let cli_remove_bad = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "remove",
            "--password",
            "Wrong123!",
        ])
        .unwrap();
        assert!(run_app(cli_remove_bad, &store, &cb, &mut Vec::new()).is_err());

        // 6. Remove password with correct password succeeds
        let cli_remove_good = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "remove",
            "--password",
            "Master123!",
        ])
        .unwrap();
        assert!(run_app(cli_remove_good, &store, &cb, &mut Vec::new()).is_ok());

        std::env::remove_var("TFA_AUTH_FILE");
        std::env::remove_var("TFA_SESSION_FILE");
        std::env::remove_var("TFA_PASSWORD");
    }

    #[test]
    fn test_resolve_instant_secret_with_reader() {
        // 1. Direct secret
        let res = resolve_instant_secret_with_reader(
            Some("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_string()),
            None,
            io::Cursor::new(b""),
            false,
        )
        .unwrap();
        assert_eq!(res, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");

        // 2. Empty direct secret fails
        let res_empty = resolve_instant_secret_with_reader(
            Some("   ".to_string()),
            None,
            io::Cursor::new(b""),
            false,
        );
        assert!(res_empty.is_err());

        // 3. Stdin via "-"
        let res_dash = resolve_instant_secret_with_reader(
            Some("-".to_string()),
            None,
            io::Cursor::new(b"  GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ  \n"),
            false,
        )
        .unwrap();
        assert_eq!(res_dash, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");

        // 4. Stdin via "-" with empty input fails
        let res_dash_empty = resolve_instant_secret_with_reader(
            Some("-".to_string()),
            None,
            io::Cursor::new(b"  \n  "),
            false,
        );
        assert!(res_dash_empty.is_err());

        // 5. Non-terminal piped stdin when secret is None
        let res_pipe = resolve_instant_secret_with_reader(
            None,
            None,
            io::Cursor::new(b"JBSWY3DPEHPK3PXP\n"),
            false,
        )
        .unwrap();
        assert_eq!(res_pipe, "JBSWY3DPEHPK3PXP");

        // 6. Non-terminal piped stdin when empty fails
        let res_pipe_empty =
            resolve_instant_secret_with_reader(None, None, io::Cursor::new(b"  \n"), false);
        assert!(res_pipe_empty.is_err());

        // 7. Interactive terminal reader
        let res_term = resolve_instant_secret_with_reader(
            None,
            None,
            io::Cursor::new(b"JBSWY3DPEHPK3PXP\n"),
            true,
        )
        .unwrap();
        assert_eq!(res_term, "JBSWY3DPEHPK3PXP");

        // 8. Interactive terminal with empty input fails
        let res_term_empty =
            resolve_instant_secret_with_reader(None, None, io::Cursor::new(b"\n"), true);
        assert!(res_term_empty.is_err());

        // 9. Environment variable TOTP_TOKEN
        std::env::set_var("TOTP_TOKEN", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        let res_env =
            resolve_instant_secret_with_reader(None, None, io::Cursor::new(b""), false).unwrap();
        assert_eq!(res_env, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        std::env::remove_var("TOTP_TOKEN");
    }

    #[test]
    fn test_cli_parse_instant() {
        // Positional argument
        let cli =
            Cli::try_parse_from(["tfa", "instant", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"]).unwrap();
        match cli.command {
            Some(Commands::Instant {
                secret,
                secret_flag,
                qr,
                digits,
                period,
                algorithm,
                counter,
                copy,
                watch,
            }) => {
                assert_eq!(secret, Some("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_string()));
                assert_eq!(secret_flag, None);
                assert_eq!(qr, None);
                assert_eq!(digits, None);
                assert_eq!(period, None);
                assert_eq!(algorithm, None);
                assert_eq!(counter, None);
                assert_eq!(copy, None);
                assert!(!watch);
            }
            _ => panic!("Expected instant subcommand"),
        }

        // Subcommand alias "ins" with all flags
        let cli_alias = Cli::try_parse_from([
            "tfa",
            "ins",
            "--secret",
            "JBSWY3DPEHPK3PXP",
            "-d",
            "8",
            "-p",
            "60",
            "-a",
            "sha256",
            "--counter",
            "42",
            "-c",
            "-w",
        ])
        .unwrap();
        match cli_alias.command {
            Some(Commands::Instant {
                secret,
                secret_flag,
                digits,
                period,
                algorithm,
                counter,
                copy,
                watch,
                ..
            }) => {
                assert_eq!(secret, None);
                assert_eq!(secret_flag, Some("JBSWY3DPEHPK3PXP".to_string()));
                assert_eq!(digits, Some(8));
                assert_eq!(period, Some(60));
                assert_eq!(algorithm, Some(totp_rs::Algorithm::SHA256));
                assert_eq!(counter, Some(42));
                assert_eq!(copy, Some(true));
                assert!(watch);
            }
            _ => panic!("Expected instant via ins alias"),
        }

        // Generate with --secret
        let cli_gen_sec = Cli::try_parse_from([
            "tfa",
            "generate",
            "--secret",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "-d",
            "7",
            "-p",
            "45",
            "-a",
            "sha512",
        ])
        .unwrap();
        match cli_gen_sec.command {
            Some(Commands::Generate {
                secret,
                digits,
                period,
                algorithm,
                ..
            }) => {
                assert_eq!(secret, Some("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_string()));
                assert_eq!(digits, Some(7));
                assert_eq!(period, Some(45));
                assert_eq!(algorithm, Some(totp_rs::Algorithm::SHA512));
            }
            _ => panic!("Expected generate subcommand with secret"),
        }
    }

    #[test]
    fn test_run_app_instant_flows() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // 1. Instant with raw Base32 secret
        let mut out = Vec::new();
        let cli1 =
            Cli::try_parse_from(["tfa", "instant", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"]).unwrap();
        assert!(run_app(cli1, &store, &cb, &mut out).is_ok());
        let out_str1 = String::from_utf8(out).unwrap();
        assert!(out_str1.contains("remaining)"));
        let token1 = out_str1.split_whitespace().next().unwrap();
        assert_eq!(token1.len(), 6);
        assert!(token1.chars().all(|c| c.is_ascii_digit()));

        // 2. Instant with copy flag
        let mut out2 = Vec::new();
        let cli2 =
            Cli::try_parse_from(["tfa", "instant", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", "-c"])
                .unwrap();
        assert!(run_app(cli2, &store, &cb, &mut out2).is_ok());
        let copied = cb.text.lock().unwrap().clone();
        assert!(copied.is_some());
        assert_eq!(copied.unwrap(), token1);

        // 3. Instant with custom parameters (-d 8 -p 60 -a sha256)
        let mut out3 = Vec::new();
        let cli3 = Cli::try_parse_from([
            "tfa",
            "instant",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "-d",
            "8",
            "-p",
            "60",
            "-a",
            "sha256",
        ])
        .unwrap();
        assert!(run_app(cli3, &store, &cb, &mut out3).is_ok());
        let out_str3 = String::from_utf8(out3).unwrap();
        let token3 = out_str3.split_whitespace().next().unwrap();
        assert_eq!(token3.len(), 8);

        // 4. Instant with OTPAuth URI
        let mut out4 = Vec::new();
        let uri = "otpauth://totp/TestAcct?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&digits=8&period=60&algorithm=SHA256";
        let cli4 = Cli::try_parse_from(["tfa", "instant", uri]).unwrap();
        assert!(run_app(cli4, &store, &cb, &mut out4).is_ok());
        let out_str4 = String::from_utf8(out4).unwrap();
        let token4 = out_str4.split_whitespace().next().unwrap();
        assert_eq!(token4.len(), 8);
        assert_eq!(token3, token4);

        // 5. Instant with HOTP URI
        let mut out5 = Vec::new();
        let hotp_uri = "otpauth://hotp/HotpTest?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&counter=3";
        let cli5 = Cli::try_parse_from(["tfa", "instant", hotp_uri]).unwrap();
        assert!(run_app(cli5, &store, &cb, &mut out5).is_ok());
        let out_str5 = String::from_utf8(out5).unwrap();
        assert!(out_str5.contains("(counter: 3)"));
        let token5 = out_str5.split_whitespace().next().unwrap();
        // RFC 4226 test vector for counter=3 is 969429
        assert_eq!(token5, "969429");

        // 6. Generate --secret flow
        let mut out6 = Vec::new();
        let cli6 = Cli::try_parse_from([
            "tfa",
            "generate",
            "--secret",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        ])
        .unwrap();
        assert!(run_app(cli6, &store, &cb, &mut out6).is_ok());
        let out_str6 = String::from_utf8(out6).unwrap();
        let token6 = out_str6.split_whitespace().next().unwrap();
        assert_eq!(token6, token1);

        // 7. Instant with invalid secret fails
        let mut out7 = Vec::new();
        let cli7 = Cli::try_parse_from(["tfa", "instant", "INVALID_BASE32_189!"]).unwrap();
        assert!(run_app(cli7, &store, &cb, &mut out7).is_err());

        // 8. Instant with raw secret and --counter 5 generates HOTP code at counter 5
        let mut out8 = Vec::new();
        let cli8 = Cli::try_parse_from([
            "tfa",
            "instant",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "--counter",
            "5",
        ])
        .unwrap();
        assert!(run_app(cli8, &store, &cb, &mut out8).is_ok());
        let out_str8 = String::from_utf8(out8).unwrap();
        assert!(out_str8.contains("(counter: 5)"));
        let token8 = out_str8.split_whitespace().next().unwrap();
        // RFC 4226 test vector for counter=5 is 254676
        assert_eq!(token8, "254676");

        // 9. Instant with QR image
        let tmp_qr = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        let qr_payload =
            "otpauth://totp/GitLab:user@gitlab.com?secret=JBSWY3DPEHPK3PXP&issuer=GitLab";
        let qr_code =
            qrcodegen::QrCode::encode_text(qr_payload, qrcodegen::QrCodeEcc::Medium).unwrap();
        let size = qr_code.size() as u32;
        let border = 4u32;
        let img_dim = size + border * 2;
        let mut img_buf = image::GrayImage::new(img_dim, img_dim);
        for pixel in img_buf.pixels_mut() {
            *pixel = image::Luma([255u8]);
        }
        for y in 0..size {
            for x in 0..size {
                if qr_code.get_module(x as i32, y as i32) {
                    img_buf.put_pixel(x + border, y + border, image::Luma([0u8]));
                }
            }
        }
        img_buf.save(tmp_qr.path()).unwrap();

        let mut out_qr = Vec::new();
        let cli_qr =
            Cli::try_parse_from(["tfa", "instant", "--qr", tmp_qr.path().to_str().unwrap()])
                .unwrap();
        assert!(run_app(cli_qr, &store, &cb, &mut out_qr).is_ok());
        let out_str_qr = String::from_utf8(out_qr).unwrap();
        let token_qr = out_str_qr.split_whitespace().next().unwrap();
        assert_eq!(token_qr.len(), 6);

        // 10. Instant with TOTP_TOKEN env var
        std::env::set_var("TOTP_TOKEN", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        let mut out10 = Vec::new();
        let cli10 = Cli::try_parse_from(["tfa", "instant"]).unwrap();
        assert!(run_app(cli10, &store, &cb, &mut out10).is_ok());
        let out_str10 = String::from_utf8(out10).unwrap();
        let token10 = out_str10.split_whitespace().next().unwrap();
        assert_eq!(token10, token1);
        std::env::remove_var("TOTP_TOKEN");

        // 11. Generate with "-" when stdin is empty fails
        let cli11 = Cli::try_parse_from(["tfa", "generate", "-"]).unwrap();
        assert!(run_app(cli11, &store, &cb, &mut Vec::new()).is_err());
    }

    #[test]
    fn test_run_app_instant_with_password_protection_does_not_block() {
        let _guard = auth::tests::TEST_AUTH_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let store = MemoryStore::new();
        let cb = MockClipboard::default();
        let tmp_dir = tempfile::tempdir().unwrap();
        let auth_path = tmp_dir.path().join("auth.json");
        let session_path = tmp_dir.path().join("session.json");
        let config_path = tmp_dir.path().join("config.toml");

        std::env::remove_var("TFA_PASSWORD");
        std::env::set_var("TFA_AUTH_FILE", &auth_path);
        std::env::set_var("TFA_SESSION_FILE", &session_path);

        let cfg_content = "storage = \"file\"\n[password]\nenabled = true\ntimeout = 300\n";
        std::fs::write(&config_path, cfg_content).unwrap();

        // Configure password
        let cli_add = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "add",
            "--password",
            "SecretPass123!",
        ])
        .unwrap();
        assert!(run_app(cli_add, &store, &cb, &mut Vec::new()).is_ok());

        // Lock session immediately
        let cli_lock = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "password",
            "lock",
        ])
        .unwrap();
        assert!(run_app(cli_lock, &store, &cb, &mut Vec::new()).is_ok());

        // Normal generate should fail because session is locked and environment is non-interactive
        let cli_gen_locked = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "generate",
            "dummy_acct",
        ])
        .unwrap();
        assert!(run_app(cli_gen_locked, &store, &cb, &mut Vec::new()).is_err());

        // Instant generation MUST SUCCEED even when session is locked!
        let mut out_instant = Vec::new();
        let cli_instant = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "instant",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        ])
        .unwrap();
        assert!(run_app(cli_instant, &store, &cb, &mut out_instant).is_ok());
        let token = String::from_utf8(out_instant).unwrap();
        assert!(token.contains("remaining)"));

        // Generate --secret MUST ALSO SUCCEED even when session is locked!
        let mut out_gen_sec = Vec::new();
        let cli_gen_sec = Cli::try_parse_from([
            "tfa",
            "-C",
            config_path.to_str().unwrap(),
            "generate",
            "--secret",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
        ])
        .unwrap();
        assert!(run_app(cli_gen_sec, &store, &cb, &mut out_gen_sec).is_ok());

        std::env::remove_var("TFA_AUTH_FILE");
        std::env::remove_var("TFA_SESSION_FILE");
        std::env::remove_var("TFA_PASSWORD");
    }

    #[test]
    fn test_cli_parse_rename_and_move() {
        let cli_rename = Cli::try_parse_from(["tfa", "rename", "github", "gh"]).unwrap();
        match cli_rename.command {
            Some(Commands::Rename { old_name, new_name }) => {
                assert_eq!(old_name, "github");
                assert_eq!(new_name, "gh");
            }
            _ => panic!("Expected Rename command"),
        }

        let cli_move = Cli::try_parse_from(["tfa", "move", "router", "--to", "home"]).unwrap();
        match cli_move.command {
            Some(Commands::Move { account, to }) => {
                assert_eq!(account, "router");
                assert_eq!(to, "home");
            }
            _ => panic!("Expected Move command"),
        }

        let cli_mv = Cli::try_parse_from(["tfa", "mv", "aws", "-t", "work"]).unwrap();
        match cli_mv.command {
            Some(Commands::Move { account, to }) => {
                assert_eq!(account, "aws");
                assert_eq!(to, "work");
            }
            _ => panic!("Expected Move command via alias"),
        }
    }

    #[test]
    fn test_run_app_rename_flow() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // 1. Add account with custom parameters (digits=8, period=60, sha256)
        let cli_add_gh = Cli::try_parse_from([
            "tfa",
            "add",
            "github",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "--digits",
            "8",
            "--period",
            "60",
            "--algorithm",
            "sha256",
        ])
        .unwrap();
        assert!(run_app(cli_add_gh, &store, &cb, &mut Vec::new()).is_ok());

        // Add a second account
        let cli_add_gl = Cli::try_parse_from(["tfa", "add", "gitlab", "JBSWY3DPEHPK3PXP"]).unwrap();
        assert!(run_app(cli_add_gl, &store, &cb, &mut Vec::new()).is_ok());

        // 2. Rename github -> gh
        let mut out_rename = Vec::new();
        let cli_rename = Cli::try_parse_from(["tfa", "rename", "github", "gh"]).unwrap();
        assert!(run_app(cli_rename, &store, &cb, &mut out_rename).is_ok());
        let rename_str = String::from_utf8(out_rename).unwrap();
        assert!(rename_str.contains("Account 'github' renamed to 'gh'."));

        // 3. Verify gh preserved all parameters
        let mut out_show_gh = Vec::new();
        let cli_show_gh = Cli::try_parse_from(["tfa", "show", "gh"]).unwrap();
        assert!(run_app(cli_show_gh, &store, &cb, &mut out_show_gh).is_ok());
        let show_str = String::from_utf8(out_show_gh).unwrap();
        assert!(show_str.contains("Account:   gh"));
        assert!(show_str.contains("Digits:    8"));
        assert!(show_str.contains("Period:    60s"));
        assert!(show_str.contains("Algorithm: SHA256"));

        // Old name should not exist
        let cli_show_old = Cli::try_parse_from(["tfa", "show", "github"]).unwrap();
        assert!(run_app(cli_show_old, &store, &cb, &mut Vec::new()).is_err());

        // 4. Duplicate target error: rename gitlab -> gh
        let cli_rename_dup = Cli::try_parse_from(["tfa", "rename", "gitlab", "gh"]).unwrap();
        let err_dup = run_app(cli_rename_dup, &store, &cb, &mut Vec::new()).unwrap_err();
        assert!(err_dup.to_string().contains("already exists"));

        // 5. Missing account error
        let cli_rename_missing =
            Cli::try_parse_from(["tfa", "rename", "nonexistent", "newname"]).unwrap();
        let err_missing = run_app(cli_rename_missing, &store, &cb, &mut Vec::new()).unwrap_err();
        assert!(err_missing.to_string().contains("not found"));

        // 6. Scoped namespace rename
        let cli_add_work =
            Cli::try_parse_from(["tfa", "-n", "work", "add", "aws", "JBSWY3DPEHPK3PXP"]).unwrap();
        assert!(run_app(cli_add_work, &store, &cb, &mut Vec::new()).is_ok());

        let cli_rename_work =
            Cli::try_parse_from(["tfa", "-n", "work", "rename", "aws", "amazon"]).unwrap();
        assert!(run_app(cli_rename_work, &store, &cb, &mut Vec::new()).is_ok());

        let cli_show_work = Cli::try_parse_from(["tfa", "-n", "work", "show", "amazon"]).unwrap();
        assert!(run_app(cli_show_work, &store, &cb, &mut Vec::new()).is_ok());

        let cli_show_work_old = Cli::try_parse_from(["tfa", "-n", "work", "show", "aws"]).unwrap();
        assert!(run_app(cli_show_work_old, &store, &cb, &mut Vec::new()).is_err());
    }

    #[test]
    fn test_run_app_move_flow() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // 1. Add account with custom parameters in default namespace
        let cli_add_vpn = Cli::try_parse_from([
            "tfa",
            "add",
            "vpn",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "--digits",
            "8",
            "--period",
            "60",
            "--algorithm",
            "sha256",
        ])
        .unwrap();
        assert!(run_app(cli_add_vpn, &store, &cb, &mut Vec::new()).is_ok());

        // 2. Move vpn to infra namespace
        let mut out_move = Vec::new();
        let cli_move = Cli::try_parse_from(["tfa", "move", "vpn", "--to", "infra"]).unwrap();
        assert!(run_app(cli_move, &store, &cb, &mut out_move).is_ok());
        let move_str = String::from_utf8(out_move).unwrap();
        assert!(move_str.contains("Account 'vpn' moved to namespace 'infra'."));

        // Default namespace no longer lists vpn
        let mut out_list_def = Vec::new();
        let cli_list_def = Cli::try_parse_from(["tfa", "list"]).unwrap();
        assert!(run_app(cli_list_def, &store, &cb, &mut out_list_def).is_ok());
        assert!(!String::from_utf8(out_list_def).unwrap().contains("vpn"));

        // Infra namespace now lists vpn
        let mut out_list_infra = Vec::new();
        let cli_list_infra = Cli::try_parse_from(["tfa", "-n", "infra", "list"]).unwrap();
        assert!(run_app(cli_list_infra, &store, &cb, &mut out_list_infra).is_ok());
        assert!(String::from_utf8(out_list_infra).unwrap().contains("vpn"));

        // Parameters preserved in infra namespace
        let mut out_show_infra = Vec::new();
        let cli_show_infra = Cli::try_parse_from(["tfa", "-n", "infra", "show", "vpn"]).unwrap();
        assert!(run_app(cli_show_infra, &store, &cb, &mut out_show_infra).is_ok());
        let show_str = String::from_utf8(out_show_infra).unwrap();
        assert!(show_str.contains("Digits:    8"));
        assert!(show_str.contains("Period:    60s"));
        assert!(show_str.contains("Algorithm: SHA256"));

        // 3. Move back from infra to default using short flag -t
        let cli_move_back =
            Cli::try_parse_from(["tfa", "-n", "infra", "mv", "vpn", "-t", "default"]).unwrap();
        assert!(run_app(cli_move_back, &store, &cb, &mut Vec::new()).is_ok());

        let mut out_list_def2 = Vec::new();
        assert!(run_app(
            Cli::try_parse_from(["tfa", "list"]).unwrap(),
            &store,
            &cb,
            &mut out_list_def2
        )
        .is_ok());
        assert!(String::from_utf8(out_list_def2).unwrap().contains("vpn"));

        // 4. Duplicate target error
        let cli_add_infra_dup =
            Cli::try_parse_from(["tfa", "-n", "infra", "add", "vpn", "JBSWY3DPEHPK3PXP"]).unwrap();
        assert!(run_app(cli_add_infra_dup, &store, &cb, &mut Vec::new()).is_ok());

        let cli_move_dup = Cli::try_parse_from(["tfa", "move", "vpn", "--to", "infra"]).unwrap();
        let err_dup = run_app(cli_move_dup, &store, &cb, &mut Vec::new()).unwrap_err();
        assert!(err_dup
            .to_string()
            .contains("already exists in namespace 'infra'"));

        // 5. Identical namespace error
        let cli_move_same = Cli::try_parse_from(["tfa", "move", "vpn", "--to", "default"]).unwrap();
        let err_same = run_app(cli_move_same, &store, &cb, &mut Vec::new()).unwrap_err();
        assert!(err_same.to_string().contains("identical"));

        // 6. Missing account error
        let cli_move_missing =
            Cli::try_parse_from(["tfa", "move", "nonexistent", "--to", "infra"]).unwrap();
        let err_missing = run_app(cli_move_missing, &store, &cb, &mut Vec::new()).unwrap_err();
        assert!(err_missing.to_string().contains("not found"));
    }

    #[test]
    fn test_cli_parse_audit() {
        let cli = Cli::try_parse_from(["tfa", "audit"]).unwrap();
        match cli.command {
            Some(Commands::Audit { json }) => assert!(!json),
            _ => panic!("Expected Audit command"),
        }

        let cli_json = Cli::try_parse_from(["tfa", "audit", "--json"]).unwrap();
        match cli_json.command {
            Some(Commands::Audit { json }) => assert!(json),
            _ => panic!("Expected Audit command with --json"),
        }
    }

    #[test]
    fn test_run_app_audit_execution() {
        let store = MemoryStore::new();
        let cb = MockClipboard::default();

        // Empty store check
        let cli_empty = Cli::try_parse_from(["tfa", "audit"]).unwrap();
        let mut out_empty = Vec::new();
        assert!(run_app(cli_empty, &store, &cb, &mut out_empty).is_ok());
        let empty_str = String::from_utf8(out_empty).unwrap();
        assert!(empty_str.contains("Accounts Scanned: 0"));
        assert!(empty_str.contains("Issues Found: 0"));
        assert!(empty_str.contains("No security vulnerabilities detected."));

        // Empty store JSON check
        let cli_empty_json = Cli::try_parse_from(["tfa", "audit", "--json"]).unwrap();
        let mut out_empty_json = Vec::new();
        assert!(run_app(cli_empty_json, &store, &cb, &mut out_empty_json).is_ok());
        let empty_json_str = String::from_utf8(out_empty_json).unwrap();
        let report_empty: audit::AuditReport = serde_json::from_str(&empty_json_str).unwrap();
        assert_eq!(report_empty.accounts_scanned, 0);
        assert_eq!(report_empty.summary.total_issues, 0);

        // Add accounts:
        // 1. Weak secret (8 Base32 chars, SHA-1) in default namespace
        let cli_add_weak = Cli::try_parse_from(["tfa", "add", "weak_one", "GEZDGNBV"]).unwrap();
        assert!(run_app(cli_add_weak, &store, &cb, &mut Vec::new()).is_ok());

        // 2. Strong secret with SHA-1 in default namespace (shares seed with work account)
        let cli_add_shared1 =
            Cli::try_parse_from(["tfa", "add", "shared1", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"])
                .unwrap();
        assert!(run_app(cli_add_shared1, &store, &cb, &mut Vec::new()).is_ok());

        // 3. Same strong seed in 'work' namespace (SHA-256)
        let cli_add_shared2 = Cli::try_parse_from([
            "tfa",
            "-n",
            "work",
            "add",
            "shared2",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            "--algorithm",
            "sha256",
        ])
        .unwrap();
        assert!(run_app(cli_add_shared2, &store, &cb, &mut Vec::new()).is_ok());

        // Run audit human-readable
        let cli_audit = Cli::try_parse_from(["tfa", "audit"]).unwrap();
        let mut out_audit = Vec::new();
        assert!(run_app(cli_audit, &store, &cb, &mut out_audit).is_ok());
        let audit_str = String::from_utf8(out_audit).unwrap();
        assert!(audit_str.contains("Accounts Scanned: 3"));
        assert!(audit_str.contains("Weak secret with 8 Base32 characters"));
        assert!(audit_str.contains("Legacy SHA-1 algorithm"));
        assert!(audit_str.contains("Duplicate secret seed shared with 'work:shared2'"));
        assert!(audit_str.contains("Duplicate secret seed shared with 'default:shared1'"));

        // Run audit JSON
        let cli_json = Cli::try_parse_from(["tfa", "audit", "--json"]).unwrap();
        let mut out_json = Vec::new();
        assert!(run_app(cli_json, &store, &cb, &mut out_json).is_ok());
        let json_str = String::from_utf8(out_json).unwrap();
        let report: audit::AuditReport = serde_json::from_str(&json_str).unwrap();
        assert_eq!(report.accounts_scanned, 3);
        assert_eq!(report.summary.weak_secrets, 1);
        assert_eq!(report.summary.duplicate_secrets, 2);
        assert_eq!(report.summary.legacy_algorithms, 2); // weak_one and shared1 both use SHA-1
    }
}
