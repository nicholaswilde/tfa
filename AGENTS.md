# AGENTS.md

Conventions and architecture for AI agents working on `tfa`.

---

## 1. Project Overview & Architecture

**`tfa`**: Rust CLI Two-Factor Authentication (2FA / TOTP) app for secure credential storage, TOTP generation, and account management.

### Core Goals
- **Security First:** Store credentials/secrets via system `keyring`. Never log or leak secrets.
- **Portability:** Static single-binary across AMD64, ARM64, ARMv7, ARMv6, macOS, Windows.
- **Simplicity & Speed:** Fast execution, zero unnecessary dependencies (Ponytail discipline).
- **Token Efficiency:** Concise CLI output and help text.

### Key Capabilities
- Manage 2FA accounts (add, list, update, remove) via system keyring.
- Generate RFC 6238 TOTP codes via `totp-rs`.
- Interactive and automated CLI flows via `clap`.

### Reference Implementation
For feature parity, design, or CLI flags:
- **Path:** `/home/nicholas/git/yitsushi/totp-cli`
- Check `totp-cli` commands, flags, and config structure when expanding `tfa`.

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

- **Build Discipline (Ponytail):** Minimum code solving problem. Reuse stdlib; no speculative abstractions.
- **Surgical Changes:** Modify only requested code. Keep edits focused and verifiable.
- **Secret Hygiene:** Never write unencrypted secrets to disk, configs, or git history. Keep secrets in `keyring` or SOPS.
- **Configuration Parity:** When modifying config fields in `src/config.rs`, update all config examples in `README.md` (flat/nested TOML, flat/nested YAML, JSON) and `assets/config.example.toml`.
- **Code Quality:** Must pass `cargo fmt -- --check` and `cargo clippy -- -D warnings`.

---

## 4. Testing & Quality Gates

### Test-Driven Development (TDD) Protocol
1. **Red Phase:** Write failing unit/integration tests before implementation. Run `cargo test` to verify failure.
2. **Green Phase:** Write minimum code to pass tests.
3. **Refactor Phase:** Clean up code while keeping tests green and adhering to Ponytail discipline.

### Quality Gates
Before completing any task:
- [ ] `task fmt:check` passes.
- [ ] `task lint` / `task clippy` passes with zero warnings (`-D warnings`).
- [ ] `task test` passes all unit and integration tests.
- [ ] Code coverage stays >80%.

---

## 5. Development Commands (`Taskfile.yml`)

Use `task` as primary entry point:

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

## 6. RTK Command Guidelines
Follow global `~/.gemini/config/rules/rtk-rules.md`, `ripgrep-rules.md`, and `github-cli-rules.md`.

## What To Do Next
- When asked "what to do next", **check remote repository issues first** via `gh`:
  ```bash
  rtk gh issue list | cat
  ```
