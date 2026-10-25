# AGENTS.md

Instructions, conventions, and architectural context for AI agents working on `tfa`.

---

## 1. Project Overview & Architecture

**`tfa`** is a high-performance CLI Two-Factor Authentication (2FA / TOTP) application written in Rust. It provides secure credential storage, TOTP token generation, and account management from the terminal.

### Core Goals
- **Security First:** Secure credential and secret storage leveraging the system `keyring`. Never log or leak secrets.
- **Portability:** Static, single-binary distribution compiled across multiple architectures (AMD64, ARM64, ARMv7, ARMv6, macOS, Windows).
- **Simplicity & Speed:** Fast, minimal-overhead command execution with zero unnecessary dependencies (Ponytail discipline).
- **Token Efficiency:** Concise CLI output and help text for optimal terminal and agent consumption.

### Key Capabilities
- Manage 2FA accounts (add, list, update, remove) securely via system keyring.
- Generate time-based one-time passwords (TOTP) compliant with RFC 6238 via `totp-rs`.
- Interactive and automated CLI flows powered by `clap`.

### Reference Implementation
For feature parity, capabilities, design inspiration, or CLI conveniences, refer to:
- **Path:** `/home/nicholas/git/yitsushi/totp-cli`
- Consult `totp-cli` commands, flags, configuration structure, and feature set when planning or expanding `tfa`.

---

## 2. Technology Stack

- **Language & Edition:** Rust (Edition 2021)
- **CLI Parsing:** `clap` (with `derive`, `env` features)
- **TOTP Engine:** `totp-rs`
- **Keyring / Secure Storage:** `keyring`
- **Serialization:** `serde`, `serde_json`
- **Error Handling:** `anyhow` / `thiserror`
- **Observability:** `tracing`, `tracing-subscriber`
- **Build & Multi-Arch:** `cross` for cross-compilation, `build.rs` for git-derived `PROJECT_VERSION`
- **Automation:** `task` (`Taskfile.yml`)

---

## 3. Development Principles & Code Style

- **Build Discipline (Ponytail):** Minimal code that solves the problem. Reuse standard library where possible; avoid speculative abstractions.
- **Surgical Changes:** Modify only what is requested. Keep edits focused and verifiable.
- **Secret Hygiene:** Never write unencrypted secrets to disk, configs, or commit history. Passwords/keys reside in `keyring` or encrypted via SOPS if configured.
- **Code Quality:** All code must pass `cargo fmt -- --check` and `cargo clippy -- -D warnings`.

---

## 4. Testing & Quality Gates

### Test-Driven Development (TDD) Protocol
1. **Red Phase:** Write failing unit or integration tests reproducing the bug or specifying the new feature before writing any implementation code. Run `cargo test` to verify the failure.
2. **Green Phase:** Write the minimum necessary implementation code to pass the tests.
3. **Refactor Phase:** Refactor and clean up code while keeping all tests passing. Verify adherence to Ponytail discipline.

### Quality Gates
Before completing any task, ensure:
- [ ] `task fmt:check` passes without reformatting needed.
- [ ] `task lint` / `task clippy` passes with zero warnings (`-D warnings`).
- [ ] `task test` passes all unit and integration tests.
- [ ] Code coverage target remains high (>80%).

---

## 5. Development Commands (`Taskfile.yml`)

Use `task` as the primary development entry point:

```bash
task check        # Check for compilation errors
task test         # Run unit tests
task fmt          # Format code using rustfmt
task lint         # Run lints using clippy
task build:debug  # Build debug binary
task build:local  # Build release binary locally
task test:ci      # Run full CI suite (fmt, clippy, tests)
task build        # Build all multi-arch targets via cross
```
