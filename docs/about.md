# About

`tfa` is a fast, single-binary Rust CLI Two-Factor Authentication (2FA / TOTP / HOTP / Steam Guard) application built for secure credential storage, instant token generation, and scriptable terminal workflows. Its command design takes inspiration from [`totp-cli`][1] while adding native OS keyring integration, `age`-encrypted storage, terminal QR scanning/rendering, encrypted backups, and security auditing.

## :sparkles: Features

- **Security First:** Credentials and secret seeds reside securely in your OS keychain/keyring (`secret-service`/D-Bus on Linux, Keychain on macOS, Windows Credential Manager).
- **Headless Fallback & `age` Storage:** Automatically falls back to user-only file storage (`0600` permissions at `~/.local/share/tfa/credentials.json`) when headless or without an active Secret Service daemon, or stores credentials in an [`age`][2]-encrypted file (`~/.local/share/tfa/credentials.age`). Configurable via `--storage <auto|keyring|file|age>` or `TFA_STORAGE`.
- **RFC 6238 & RFC 4226 Compliant:** Standard time-based one-time passwords (TOTP), counter-based HOTP, and 5-character Steam Guard codes powered by [`totp-rs`][3].
- **Live Watch & Countdown:** Watch an individual token or all accounts in a namespace in real-time with continuous progress bars (`tfa watch`, `tfa watch all`).
- **QR Code Support:** Render terminal-native ANSI/Unicode block QR codes for mobile authenticator scanning (`tfa qr`), export batch Google Authenticator migration QR codes (`tfa qr --all`), and scan QR code images (`tfa add --qr`, `tfa instant --qr`).
- **Flexible Secret Input:** Supports raw Base32 secret keys, standard `otpauth://totp/...` URIs, stdin pipes, and environment variables.
- **Clipboard Integration:** Instantly copy tokens to your system clipboard with `-c` / `--copy`, OSC 52 terminal escape sequences, or custom commands (`wl-copy`, `xclip`), with optional auto-clear timers.
- **Multi-Namespace Isolation:** Organize personal, home, and work 2FA accounts into segregated namespaces with `-n` / `--namespace` and `-A` / `--all-namespaces`.
- **Encrypted Backups & Multi-Vendor Import:** Export backups encrypted with Argon2id + AES-256-GCM or GPG/OpenPGP, and import directly from Google Authenticator, 2FAS, Aegis, Ente Auth, Bitwarden, 1Password, LastPass, Proton, andOTP, Raivo OTP, Stratum, and `totp-cli`.
- **Dynamic Shell Completions:** Dynamic `<TAB>` completion of stored accounts and namespaces across Bash, Zsh, Fish, PowerShell, and Elvish via `clap_complete`.
- **Lightweight & Portable:** Static, zero-fluff binary compiled for AMD64, ARM64, ARMv7, ARMv6, macOS, and Windows.

## :shield: Security Architecture

- **Zero Plaintext Config Secrets:** Configuration files (`config.{toml,yaml,yml,json}`) hold only non-secret preferences; account seeds live exclusively in the configured storage backend.
- **Strict Permissions:** Local credential, auth, and session files enforce `0600` permissions (`0700` on directories) and can be verified anytime with [`tfa audit`](./security.md#security-audit) and [`tfa doctor`](./security.md#system-diagnostics).
- **Optional Master Password:** Protect vault access with a master password and sliding-window unlock session cache via [`tfa password`](./security.md#password-protection-unlock-caching).
- **In-Memory Backend Migration:** Move secrets directly across `keyring`, `file`, and `age` backends via [`tfa migrate`](./configuration.md#storage-backend-migration) without writing intermediate plaintext files.

[1]: https://github.com/yitsushi/totp-cli
[2]: https://github.com/FiloSottile/age
[3]: https://github.com/constantoine/totp-rs
