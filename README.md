# :lock: tfa :key:

[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/tfa/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/tfa/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](LICENSE)

A high-performance, single-binary CLI Two-Factor Authentication (2FA / TOTP) application written in Rust. It provides secure credential storage via your system keyring, instant token generation, and shell autocompletions.

---

## :sparkles: Features

- **Security First:** Credentials and secret seeds reside securely in your OS keychain/keyring (`secret-service`/D-Bus on Linux, Keychain on macOS, Windows Credential Manager).
- **Headless Fallback:** Automatically falls back to secure user-only file storage (`0600` permissions at `~/.local/share/tfa/credentials.json`) when headless or without active Secret Service daemon. Configurable via `--storage <auto|keyring|file>` or `TFA_STORAGE`.
- **RFC 6238 Compliant:** Standard time-based one-time passwords (TOTP) powered by [`totp-rs`](https://github.com/constantoine/totp-rs).
- **QR Code Support:** Render terminal-native ANSI/Unicode block QR codes for mobile authenticator scanning (`tfa qr`), and scan QR code images (`tfa add --qr`) to import accounts automatically.
- **Flexible Secret Input:** Supports raw Base32 secret keys and standard `otpauth://totp/...` URIs.
- **Clipboard Integration:** Instantly copy tokens to your system clipboard with `-c` / `--copy`.
- **Shell Completions:** Built-in completion generation for Bash, Zsh, Fish, PowerShell, and Elvish via `clap_complete`.
- **Lightweight & Portable:** Static, zero-fluff binary compiled for AMD64, ARM64, ARMv7, ARMv6, macOS, and Windows.

---

## :rocket: Installation & Build

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (1.88+)
- [Task](https://taskfile.dev/) (optional, recommended)

### Build Locally

```bash
task build:local
# Or directly with Cargo:
cargo build --release
```

The compiled binary will be placed at `target/release/tfa`.

---

## :keyboard: Usage

```bash
tfa [COMMAND]
```

### Commands

| Command | Description | Example |
| :--- | :--- | :--- |
| `generate` | Generate current 6-digit TOTP token | `tfa generate github` |
| `generate -c` | Generate token and copy to clipboard | `tfa generate github -c` |
| `generate -w` | Watch token countdown with progress bar | `tfa generate github -w` |
| `watch` | Watch live TOTP countdown with progress bar | `tfa watch github [-c]` |
| `add` | Add account secret or `otpauth://` URI | `tfa add github JBSWY3DPEHPK3PXP` |
| `qr` | Render account as terminal QR code | `tfa qr github [--light]` |
| `list` | List all configured accounts | `tfa list` |
| `show` | Display account parameters and stored secret | `tfa show github [-r]` |
| `export` | Export accounts to encrypted backup or stdout | `tfa export backup.enc` |
| `import` | Import accounts from backup, migration URI, or JSON | `tfa import backup.enc` |
| `delete` | Remove an account secret from keyring | `tfa delete github` |
| `completions` | Output shell completion script | `tfa completions zsh` |

### Adding Accounts

Pass the Base32 secret or `otpauth://` URI directly:

```bash
# Using Base32 secret
tfa add github JBSWY3DPEHPK3PXP

# Using OTPAuth URI
tfa add google "otpauth://totp/Google:user@gmail.com?secret=JBSWY3DPEHPK3PXP&issuer=Google"

# Interactive prompt (avoids shell history leaks)
tfa add aws
Enter TOTP secret for 'aws':

# Import directly from a QR code image / screenshot:
tfa add --qr /path/to/qr_screenshot.png
tfa add my_account --qr /path/to/qr_screenshot.png
```

### QR Code Display

Render any account as a 2D ANSI/Unicode block QR code in the terminal to easily pair mobile authenticator apps:

```bash
# Standard dark-background terminal (default)
tfa qr github

# Light-background terminal
tfa qr github --light
```

### Generating Tokens

```bash
$ tfa generate github
482910 (24s remaining)

# With automatic clipboard copy:
$ tfa generate github -c
482910 (24s remaining) [copied]

# Live watch mode with dynamic progress bar:
$ tfa watch github
github  482910  [████████░░░░] 18s

# Or via generate flag:
$ tfa generate github -w -c
```

### Backup, Export & Import

`tfa` provides secure backup and restore with Argon2id + AES-256-GCM encryption, as well as seamless migration from other 2FA applications.

#### Exporting Accounts
```bash
# Export encrypted backup (prompts for master passphrase):
tfa export backup.enc

# Export unencrypted JSON to stdout or file:
tfa export backup.json --unencrypted
```

#### Importing Accounts
```bash
# Restore from an encrypted backup file:
tfa import backup.enc

# Import directly from Google Authenticator QR migration link:
tfa import "otpauth-migration://offline?data=..."

# Import from Bitwarden or Aegis JSON export:
tfa import bitwarden_export.json

# Preview changes with dry-run:
tfa import backup.enc --dry-run

# Handle conflicts (overwrite existing or skip):
tfa import backup.enc --overwrite
tfa import backup.enc --skip
```

---

## :books: Shell Completions

Generate and source completions for your shell:

### Bash
```bash
tfa completions bash > ~/.local/share/bash-completion/completions/tfa
```

### Zsh
```bash
tfa completions zsh > ~/.zfunc/_tfa
# Ensure fpath includes ~/.zfunc in your .zshrc:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit
```

### Fish
```bash
tfa completions fish > ~/.config/fish/completions/tfa.fish
```

Pre-generated completion scripts are also available in the [`completions/`](completions/) folder.

---

## :test_tube: Development & Quality Gates

Use `task` as the primary command runner:

```bash
task check        # Type check code
task test         # Run formatting, clippy lints, and unit tests
task test:ci      # Run full suite using CI profile
task lint         # Run clippy with -D warnings
task fmt          # Auto-format with rustfmt
task coverage     # Run tests and output coverage summary
task build        # Build all multi-arch Linux targets via cross
```

---

## :balance_scale: License

[Apache License 2.0](LICENSE)

## :writing_hand: Author

Nicholas Wilde.
