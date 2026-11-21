# Security & Backups

## :inbox_tray: Backup, Export & Import

`tfa` provides secure backup and restore with Argon2id + AES-256-GCM symmetric encryption or GPG / OpenPGP public-key encryption, as well as seamless migration from other 2FA applications.

### Exporting Accounts

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

### Importing Accounts

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

## :lock: Password Protection & Unlock Caching

`tfa` supports optional master password protection with session caching similar to `sudo` or `gpg-agent`. When enabled, running `tfa` prompts for the master password, which remains unlocked for a configurable timeout period (default: 300 seconds / 5 minutes) via a sliding window session cache.

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

## :stethoscope: System Diagnostics

Validate system prerequisites, troubleshoot runtime issues, and check environment health with `tfa doctor`:

- **Keyring Connectivity:** Verify OS keychain/keyring (D-Bus / Secret Service on Linux, Keychain on macOS, Credential Manager on Windows) responsiveness.
- **Storage & Permissions:** Verify strict `0600` permissions on credentials/config files and `0700` on directories.
- **Clock Drift Detection:** Query NTP pool (`pool.ntp.org:123`) to measure local system time offset against true network time.
- **Clipboard Availability:** Detect system clipboard (`arboard`) and terminal OSC 52 fallback.
- **Age Prerequisites:** Validate age identity keys and environment configuration for `age` storage backend and backups.
- **Terminal Capabilities:** Check UTF-8 unicode block rendering for ANSI QR codes and color support.

```bash
# Run human-readable system diagnostic check:
tfa doctor

# Output structured diagnostics report as JSON:
tfa doctor --json

# Query custom NTP server:
tfa doctor --ntp-server time.cloudflare.com
```
