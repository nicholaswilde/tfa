use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "tfa")]
#[command(author, version = env!("PROJECT_VERSION"), about = "CLI Two-Factor Authentication (2FA) tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a TOTP token for an account
    Generate {
        /// Name of the account
        account: String,
    },
    /// Add a new account secret
    Add {
        /// Name of the account
        account: String,
        /// TOTP secret key
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate { account }) => {
            println!("Generate TOTP for {account}");
        }
        Some(Commands::Add { account, .. }) => {
            println!("Add account {account}");
        }
        Some(Commands::List) => {
            println!("Listing accounts");
        }
        Some(Commands::Delete { account }) => {
            println!("Delete account {account}");
        }
        None => {
            println!("Use --help for usage instructions.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
