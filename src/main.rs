use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::io::{self, BufRead, Write};

mod clipboard;
mod storage;
mod totp;

use clipboard::{Clipboard, SystemClipboard};
use storage::{AccountStore, KeyringStore};
use totp::TfaTotp;

#[derive(Parser, Debug)]
#[command(name = "tfa")]
#[command(author, version = env!("PROJECT_VERSION"), about = "CLI Two-Factor Authentication (2FA) tool", long_about = None)]
pub struct Cli {
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
    },
    /// List all stored accounts
    List,
    /// Delete an account secret
    Delete {
        /// Name of the account
        account: String,
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
        Some(Commands::Add { account, secret }) => {
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

            // Validate before saving
            TfaTotp::from_input(&account, &raw_secret)?;
            store.set_secret(&account, &raw_secret)?;
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
        Some(Commands::Completions { shell }) => {
            generate_completions(shell, out_writer);
        }
        None => {
            writeln!(out_writer, "Use --help for usage instructions.")?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let store = KeyringStore::default();
    let clipboard = SystemClipboard;
    run_app(cli, &store, &clipboard, &mut io::stdout())
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
        assert!(cli.unwrap().command.is_none());
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
}
