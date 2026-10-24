# :lock: tfa :key:

[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/tfa/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/tfa/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](LICENSE)

A high-performance, single-binary CLI Two-Factor Authentication (2FA / TOTP) application written in Rust. It provides secure credential storage via your system keyring, instant token generation, and shell autocompletions.

---

## :sparkles: Features

- **Security First:** Credentials and secret seeds reside securely in your OS keychain/keyring (`secret-service`/D-Bus on Linux, Keychain on macOS, Windows Credential Manager). Plaintext secrets are never written to disk or config files.
- **RFC 6238 Compliant:** Standard time-based one-time passwords (TOTP) powered by [`totp-rs`](https://github.com/constantoine/totp-rs).
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
| `add` | Add account secret or `otpauth://` URI | `tfa add github JBSWY3DPEHPK3PXP` |
| `list` | List all configured accounts | `tfa list` |
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
```

### Generating Tokens

```bash
$ tfa generate github
482910 (24s remaining)

# With automatic clipboard copy:
$ tfa generate github -c
482910 (24s remaining) [copied]
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
