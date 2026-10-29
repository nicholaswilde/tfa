# :lock: tfa :key:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/tfa/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/tfa?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/tfa/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/tfa/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](LICENSE)

A high-performance, single-binary CLI Two-Factor Authentication (2FA / TOTP) application written in Rust. It provides secure credential storage via your system keyring, instant token generation, and shell autocompletions.

> [!WARNING]
> This project is currently in a `v0.X.X` development stage. Features and configurations are subject to change, and breaking changes may be introduced at any time.

> [!IMPORTANT]
> It is strongly recommended to use a secondary 2FA application to hold your tokens and to regularly test both the encryption and decryption of your backups.

---

## :sparkles: Features

- **Security First:** Credentials and secret seeds reside securely in your OS keychain/keyring (`secret-service`/D-Bus on Linux, Keychain on macOS, Windows Credential Manager).
- **Headless Fallback:** Automatically falls back to secure user-only file storage (`0600` permissions at `~/.local/share/tfa/credentials.json`) when headless or without active Secret Service daemon. Configurable via `--storage <auto|keyring|file|age>` or `TFA_STORAGE`.
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

When executed without a command (`tfa`), it runs the default command (`list`, or configured via `default_command` / `TFA_DEFAULT_COMMAND`).

### Commands

| Command | Description | Example |
| :--- | :--- | :--- |
| `generate` (`gen`, `g`) | Generate current TOTP token (interactive fuzzy search if omitted) | `tfa generate [account] [--json]` |
| `generate -c` | Generate token and copy to clipboard | `tfa generate [account] -c` |
| `generate -q` | Emit raw token only (auto-defaults when piped) | `tfa generate [account] -q` |
| `generate -w` | Watch token countdown with progress bar | `tfa generate [account] -w` |
| `instant` (`ins`) | Generate token instantly from secret, URI, or stdin without storage | `tfa instant [secret_or_uri]` |
| `watch` | Watch live TOTP countdown with progress bar | `tfa watch [account] [-c]` |
| `watch all` | Watch all passwords in active namespace | `tfa watch all` |
| `add` | Add account secret or `otpauth://` URI | `tfa add github JBSWY3DPEHPK3PXP` |
| `edit` (`set`) | Edit parameters or prefix of an existing account in-place | `tfa edit github --digits 8 --prefix 1234` |
| `qr` | Render account or batch migration QR code for mobile scanning | `tfa qr [account] [--all] [--light]` |
| `list` (`ls`) | List all configured accounts | `tfa list [--codes] [--json]` |
| `status` | Live overview of all accounts and tokens | `tfa status [--json]` |
| `show` | Display account parameters, raw secret, or live OTP code | `tfa show github [-r] [-t] [--json]` |
| `export` | Export accounts to encrypted backup or stdout (JSON, OTPAuth, YAML) | `tfa export [--format <json|otpauth|yaml>] [file]` |
| `import` | Import accounts from backup, migration URI, or JSON | `tfa import backup.enc` |
| `migrate` | In-memory migration between storage backends (`keyring`, `file`, `age`) | `tfa migrate --from keyring --to age` |
| `delete` (`del`, `rm`, `remove`) | Remove an account secret (interactive fuzzy search if omitted) | `tfa delete [account]` |
| `rename` (`ren`) | Rename an account within the active namespace | `tfa rename <old_name> <new_name>` |
| `move` (`mv`) | Relocate an account to a different namespace | `tfa move <account> --to <target_ns>` |
| `password` (`pwd`, `pw`) | Manage master password protection and session unlock caching (`add`, `remove`, `lock`, `unlock`, `status`) | `tfa password status` |
| `audit` | Scan stored accounts and credentials file for security vulnerabilities | `tfa audit [--json]` |
| `namespace` (`ns`) | Multi-tenant namespace management (`current`, `default`, `list`/`ls`, `create`, `rename`/`ren`/`mv`, `delete`/`del`/`rm`, `set-default`/`use`/`switch`) | `tfa ns` |
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

