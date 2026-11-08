//! Two-Factor Authentication (2FA) CLI application entrypoint.
//!
//! A thin binary dispatch wrapper delegating execution to the `tfa` library crate.

fn main() -> anyhow::Result<()> {
    tfa::run()
}
