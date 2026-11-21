# :lock: tfa :key:

![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/nicholaswilde/tfa?style=for-the-badge)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/tfa/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/tfa/actions/workflows/ci.yml)
[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/tfa/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/tfa?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](../LICENSE)

A high-performance, single-binary CLI Two-Factor Authentication (2FA / TOTP) application written in Rust. It provides secure credential storage via your system keyring, `age` encryption, or headless file fallback, instant token generation, and dynamic shell autocompletions.

> [!WARNING]
> This project is currently in a `v0.X.X` development stage. Features and configurations are subject to change, and breaking changes may be introduced at any time.

> [!IMPORTANT]
> It is strongly recommended to use a secondary 2FA application to hold your tokens and to regularly test both the encryption and decryption of your backups.

---

## :rocket:&nbsp; TL;DR

### :floppy_disk:&nbsp; Installation

```bash
# Homebrew
brew install nicholaswilde/homebrew-tap/tfa
```

```powershell
# Scoop (Windows)
scoop bucket add nicholaswilde https://github.com/nicholaswilde/scoop-bucket
scoop install nicholaswilde/tfa
```

```bash
# Cargo (from source)
cargo install --git https://github.com/nicholaswilde/tfa
```

Debian/Ubuntu `.deb` packages are also published via [`nicholaswilde/apt`](https://nicholaswilde.io/apt) — see [Installation](https://nicholaswilde.io/tfa/installation/).

### :gear:&nbsp; Usage

```bash
# Add an account secret or otpauth:// URI
tfa add github JBSWY3DPEHPK3PXP

# Generate a TOTP code and copy to clipboard
tfa generate github -c

# Watch live countdowns for all accounts in the active namespace
tfa watch all

# View a status overview of all accounts and tokens
tfa status

# Validate system prerequisites, keyring, permissions, and NTP clock drift
tfa doctor

# Check for or install the latest GitHub release
tfa self-update --check
```

---

## :book:&nbsp; Documentation

Documentation can be found on the [tfa documentation site](https://nicholaswilde.io/tfa).

---

## :star:&nbsp; Contributing

See [Contributing](https://nicholaswilde.io/tfa/CONTRIBUTING/).

---

## :sparkles:&nbsp; Code of Conduct

See [Code of Conduct](https://nicholaswilde.io/tfa/CODE_OF_CONDUCT/).

---

## :balance_scale:&nbsp; License

[Apache License 2.0](../LICENSE)

---

## :pencil:&nbsp; Author

This project was started in 2026 by [Nicholas Wilde](https://github.com/nicholaswilde/).
