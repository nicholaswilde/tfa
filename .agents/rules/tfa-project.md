# TFA Project Rules

- **Test-Driven Development (TDD)**: Strictly mandatory. Always write failing tests first (Red), implement minimum code to pass (Green), then refactor. Never write unverified implementation code without prior failing tests.
- **Stack Constraints**: Must use `clap` (derive/env), `totp-rs`, and `keyring`. Do not swap or introduce alternative CLI/TOTP/storage engines without explicit approval.
- **Multi-Architecture**: Always preserve cross-compilation compatibility via `cross` across target triples (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `armv7-unknown-linux-musleabihf`, `arm-unknown-linux-gnueabihf`, macOS, and Windows).
- **Standard Stack Tools**:
  - Task runner: `task` (`Taskfile.yml`).
  - Secret safety: Never commit plain secrets, tokens, or seeds.
  - Profiles: Keep `[profile.release]` size-optimized (`opt-level = "z"`, `lto = true`, `strip = true`).