# Add account with a static PIN / prefix prepended to generated OTP tokens:
tfa add vpn JBSWY3DPEHPK3PXP --prefix 1234

# Edit an existing account's parameters or prefix in-place:
tfa edit vpn --digits 8 --period 60 --prefix 9999
tfa edit vpn --clear-prefix
```

### QR Code Display

Render any account or batch-export accounts as terminal ANSI/Unicode block QR codes to easily pair mobile authenticator apps:

```bash
# Standard dark-background terminal (default)
tfa qr github

# Light-background terminal
tfa qr github --light

# Export all accounts in active namespace as Google Authenticator migration QR code(s):
tfa qr --all # or tfa qr --migration
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

# Copy token and automatically clear clipboard after 15 seconds (or default 30s if value omitted):
$ tfa generate github -c --clear-after 15
$ tfa generate github -c --clear-after

# Explicitly disable automatic clipboard clearing when enabled in config:
$ tfa generate github -c --no-clear
$ tfa generate github -c --clear-after 0

# Clean raw token output without decorations (-q / --quiet / --token-only):
$ tfa generate github -q
482910

# TTY auto-detection: piped or scripted output automatically defaults to quiet token-only:
$ TOKEN=$(tfa generate github)
$ echo "$TOKEN"
482910

# Explicitly override quiet mode when piped:
$ tfa generate github --quiet=false | cat
482910 (24s remaining)

# Preview upcoming token for next time step (with warning indicator when expiring soon):
$ tfa generate github --next
482910 (24s remaining) (next: 192837)
# When expiring within 5s:
# 482910 (3s remaining) [expiring soon] (next: 192837)

# Generate token with relative time offset (clock skew) or explicit Unix timestamp:
$ tfa generate github --skew -30
$ tfa generate github --skew +30
$ tfa generate github --at 1730000000

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

# Clean raw token output without decorations (-q / --quiet / --token-only)
tfa instant JBSWY3DPEHPK3PXP -q

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

# Instant at specific Unix timestamp or relative clock skew
tfa instant JBSWY3DPEHPK3PXP --at 1730000000
tfa instant JBSWY3DPEHPK3PXP --skew -30
```

### Viewing Account Details & Secrets

Inspect account parameters, extract the stored secret key/URI, or print the current live OTP code:

```bash
# Formatted account overview
$ tfa show github
Account:   github
Type:      TOTP
Digits:    6
Period:    30s
Algorithm: SHA1
Secret:    JBSWY3DPEHPK3PXP
Live Code: 482910 (24s remaining)

# Output only the current live OTP code (-t / --token / --code):
$ tfa show github -t
482910

# Output only the raw secret or URI (-r / --raw):
$ tfa show github -r
JBSWY3DPEHPK3PXP

# JSON format for scripts and integrations:
$ tfa show github --json
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

`tfa` provides secure backup and restore with Argon2id + AES-256-GCM symmetric encryption or GPG / OpenPGP public-key encryption, as well as seamless migration from other 2FA applications.

#### Exporting Accounts
```bash
# Export encrypted backup (prompts for master passphrase):
tfa export backup.enc

# Export encrypted with a GPG key fingerprint, key ID, or email:
tfa export backup.gpg --gpg 0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3

# Export using default GPG key fingerprint from config or TFA_GPG_FINGERPRINT:
tfa export backup.gpg --gpg

# Export unencrypted JSON to stdout or file:
tfa export backup.json --unencrypted

# Export unencrypted in JSON, OTPAuth URI list, or YAML format:
tfa export backup.json --unencrypted --format json
tfa export accounts.txt --unencrypted --format otpauth
tfa export dump.yaml --unencrypted --format yaml
```

