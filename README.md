# :lock: tfa :key:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/tfa/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/tfa?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/tfa/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/tfa/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](LICENSE)

A high-performance, single-binary CLI Two-Factor Authentication (2FA / TOTP) application written in Rust. It provides secure credential storage via your system keyring, instant token generation, and shell autocompletions.

> [!WARNING]
> This project is currently in a `v0.X.X` development stage. Features and configurations are subject to change, and breaking changes may be introduced at any time.

---

## :sparkles: Features

- **Security First:** Credentials and secret seeds reside securely in your OS keychain/keyring (`secret-service`/D-Bus on Linux, Keychain on macOS, Windows Credential Manager).
- **Headless Fallback:** Automatically falls back to secure user-only file storage (`0600` permissions at `~/.local/share/tfa/credentials.json`) when headless or without active Secret Service daemon. Configurable via `--storage <auto|keyring|file>` or `TFA_STORAGE`.
- **RFC 6238 Compliant:** Standard time-based one-time passwords (TOTP) powered by [`totp-rs`](https://github.com/constantoine/totp-rs).
- **Live Watch & Countdown:** Watch an individual token or all accounts in a namespace in real-time with continuous progress bars (`tfa watch`, `tfa watch all`).
- **QR Code Support:** Render terminal-native ANSI/Unicode block QR codes for mobile authenticator scanning (`tfa qr`), and scan QR code images (`tfa add --qr`) to import accounts automatically.
- **Flexible Secret Input:** Supports raw Base32 secret keys and standard `otpauth://totp/...` URIs.
- **Clipboard Integration:** Instantly copy tokens to your system clipboard with `-c` / `--copy`.
- **Shell Completions:** Built-in completion generation for Bash, Zsh, Fish, PowerShell, and Elvish via `clap_complete`.
- **Lightweight & Portable:** Static, zero-fluff binary compiled for AMD64, ARM64, ARMv7, ARMv6, macOS, and Windows.

---

## :rocket: Installation & Build

### Homebrew

Install `tfa` via [Homebrew](https://brew.sh/) using the [nicholaswilde/homebrew-tap](https://github.com/nicholaswilde/homebrew-tap):

```bash
brew install nicholaswilde/homebrew-tap/tfa
```

Or tap the repository first:

```bash
brew tap nicholaswilde/homebrew-tap
brew install tfa
```

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
| `generate` (`gen`) | Generate current TOTP token (interactive fuzzy search if omitted) | `tfa generate [account] [--json]` |
| `generate -c` | Generate token and copy to clipboard | `tfa generate [account] -c` |
| `generate -w` | Watch token countdown with progress bar | `tfa generate [account] -w` |
| `instant` (`ins`) | Generate token instantly from secret, URI, or stdin without storage | `tfa instant [secret_or_uri]` |
| `watch` | Watch live TOTP countdown with progress bar | `tfa watch [account] [-c]` |
| `watch all` | Watch all passwords in active namespace | `tfa watch all` |
| `add` | Add account secret or `otpauth://` URI | `tfa add github JBSWY3DPEHPK3PXP` |
| `qr` | Render account as terminal QR code | `tfa qr github [--light]` |
| `list` | List all configured accounts | `tfa list [--codes] [--json]` |
| `status` | Live overview of all accounts and tokens | `tfa status [--json]` |
| `show` | Display account parameters and stored secret | `tfa show github [-r] [--json]` |
| `export` | Export accounts to encrypted backup or stdout | `tfa export backup.enc` |
| `import` | Import accounts from backup, migration URI, or JSON | `tfa import backup.enc` |
| `delete` | Remove an account secret (interactive fuzzy search if omitted) | `tfa delete [account]` |
| `rename` | Rename an account within the active namespace | `tfa rename <old_name> <new_name>` |
| `move` (`mv`) | Relocate an account to a different namespace | `tfa move <account> --to <target_ns>` |
| `password` (`pwd`, `pw`) | Manage master password protection and session unlock caching (`add`, `remove`, `lock`, `unlock`, `status`) | `tfa password status` |
| `namespace` (`ns`) | Multi-tenant namespace management (`list`, `create`, `rename`, `delete`, `set-default`) | `tfa ns list` |
| `completions` | Output shell completion script | `tfa completions zsh` |
| `man` | Output UNIX roff man page | `tfa man > assets/tfa.1` |

### Adding Accounts

Pass the Base32 secret or `otpauth://` URI directly:

```bash
# Using Base32 secret
tfa add github JBSWY3DPEHPK3PXP

# Using OTPAuth URI
tfa add google "otpauth://totp/Google:user@gmail.com?secret=JBSWY3DPEHPK3PXP&issuer=Google"

# Steam Guard account (5-character alphanumeric token):
tfa add steam_account JBSWY3DPEHPK3PXP -a steam

# Counter-based HOTP account (RFC 4226):
tfa add hotp_account GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ --counter 0

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

# Interactive fuzzy search (prompted when account is omitted in interactive terminal):
$ tfa generate
Select account to generate token |
> github
  aws
482910 (24s remaining)

# With automatic clipboard copy:
$ tfa generate github -c
482910 (24s remaining) [copied]

# Preview upcoming token for next time step (with warning indicator when expiring soon):
$ tfa generate github --next
482910 (24s remaining) (next: 192837)
# When expiring within 5s:
# 482910 (3s remaining) [expiring soon] (next: 192837)

# Programmatic JSON token output:
$ tfa generate github --json
{
  "account": "github",
  "token": "482910",
  "ttl": 24,
  "period": 30,
  "algorithm": "SHA1"
}

# HOTP token generation (increments stored counter automatically):
$ tfa generate hotp_account
755224 (counter: 0)

# Generate HOTP token for specific counter value:
$ tfa generate hotp_account --counter 5
254676 (counter: 5)

# Live watch mode with dynamic progress bar:
$ tfa watch github
github  482910  [████████░░░░] 18s

# Watch all passwords in the active namespace:
$ tfa watch all
ACCOUNT   TOKEN     PROGRESS
aws       789012    [████░░░░░░░░] 10s
github    482910    [████████░░░░] 18s
google    654321    [████████████] 30s

# Filter accounts in watch all using substring or glob patterns:
$ tfa watch all --filter "git*"
$ tfa watch --all -f "*dev*"

# Or via --all flag:
$ tfa watch --all

# Or via generate flag:
$ tfa generate github -w -c
```

### Stateless / Instant Token Generation

Generate tokens directly from a raw secret, `otpauth://` URI, stdin pipe, or QR image without saving to keyring or file storage:

```bash
# Instant from Base32 secret
tfa instant JBSWY3DPEHPK3PXP

# Instant with custom parameters
tfa instant JBSWY3DPEHPK3PXP -d 8 -p 60 -a sha256

# Instant from OTPAuth URI
tfa instant "otpauth://totp/Example:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Example"

# Reading from stdin pipe or dash
echo "JBSWY3DPEHPK3PXP" | tfa instant -
echo "JBSWY3DPEHPK3PXP" | tfa instant

# Stateless via generate flag
tfa generate --secret JBSWY3DPEHPK3PXP

# From environment variable (TOTP_TOKEN)
TOTP_TOKEN=JBSWY3DPEHPK3PXP tfa instant

# Instant from QR code image
tfa instant --qr /path/to/qr.png
```

### Dashboard & Status Overview

View all stored accounts, live TOTP tokens, and remaining seconds in a single table or JSON output (ideal for tmux, polybar, or waybar status modules):

```bash
# Formatted table overview
$ tfa status
# or
$ tfa list --codes

ACCOUNT  TOKEN    TTL
github   482910   18s
aws      891234   25s

# JSON output for scripts and status bars
$ tfa status --json
[
  {
    "account": "github",
    "token": "482910",
    "ttl": 18
  },
  {
    "account": "aws",
    "token": "891234",
    "ttl": 25
  }
]
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

## :lock: Password Protection & Unlock Caching

`tfa` supports optional master password protection with session caching similar to `sudo` or `gpg-agent`. When enabled, running `tfa` prompts for the master password, which remains unlocked for a configurable timeout period (default: 300 seconds / 5 minutes) via a sliding window session cache.

### Password Commands

```bash
# Set or update master password and enable protection:
tfa password add
# Or specify a custom timeout in seconds:
tfa password add --timeout 600

# View current password protection and unlock status:
tfa password status

# Lock the active session immediately (like 'sudo -k'):
tfa password lock

# Manually unlock the session:
tfa password unlock

# Remove password protection and delete stored master password:
tfa password remove
```

When password protection is enabled in the configuration file, any `tfa` command that accesses or generates credentials verifies that the session is unlocked or prompts for the password securely without leaking to shell history.

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

## :gear: Configuration

`tfa` supports optional configuration files in **TOML**, **YAML**, or **JSON** format.

### Hierarchical Precedence
Settings resolve in strict hierarchical precedence order (highest to lowest):
1. **Command Line Arguments** (e.g. `--storage`, `--clipboard`, `--clip-command`, `--namespace`, `--digits`, `--period`, `--algorithm`, `--copy`, `--light`)
2. **Environment Variables** (e.g. `TFA_STORAGE`, `TFA_CLIPBOARD`, `TFA_CLIP_COMMAND`, `TFA_NAMESPACE`, `TFA_CONFIG`, `TFA_DIGITS`, `TFA_PERIOD`, `TFA_ALGORITHM`, `TFA_AUTO_COPY`, `TFA_LIGHT`)
3. **Configuration File** (`-C <path>` / `--config <path>`, or default locations: `~/.config/tfa/config.{toml,yaml,yml,json}`)
4. **Hardcoded Defaults** (storage: `auto`, clipboard: `system`, namespace: `default`, digits: `6`, period: `30`, algorithm: `sha1`, auto_copy: `false`, light: `false`)

### Storage Backends & Headless Environments

`tfa` supports three storage options via `storage` in config, `--storage`, or `TFA_STORAGE`:

- **`auto` (default):** Attempts the OS system keyring first (Secret Service on Linux, Keychain on macOS, Credential Manager on Windows). If the keyring daemon is unreachable, locked, or running in a headless/SSH session (`SS error`), `tfa` displays a notice and automatically falls back to local user file storage (`0600` permissions at `~/.local/share/tfa/credentials.json`).
  ```
  [notice] Keyring unavailable (Failed saving secret in keyring: Couldn't access platform storage: SS error: result not returned from SS API); saving to local file store
  ```
- **`file`:** Directly reads and writes to `~/.local/share/tfa/credentials.json` without querying DBus or system keyring, silencing fallback notices in headless, SSH, or server environments.
- **`keyring`:** Strictly requires the OS system keyring and returns a hard error if unavailable.

### Example Configuration Files

#### TOML (`~/.config/tfa/config.toml`)
```toml
storage = "keyring"           # auto, keyring, or file
clipboard = "system"          # system or osc52 (terminal ANSI escape sequence)
clip_command = "wl-copy"      # optional custom command receiving token via stdin
default_namespace = "personal"# active default namespace (default: "default")
digits = 6                    # 6 or 8
period = 30                   # 30 or 60
algorithm = "sha1"            # sha1, sha256, sha512, or steam
auto_copy = true              # automatically copy generated tokens
light = false                 # terminal QR light theme
password = true               # enable master password protection
password_timeout = 300        # unlock cache TTL in seconds (default: 300)
```

#### YAML (`~/.config/tfa/config.yaml`)
```yaml
storage: auto
clipboard: system
clip_command: "xclip -selection clipboard"
default_namespace: personal
digits: 6
period: 30
algorithm: sha1
auto_copy: true
light: false
password: true
password_timeout: 300
```

#### JSON (`~/.config/tfa/config.json`)
```json
{
  "storage": "auto",
  "clipboard": "system",
  "clip_command": "wl-copy",
  "default_namespace": "personal",
  "digits": 6,
  "period": 30,
  "algorithm": "sha1",
  "auto_copy": true,
  "light": false,
  "password": true,
  "password_timeout": 300
}
```

---

## :card_index_dividers: Multi-Namespace Support

Organize personal, home, and work 2FA tokens into completely segregated namespaces:

```bash
# Scope any command to a namespace with -n / --namespace
tfa -n work add aws JBSWY3DPEHPK3PXP
tfa -n work list
tfa -n work generate aws
tfa -n work watch all

# List all namespaces
tfa ns list

# Create or rename namespaces
tfa ns create dev
tfa ns rename work office

# Rename an account within current namespace
tfa rename github gh

# Move an account to another namespace
tfa move aws --to work
tfa mv personal_email -t home

# Switch active default namespace across sessions (persists to config file)
tfa ns set-default work

# Delete a namespace and its contained accounts
tfa ns delete office --force
```

---

## :page_facing_up: Man Page

Generate or view the UNIX man page:

```bash
# Generate man page
task man
# Or with binary
tfa man > assets/tfa.1

# Preview formatted man page
man -l assets/tfa.1
```

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

​This project was started in 2026 by ​[Nicholas Wilde​](https://github.com/nicholaswilde)
