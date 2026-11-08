pub use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::engine::ArgValueCandidates;
pub use clap_complete::Shell;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use crate::clipboard::ClipboardProvider;
use crate::completions;
use crate::export::ExportFormat;
use crate::migrate::MigrateBackend;

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

    /// Automatically clear copied clipboard token after timeout in seconds (or 0 / false to disable)
    #[arg(
        long,
        value_name = "SECONDS",
        num_args = 0..=1,
        default_missing_value = "true",
        global = true,
        conflicts_with = "no_clear"
    )]
    pub clear_after: Option<String>,

    /// Disable automatic clipboard clearing
    #[arg(
        long,
        visible_alias = "no-clear-after",
        global = true,
        conflicts_with = "clear_after"
    )]
    pub no_clear: bool,

    /// Internal background clipboard clear worker flag
    #[arg(long, hide = true)]
    pub internal_clear_clipboard: Option<u64>,

    /// Target namespace for account scoping (e.g. personal, work, home)
    #[arg(
        short = 'n',
        long,
        env = "TFA_NAMESPACE",
        global = true,
        conflicts_with = "all_namespaces",
        add = ArgValueCandidates::new(completions::complete_namespaces)
    )]
    pub namespace: Option<String>,

    /// Show accounts across all namespaces
    #[arg(
        short = 'A',
        long = "all-namespaces",
        global = true,
        conflicts_with = "namespace"
    )]
    pub all_namespaces: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn try_parse_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let cli = <Self as clap::Parser>::try_parse_from(itr)?;
        if cli.all_namespaces && cli.namespace.is_some() {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::ArgumentConflict,
                "the argument '--all-namespaces' cannot be used with '--namespace <NAMESPACE>'\n",
            ));
        }
        Ok(cli)
    }

    pub fn parse() -> Self {
        match Self::try_parse_from(std::env::args_os()) {
            Ok(cli) => cli,
            Err(e) => e.exit(),
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Generate a 2FA token (TOTP, Steam, or HOTP) for an account
    #[command(alias = "gen", alias = "g")]
    Generate {
        /// Name of the account (interactively prompted via fuzzy search if omitted and --secret not supplied)
        #[arg(add = ArgValueCandidates::new(completions::complete_accounts))]
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
        /// Unix timestamp in seconds for TOTP generation
        #[arg(long, conflicts_with = "watch", conflicts_with = "counter")]
        at: Option<u64>,
        /// Relative time offset in seconds (e.g. +30, -60) for TOTP generation
        #[arg(
            long,
            allow_hyphen_values = true,
            conflicts_with = "watch",
            conflicts_with = "counter"
        )]
        skew: Option<i64>,
        /// Preview upcoming token for next time step
        #[arg(long, conflicts_with = "watch", conflicts_with = "counter")]
        next: bool,
        /// Output generated token details as JSON
        #[arg(long)]
        json: bool,
        /// Emit only the raw token string without decorations or metadata
        #[arg(
            short = 'q',
            long = "quiet",
            visible_alias = "token-only",
            num_args = 0..=1,
            default_missing_value = "true",
            require_equals = true,
            conflicts_with = "watch",
            conflicts_with = "next",
            conflicts_with = "json"
        )]
        quiet: Option<bool>,
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
        /// Unix timestamp in seconds for TOTP generation
        #[arg(long, conflicts_with = "watch", conflicts_with = "counter")]
        at: Option<u64>,
        /// Relative time offset in seconds (e.g. +30, -60) for TOTP generation
        #[arg(
            long,
            allow_hyphen_values = true,
            conflicts_with = "watch",
            conflicts_with = "counter"
        )]
        skew: Option<i64>,
        /// Copy generated token to clipboard
        #[arg(short, long, num_args = 0..=1, default_missing_value = "true")]
        copy: Option<bool>,
        /// Watch and continuously refresh code with progress bar
        #[arg(short, long)]
        watch: bool,
        /// Emit only the raw token string without decorations or metadata
        #[arg(
            short = 'q',
            long = "quiet",
            visible_alias = "token-only",
            num_args = 0..=1,
            default_missing_value = "true",
            require_equals = true,
            conflicts_with = "watch"
        )]
        quiet: Option<bool>,
    },
    /// Live watch mode displaying continuous TOTP countdown with progress bar
    Watch {
        #[command(subcommand)]
        command: Option<WatchCommands>,
        /// Name of the account (interactively prompted via fuzzy search if omitted)
        #[arg(add = ArgValueCandidates::new(completions::complete_accounts))]
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
        /// Static prefix (e.g. PIN) prepended to generated OTP tokens
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Edit parameters or prefix of an existing account in-place
    #[command(alias = "set")]
    Edit {
        /// Name of the account
        #[arg(add = ArgValueCandidates::new(completions::complete_accounts))]
        account: String,
        /// Number of digits in generated code (e.g. 5, 6, 8)
        #[arg(short, long)]
        digits: Option<u8>,
        /// Time step period in seconds (e.g. 30 or 60)
        #[arg(short, long)]
        period: Option<u64>,
        /// Hash algorithm (sha1, sha256, sha512, steam)
        #[arg(short, long)]
        algorithm: Option<totp_rs::Algorithm>,
        /// Counter for counter-based (HOTP) accounts
        #[arg(short, long)]
        counter: Option<u64>,
        /// Static prefix (e.g. PIN) prepended to generated OTP tokens
        #[arg(long, conflicts_with = "clear_prefix")]
        prefix: Option<String>,
        /// Clear any configured token prefix
        #[arg(long, conflicts_with = "prefix")]
        clear_prefix: bool,
    },
    /// Render account as a terminal QR code for mobile authenticator scanning
    Qr {
        /// Name of the account (interactively prompted if omitted and --all not supplied)
        #[arg(
            conflicts_with = "all",
            add = ArgValueCandidates::new(completions::complete_accounts)
        )]
        account: Option<String>,
        /// Export all accounts in the active namespace as batch migration QR code(s)
        #[arg(short, long, visible_alias = "migration", conflicts_with = "account")]
        all: bool,
        /// Invert terminal QR blocks (for light terminal backgrounds)
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        light: Option<bool>,
    },
    /// List all stored accounts or live token status
    #[command(alias = "ls")]
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
    #[command(alias = "del", alias = "rm", alias = "remove")]
    Delete {
        /// Name of the account (interactively prompted via fuzzy search if omitted)
        #[arg(add = ArgValueCandidates::new(completions::complete_accounts))]
        account: Option<String>,
    },
    /// Rename an account or batch rename accounts within the active namespace
    #[command(alias = "ren")]
    Rename {
        /// Current account name
        #[arg(
            add = ArgValueCandidates::new(completions::complete_accounts),
            required_unless_present = "batch"
        )]
        old_name: Option<String>,
        /// New account name (derived automatically if omitted)
        #[arg(conflicts_with = "batch")]
        new_name: Option<String>,
        /// Batch or auto rename accounts to clean command-line friendly names (lowercase, space replaced)
        #[arg(short = 'b', long, visible_alias = "all", visible_alias = "auto")]
        batch: bool,
        /// Space replacement character or string (default: "-")
        #[arg(
            short = 's',
            long = "separator",
            visible_alias = "replace-with",
            env = "TFA_RENAME_SEPARATOR"
        )]
        separator: Option<String>,
        /// Preview changes without modifying storage
        #[arg(long)]
        dry_run: bool,
    },
    /// Move an account to a different namespace
    #[command(alias = "mv")]
    Move {
        /// Name of the account to move
        #[arg(add = ArgValueCandidates::new(completions::complete_accounts))]
        account: String,
        /// Destination namespace
        #[arg(
            short = 't',
            long = "to",
            add = ArgValueCandidates::new(completions::complete_namespaces)
        )]
        to: String,
    },
    /// Show account details and stored secret
    Show {
        /// Name of the account
        #[arg(add = ArgValueCandidates::new(completions::complete_accounts))]
        account: String,
        /// Print only the raw secret or URI
        #[arg(short, long, conflicts_with = "token")]
        raw: bool,
        /// Output account details and metadata as JSON
        #[arg(long, conflicts_with = "raw", conflicts_with = "token")]
        json: bool,
        /// Print only the current live OTP token
        #[arg(
            short = 't',
            long = "token",
            visible_alias = "code",
            conflicts_with = "raw",
            conflicts_with = "json"
        )]
        token: bool,
    },
    /// Export accounts to an encrypted backup file or stdout
    Export {
        /// Destination backup file path (prints to stdout if omitted)
        file: Option<PathBuf>,
        /// Export format (json, otpauth, or yaml)
        #[arg(long, default_value = "json", value_name = "FORMAT")]
        format: ExportFormat,
        /// Export without encryption (plain JSON, YAML, or OTPAuth URLs)
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
    /// Migrate accounts directly in memory across storage backends (keyring, file, age)
    Migrate {
        /// Source storage backend to read accounts from
        #[arg(long, value_name = "BACKEND")]
        from: MigrateBackend,
        /// Destination storage backend to write accounts to
        #[arg(long, value_name = "BACKEND")]
        to: MigrateBackend,
        /// Overwrite existing accounts in destination storage
        #[arg(long)]
        overwrite: bool,
        /// Preview migration without writing to destination or deleting from source
        #[arg(long)]
        dry_run: bool,
        /// Delete migrated accounts from source storage after transfer
        #[arg(long)]
        purge_source: bool,
    },
    /// Generate shell autocompletions script
    Completions {
        /// Shell to generate completions for
        shell: Shell,
        /// Generate static autocompletions script instead of dynamic
        #[arg(long = "static")]
        static_completions: bool,
    },
    /// Generate UNIX roff man page
    Man,
    /// Manage and isolate accounts across multi-tenant namespaces
    #[command(alias = "ns")]
    Namespace {
        #[command(subcommand)]
        command: Option<NamespaceCommands>,
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
    /// Validate system prerequisites, storage permissions, and diagnostic health
    #[command(alias = "doc")]
    Doctor {
        /// Output doctor report in JSON format
        #[arg(long)]
        json: bool,
        /// Custom NTP server for clock drift check
        #[arg(long)]
        ntp_server: Option<String>,
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
    #[command(alias = "ls")]
    List,
    /// Show the current active namespace
    #[command(alias = "show", alias = "get")]
    Current,
    /// Show the default namespace configured in settings
    #[command(alias = "get-default", alias = "show-default")]
    Default,
    /// Create a new namespace
    Create {
        /// Name of the new namespace
        name: String,
    },
    /// Rename an existing namespace, safely migrating all contained accounts
    #[command(alias = "ren", alias = "mv")]
    Rename {
        /// Current namespace name
        #[arg(add = ArgValueCandidates::new(completions::complete_namespaces))]
        old_name: String,
        /// New namespace name
        new_name: String,
    },
    /// Delete a namespace and its contained accounts
    #[command(alias = "del", alias = "rm", alias = "remove")]
    Delete {
        /// Name of the namespace to delete
        #[arg(add = ArgValueCandidates::new(completions::complete_namespaces))]
        name: String,
        /// Force deletion without interactive confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Switch and persist default active namespace in configuration file
    #[command(alias = "use", alias = "switch", alias = "set")]
    SetDefault {
        /// Name of the namespace to set as default
        #[arg(add = ArgValueCandidates::new(completions::complete_namespaces))]
        name: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AccountStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    pub live_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}

pub fn generate_completions(shell: Shell, writer: &mut dyn Write) {
    if completions::generate_dynamic_completions(shell, writer).is_err() {
        generate_static_completions(shell, writer);
    }
}

pub fn generate_static_completions(shell: Shell, writer: &mut dyn Write) {
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

/// Determine if output should be quiet based on explicit flag or TTY auto-detection
pub fn is_stdout_quiet(quiet_flag: Option<bool>) -> bool {
    quiet_flag.unwrap_or_else(|| {
        if cfg!(test) {
            std::env::var("TFA_TEST_NON_TERMINAL").is_ok()
        } else {
            !io::stdout().is_terminal()
        }
    })
}