#### Importing Accounts
```bash
# Restore from an encrypted backup file (auto-detects Argon2id/AES-GCM or GPG):
tfa import backup.enc
tfa import backup.gpg

# Import directly from Google Authenticator QR migration image or link:
tfa import google_qr.png
tfa import "otpauth-migration://offline?data=..."

# Import from 2FAS (*.2fas), totp-cli (YAML), Bitwarden (JSON/CSV), 1Password (CSV), Aegis, LastPass, Proton, andOTP, Raivo OTP, Ente Auth, or Stratum export:
tfa import backup.2fas
tfa import dump.yaml
tfa import bitwarden_export.json
tfa import 1password_export.csv
tfa import ente_export.txt

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

## :shield: Security Audit

Scan all stored accounts across namespaces and local credential storage files for potential security issues:
- **Weak Secrets (`[HIGH]`):** Base32 secret seeds shorter than 16 characters (< 128-bit entropy).
- **Duplicate Secrets (`[HIGH]`):** Identical secret seeds shared across multiple accounts or namespaces.
- **Corrupted Accounts (`[HIGH]`):** Stored secrets or `otpauth://` URIs that fail validation.
- **File Permissions & Ownership (`[HIGH]`):** Local `credentials.json` files with permissions other than `0600` or owned by a different user UID.
- **Legacy Hash Algorithms (`[WARN]`):** Accounts using `SHA1` instead of `SHA256` or `SHA512`.

```bash
# Run human-readable security audit report:
$ tfa audit
=== Security Audit Report ===

Accounts Scanned: 3
Issues Found: 2

Findings:
  [HIGH] default:legacy_vpn - Weak secret with 10 Base32 characters (< 128-bit / 16 Base32 characters)
  [WARN] default:github - Legacy SHA-1 algorithm in use (SHA-256 or SHA-512 recommended)

Summary:
  - Weak secrets: 1
  - Duplicate secrets: 0
  - Legacy algorithms: 1
  - File permission/ownership issues: 0

# Output structured audit findings as JSON for CI/scripts:
$ tfa audit --json
```

---

## :books: Shell Completions

`tfa` features dynamic autocompletion for stored account names, configured namespaces, commands, and options. Shell scripts can be generated for Bash, Zsh, Fish, PowerShell, and Elvish:

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

### PowerShell
```powershell
tfa completions powershell >> $PROFILE
```

### Elvish
```elvish
tfa completions elvish >> ~/.elvish/rc.elv
```

> [!TIP]
> Dynamic completions automatically query stored account names and namespaces on `<TAB>` while failing gracefully if storage is locked. Pass `--static` to `tfa completions` to generate static shell completion scripts instead.

Pre-generated completion scripts are also available in the [`completions/`](completions/) folder.

---

## :gear: Configuration

`tfa` supports optional configuration files in **TOML**, **YAML**, or **JSON** format.

### Hierarchical Precedence
Settings resolve in strict hierarchical precedence order (highest to lowest):
1. **Command Line Arguments** (e.g. `--storage`, `--age-key`, `--ssh-key`, `--gpg`, `--clipboard`, `--clip-command`, `--clear-after`, `--no-clear`, `--namespace`, `--digits`, `--period`, `--algorithm`, `--copy`, `--light`)
2. **Environment Variables** (e.g. `TFA_STORAGE`, `TFA_AGE_KEY`, `TFA_AGE_KEY_FILE`, `TFA_SSH_KEY`, `TFA_SSH_KEY_FILE`, `TFA_GPG_FINGERPRINT`, `TFA_GPG_RECIPIENT`, `TFA_CLIPBOARD`, `TFA_CLIP_COMMAND`, `TFA_CLIPBOARD_TIMEOUT`, `TFA_CLIPBOARD_CLEAR`, `TFA_NAMESPACE`, `TFA_CONFIG`, `TFA_DIGITS`, `TFA_PERIOD`, `TFA_ALGORITHM`, `TFA_AUTO_COPY`, `TFA_LIGHT`)
3. **Configuration File** (`-C <path>` / `--config <path>`, or default locations: `~/.config/tfa/config.{toml,yaml,yml,json}`)
4. **Hardcoded Defaults** (storage: `auto`, clipboard: `system`, clipboard_timeout: disabled, namespace: `default`, digits: `6`, period: `30`, algorithm: `sha1`, auto_copy: `false`, light: `false`)

