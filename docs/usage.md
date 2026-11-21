# Usage

```bash
tfa [COMMAND]
```

When executed without a command (`tfa`), it runs the default command (`list`, or configured via `default_command` / `TFA_DEFAULT_COMMAND`).

## :keyboard: Commands

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
| `rename` (`ren`) | Rename an account (manual or auto-clean) or batch rename accounts | `tfa rename <old> [new] [-b] [-s <sep>] [--dry-run]` |
| `move` (`mv`) | Relocate an account to a different namespace | `tfa move <account> --to <target_ns>` |
| `password` (`pwd`, `pw`) | Manage master password protection and session unlock caching (`add`, `remove`, `lock`, `unlock`, `status`) | `tfa password status` |
| `audit` | Scan stored accounts and credentials file for security vulnerabilities | `tfa audit [--json]` |
| `doctor` (`doc`) | Validate system prerequisites, storage permissions, and diagnostic health | `tfa doctor [--json]` |
| `self-update` (`update`) | Check for and install the latest `tfa` release from GitHub | `tfa self-update [--check] [-y] [--version <ver>]` |
| `namespace` (`ns`) | Multi-tenant namespace management (`current`, `default`, `list`/`ls`, `create`, `rename`/`ren`/`mv`, `delete`/`del`/`rm`, `set-default`/`use`/`switch`) | `tfa ns` |
| `completions` | Output dynamic or static shell completion script (`bash`, `zsh`, `fish`, `powershell`, `elvish`) | `tfa completions zsh [--static]` |
| `man` | Output UNIX roff man page | `tfa man > assets/tfa.1` |

## :heavy_plus_sign: Adding & Editing Accounts

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

## :framed_picture: QR Code Display

Render any account or batch-export accounts as terminal ANSI/Unicode block QR codes to easily pair mobile authenticator apps:

```bash
# Standard dark-background terminal (default)
tfa qr github

# Light-background terminal
tfa qr github --light

# Export all accounts in active namespace as Google Authenticator migration QR code(s):
tfa qr --all # or tfa qr --migration
```

## :key: Generating & Watching Tokens

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

## :zap: Stateless / Instant Token Generation

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

## :mag: Viewing Account Details & Secrets

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

## :bar_chart: Dashboard & Status Overview

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

# Rename an account within current namespace (manual or auto-clean)
tfa rename github gh
tfa ren github gh
tfa rename github gh --dry-run
tfa rename "My Account" -s _
tfa rename "My Account" -b -s _

# Batch rename accounts to clean command-line friendly names (lowercase, replace spaces)
tfa rename --batch --dry-run
tfa rename --batch
tfa rename -b -s _
tfa -A rename --batch

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
