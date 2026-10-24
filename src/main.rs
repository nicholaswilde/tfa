use clap::{Parser, Subcommand};
use std::io::{self, BufRead};

mod storage;
mod totp;

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
}

pub fn run_app(cli: Cli, store: &dyn AccountStore) -> anyhow::Result<()> {
    match cli.command {
        Some(Commands::Generate { account }) => {
            let secret = store.get_secret(&account)?;
            let engine = TfaTotp::from_input(&account, &secret)?;
            let code = engine.generate_current();
            let ttl = engine.ttl();
            println!("{code} ({ttl}s remaining)");
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
            println!("Account '{account}' added successfully.");
        }
        Some(Commands::List) => {
            let accounts = store.list_accounts()?;
            if accounts.is_empty() {
                println!("No accounts configured.");
            } else {
                for account in accounts {
                    println!("{account}");
                }
            }
        }
        Some(Commands::Delete { account }) => {
            store.delete_account(&account)?;
            println!("Account '{account}' deleted.");
        }
        None => {
            println!("Use --help for usage instructions.");
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let store = KeyringStore::default();
    run_app(cli, &store)
}

#[cfg(test)]
mod tests {
    use super::*;
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
            Some(Commands::Generate { account }) => assert_eq!(account, "github"),
            _ => panic!("Expected generate subcommand"),
        }
    }

    #[test]
    fn test_run_app_add_and_generate_flow() {
        let store = MemoryStore::new();

        // Add
        let add_cli =
            Cli::try_parse_from(["tfa", "add", "test_acc", "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"])
                .unwrap();
        assert!(run_app(add_cli, &store).is_ok());

        // List
        let list_cli = Cli::try_parse_from(["tfa", "list"]).unwrap();
        assert!(run_app(list_cli, &store).is_ok());

        // Generate
        let gen_cli = Cli::try_parse_from(["tfa", "generate", "test_acc"]).unwrap();
        assert!(run_app(gen_cli, &store).is_ok());

        // Delete
        let del_cli = Cli::try_parse_from(["tfa", "delete", "test_acc"]).unwrap();
        assert!(run_app(del_cli, &store).is_ok());

        // Generate deleted account should fail
        let gen_deleted = Cli::try_parse_from(["tfa", "generate", "test_acc"]).unwrap();
        assert!(run_app(gen_deleted, &store).is_err());
    }

    #[test]
    fn test_run_app_list_empty() {
        let store = MemoryStore::new();
        let list_cli = Cli::try_parse_from(["tfa", "list"]).unwrap();
        assert!(run_app(list_cli, &store).is_ok());
    }

    #[test]
    fn test_run_app_delete_missing_fails() {
        let store = MemoryStore::new();
        let del_cli = Cli::try_parse_from(["tfa", "delete", "missing"]).unwrap();
        assert!(run_app(del_cli, &store).is_err());
    }

    #[test]
    fn test_run_app_none_command() {
        let store = MemoryStore::new();
        let none_cli = Cli::try_parse_from(["tfa"]).unwrap();
        assert!(run_app(none_cli, &store).is_ok());
    }
}