### Storage Backends & Headless Environments

`tfa` supports four storage options via `storage` in config, `--storage`, or `TFA_STORAGE`:

- **`auto` (default):** Attempts the OS system keyring first (Secret Service on Linux, Keychain on macOS, Credential Manager on Windows). If the keyring daemon is unreachable, locked, or running in a headless/SSH session (`SS error`), `tfa` displays a notice and automatically falls back to local user file storage (`0600` permissions at `~/.local/share/tfa/credentials.json`).
  ```
  [notice] Keyring unavailable (Failed saving secret in keyring: Couldn't access platform storage: SS error: result not returned from SS API); saving to local file store
  ```
- **`file`:** Directly reads and writes to `~/.local/share/tfa/credentials.json` without querying DBus or system keyring, silencing fallback notices in headless, SSH, or server environments.
- **`keyring`:** Strictly requires the OS system keyring and returns a hard error if unavailable.
- **`age`:** Identity-based encrypted file storage (`~/.local/share/tfa/credentials.age`) using [`age`](https://github.com/FiloSottile/age). Encrypts credentials with age public keys, SSH keys (`~/.ssh/id_ed25519.pub`, `~/.ssh/id_rsa.pub`), or a passphrase.

#### Age Storage Key Resolution
When using `--storage age`, `tfa` resolves decryption identities and encryption recipients in the following order:
1. Explicit age key argument or environment variable: `--age-key`, `TFA_AGE_KEY`, `TFA_AGE_KEY_FILE`, `AGE_KEY_FILE`, or `SOPS_AGE_KEY_FILE`.
2. Explicit SSH key file location: `--ssh-key` (`--ssh-key-file`, `--age-ssh-key`), `TFA_SSH_KEY_FILE`, `TFA_SSH_KEY`, or `ssh_key` in configuration.
3. SOPS age key fallback: `~/.config/sops/age/keys.txt` (or `%APPDATA%\sops\age\keys.txt`).
4. Default age key fallback: `~/.config/age/keys.txt` (or `$XDG_CONFIG_HOME/age/keys.txt`, `%APPDATA%\age\keys.txt`).
5. Default SSH keys: `~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, or any valid key pair in `~/.ssh/`.
6. Passphrase fallback: `TFA_AGE_PASSPHRASE` environment variable, or interactive prompt if no identity files exist.

### Storage Backend Migration

Transfer accounts directly across storage backends (`keyring`, `file`, `age`) in memory without intermediate plaintext exports:

```bash
# Migrate from keyring to age-encrypted storage
tfa migrate --from keyring --to age

# Preview migration without modifying destination or source
tfa migrate --from file --to keyring --dry-run

# Overwrite conflicting accounts in destination backend
tfa migrate --from keyring --to file --overwrite

# Transfer accounts and delete them from the source backend
tfa migrate --from file --to age --purge-source
```

### Example Configuration Files

#### TOML (`~/.config/tfa/config.toml`)

##### Flat Format
```toml
storage = "keyring"           # auto, keyring, file, or age
age_key = "~/.config/sops/age/keys.txt" # optional age key file or secret key
ssh_key = "~/.ssh/id_ed25519" # optional SSH private key file
gpg_fingerprint = "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3" # optional default GPG recipient key
clipboard = "system"          # system or osc52 (terminal ANSI escape sequence)
clip_command = "wl-copy"      # optional custom command receiving token via stdin
clipboard_clear = true        # enable or disable automatic clipboard clearing (true/false)
clipboard_timeout = 30        # clear copied token after N seconds (0 or false to disable)
default_namespace = "personal"# active default namespace (default: "default")
default_command = "list"      # default command when invoked without args (list, status, watch all)
digits = 6                    # 6 or 8
period = 30                   # 30 or 60
algorithm = "sha1"            # sha1, sha256, sha512, or steam
auto_copy = true              # automatically copy generated tokens
light = false                 # terminal QR light theme
password = true               # enable master password protection
password_timeout = 300        # unlock cache TTL in seconds (default: 300)
```

##### Nested Format
```toml
storage = "keyring"           # auto, keyring, file, or age
age_key = "~/.config/sops/age/keys.txt" # optional age key file or secret key
ssh_key = "~/.ssh/id_ed25519" # optional SSH private key file
gpg_fingerprint = "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3" # optional default GPG recipient key
clipboard = "system"          # system or osc52 (terminal ANSI escape sequence)
clip_command = "wl-copy"      # optional custom command receiving token via stdin
clipboard_clear = true        # enable or disable automatic clipboard clearing (true/false)
clipboard_timeout = 30        # clear copied token after N seconds (0 or false to disable)
default_namespace = "personal"# active default namespace (default: "default")
default_command = "list"      # default command when invoked without args (list, status, watch all)
digits = 6                    # 6 or 8
period = 30                   # 30 or 60
algorithm = "sha1"            # sha1, sha256, sha512, or steam
auto_copy = true              # automatically copy generated tokens
light = false                 # terminal QR light theme

[password]
enabled = true                # enable master password protection
timeout = 300                 # unlock cache TTL in seconds (default: 300)

[clipboard_timeout]
enabled = true                # enable or disable automatic clipboard clearing
timeout = 30                  # clear copied token after N seconds (default: 30)
```

#### YAML (`~/.config/tfa/config.yaml`)

##### Flat Format
```yaml
storage: auto
age_key: ~/.config/sops/age/keys.txt
ssh_key: ~/.ssh/id_ed25519
gpg_fingerprint: "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3"
clipboard: system
clip_command: "xclip -selection clipboard"
clipboard_clear: true
clipboard_timeout: 30
default_namespace: personal
default_command: list
digits: 6
period: 30
algorithm: sha1
auto_copy: true
light: false
password: true
password_timeout: 300
```

##### Nested Format
```yaml
storage: auto
age_key: ~/.config/sops/age/keys.txt
ssh_key: ~/.ssh/id_ed25519
gpg_fingerprint: "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3"
clipboard: system
clip_command: "xclip -selection clipboard"
clipboard_clear: true
clipboard_timeout: 30
default_namespace: personal
default_command: list
digits: 6
period: 30
algorithm: sha1
auto_copy: true
light: false

password:
  enabled: true
  timeout: 300
```

#### JSON (`~/.config/tfa/config.json`)
```json
{
  "storage": "auto",
  "age_key": "~/.config/sops/age/keys.txt",
  "ssh_key": "~/.ssh/id_ed25519",
  "gpg_fingerprint": "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3",
  "clipboard": "system",
  "clip_command": "wl-copy",
  "default_namespace": "personal",
  "default_command": "list",
  "digits": 6,
  "period": 30,
  "algorithm": "sha1",
  "auto_copy": true,
  "light": false,
  "password": {
    "enabled": true,
    "timeout": 300
  }
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

# View accounts across all namespaces with -A / --all-namespaces
tfa list -A
tfa list -A --codes
tfa status -A
tfa watch all -A
tfa export -A --unencrypted

# Show current active or configured default namespace
tfa ns
tfa ns current
tfa ns default

# List all namespaces (indicates default and active)
tfa ns list
tfa ns ls

# Create or rename namespaces
tfa ns create dev
tfa ns rename work office
tfa ns ren work office

# Rename an account within current namespace
tfa rename github gh
tfa ren github gh

# Move an account to another namespace
tfa move aws --to work
tfa mv personal_email -t home

# Switch active default namespace across sessions (persists to config file)
tfa ns set-default work
tfa ns use work

# Delete a namespace and its contained accounts
tfa ns delete office --force
tfa ns rm office -f
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
