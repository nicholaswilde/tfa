use crate::clipboard::Clipboard;
use crate::storage::AccountStore;
use crate::totp::TfaTotp;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::Write;
use std::time::Duration;

pub fn format_progress_bar(remaining: u64, total: u64, width: usize) -> String {
    if total == 0 {
        return format!("[{}] 0s", "░".repeat(width));
    }
    let clamped_rem = remaining.min(total);
    let filled = ((clamped_rem as f64 / total as f64) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:2}s",
        "█".repeat(filled),
        "░".repeat(empty),
        clamped_rem
    )
}

struct RawModeGuard {
    enabled: bool,
}

impl RawModeGuard {
    fn new() -> Self {
        let enabled = enable_raw_mode().is_ok();
        Self { enabled }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = disable_raw_mode();
        }
    }
}

/// Run watch mode for an account. Updates live countdown and code every interval.
/// Exits cleanly when 'q', Esc, or Ctrl+C is pressed, or if max_ticks is reached (for testing).
pub fn watch_account(
    account: &str,
    store: &dyn AccountStore,
    clipboard: &dyn Clipboard,
    auto_copy: bool,
    out: &mut dyn Write,
    max_ticks: Option<usize>,
) -> Result<()> {
    let secret = store.get_secret(account)?;
    let engine = TfaTotp::from_input(account, &secret)?;
    watch_engine(account, &engine, clipboard, auto_copy, out, max_ticks)
}

/// Run watch mode for a TOTP engine instance.
pub fn watch_engine(
    label: &str,
    engine: &TfaTotp,
    clipboard: &dyn Clipboard,
    auto_copy: bool,
    out: &mut dyn Write,
    max_ticks: Option<usize>,
) -> Result<()> {
    if engine.is_hotp() {
        anyhow::bail!("Watch mode is not supported for counter-based (HOTP) accounts");
    }
    let total_step = engine.step();

    // Guard raw mode so terminal state is always restored
    let _guard = RawModeGuard::new();

    // Hide cursor and clear line start
    write!(out, "\x1b[?25l")?;
    out.flush()?;

    let mut last_code = String::new();
    let mut ticks = 0;

    let res = (|| -> Result<()> {
        loop {
            let code = engine.generate_current();
            let remaining = engine.ttl();

            if code != last_code {
                if auto_copy {
                    let _ = clipboard.set_text(&code);
                }
                last_code = code.clone();
            }

            let bar = format_progress_bar(remaining, total_step, 12);
            let copy_indicator = if auto_copy { " [copied]" } else { "" };

            // \r = beginning of line, \x1b[2K = clear current line
            write!(
                out,
                "\r\x1b[2K{}  {}  {}{}",
                label, code, bar, copy_indicator
            )?;
            out.flush()?;

            ticks += 1;
            if let Some(max) = max_ticks {
                if ticks >= max {
                    break;
                }
            }

            // Poll for keypress with 250ms timeout for smooth countdown
            match event::poll(Duration::from_millis(250)) {
                Ok(true) => {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.code == KeyCode::Char('q')
                            || key.code == KeyCode::Esc
                            || (key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL))
                        {
                            break;
                        }
                    }
                }
                Ok(false) => {}
                Err(_) => {
                    if max_ticks.is_none() {
                        std::thread::sleep(Duration::from_millis(250));
                    }
                }
            }
        }
        Ok(())
    })();

    // Restore cursor and print newline
    let _ = write!(out, "\x1b[?25h\r\n");
    let _ = out.flush();

    res
}

/// Run watch mode for all accounts in the active store/namespace.
/// Updates live countdowns and codes every interval in a formatted table.
/// Exits cleanly when 'q', Esc, or Ctrl+C is pressed, or if max_ticks is reached (for testing).
pub fn watch_all(
    store: &dyn AccountStore,
    out: &mut dyn Write,
    max_ticks: Option<usize>,
) -> Result<()> {
    let accounts = store.list_accounts()?;
    if accounts.is_empty() {
        writeln!(out, "No accounts configured.")?;
        return Ok(());
    }

    let mut entries = Vec::new();
    for acc in &accounts {
        let engine_res = store
            .get_secret(acc)
            .and_then(|sec| TfaTotp::from_input(acc, &sec));
        entries.push((acc.clone(), engine_res));
    }

    let max_acc_len = entries
        .iter()
        .map(|(a, _)| a.len())
        .max()
        .unwrap_or(7)
        .max(7);

    let total_lines = 1 + entries.len();

    // Guard raw mode so terminal state is always restored
    let _guard = RawModeGuard::new();

    // Hide cursor
    write!(out, "\x1b[?25l")?;
    out.flush()?;

    let mut ticks = 0;
    let res = (|| -> Result<()> {
        loop {
            if ticks > 0 {
                write!(out, "\x1b[{}A\r", total_lines - 1)?;
            }

            write!(
                out,
                "\x1b[2K{:<width$}  {:^8}  PROGRESS\r\n",
                "ACCOUNT",
                "TOKEN",
                width = max_acc_len
            )?;

            for (i, (acc, engine_res)) in entries.iter().enumerate() {
                let is_last = i + 1 == entries.len();
                let (token, bar) = match engine_res {
                    Ok(engine) => {
                        let token = engine.generate_current();
                        let bar = if engine.is_hotp() {
                            "[counter-based]".to_string()
                        } else {
                            format_progress_bar(engine.ttl(), engine.step(), 12)
                        };
                        (token, bar)
                    }
                    Err(_) => ("ERROR".to_string(), "[invalid secret]".to_string()),
                };

                if is_last {
                    write!(
                        out,
                        "\x1b[2K{:<width$}  {:^8}  {}",
                        acc,
                        token,
                        bar,
                        width = max_acc_len
                    )?;
                } else {
                    write!(
                        out,
                        "\x1b[2K{:<width$}  {:^8}  {}\r\n",
                        acc,
                        token,
                        bar,
                        width = max_acc_len
                    )?;
                }
            }
            out.flush()?;

            ticks += 1;
            if let Some(max) = max_ticks {
                if ticks >= max {
                    break;
                }
            }

            match event::poll(Duration::from_millis(250)) {
                Ok(true) => {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.code == KeyCode::Char('q')
                            || key.code == KeyCode::Esc
                            || (key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL))
                        {
                            break;
                        }
                    }
                }
                Ok(false) => {}
                Err(_) => {
                    if max_ticks.is_none() {
                        std::thread::sleep(Duration::from_millis(250));
                    }
                }
            }
        }
        Ok(())
    })();

    let _ = write!(out, "\x1b[?25h\r\n");
    let _ = out.flush();

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard::MockClipboard;

    #[test]
    fn test_format_progress_bar_full_and_empty() {
        let full = format_progress_bar(30, 30, 10);
        assert_eq!(full, "[██████████] 30s");

        let half = format_progress_bar(15, 30, 10);
        assert_eq!(half, "[█████░░░░░] 15s");

        let empty = format_progress_bar(0, 30, 10);
        assert_eq!(empty, "[░░░░░░░░░░]  0s");
    }

    #[test]
    fn test_format_progress_bar_custom_step() {
        let bar = format_progress_bar(45, 60, 12);
        assert_eq!(bar, "[█████████░░░] 45s");
    }

    #[test]
    fn test_format_progress_bar_zero_total() {
        let bar = format_progress_bar(5, 0, 10);
        assert_eq!(bar, "[░░░░░░░░░░] 0s");
    }

    #[test]
    fn test_watch_account_success_with_autocopy() {
        let store = crate::storage::MemoryStore::new();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let res = watch_account("github", &store, &cb, true, &mut out, Some(2));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("github"));
        assert!(s.contains("[copied]"));
        assert_eq!(cb.text.lock().unwrap().as_deref().unwrap().len(), 6);
    }

    #[test]
    fn test_watch_account_without_autocopy() {
        let store = crate::storage::MemoryStore::new();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let res = watch_account("github", &store, &cb, false, &mut out, Some(1));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("github"));
        assert!(!s.contains("[copied]"));
        assert!(cb.text.lock().unwrap().is_none());
    }

    #[test]
    fn test_watch_account_hotp_error() {
        let store = crate::storage::MemoryStore::new();
        store
            .set_secret(
                "my_hotp",
                "otpauth://hotp/my_hotp?secret=JBSWY3DPEHPK3PXP&counter=1",
            )
            .unwrap();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let res = watch_account("my_hotp", &store, &cb, false, &mut out, Some(1));
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Watch mode is not supported for counter-based (HOTP) accounts"));
    }

    #[test]
    fn test_watch_account_missing() {
        let store = crate::storage::MemoryStore::new();
        let cb = MockClipboard::default();
        let mut out = Vec::new();

        let res = watch_account("nonexistent", &store, &cb, false, &mut out, Some(1));
        assert!(res.is_err());
    }

    #[test]
    fn test_watch_all_empty() {
        let store = crate::storage::MemoryStore::new();
        let mut out = Vec::new();
        let res = watch_all(&store, &mut out, Some(1));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("No accounts configured."));
    }

    #[test]
    fn test_watch_all_multiple_accounts() {
        let store = crate::storage::MemoryStore::new();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        store.set_secret("google", "JBSWY3DPEHPK3PXP").unwrap();
        let mut out = Vec::new();
        let res = watch_all(&store, &mut out, Some(2));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("ACCOUNT"));
        assert!(s.contains("TOKEN"));
        assert!(s.contains("PROGRESS"));
        assert!(s.contains("github"));
        assert!(s.contains("google"));
    }

    #[test]
    fn test_watch_all_with_hotp() {
        let store = crate::storage::MemoryStore::new();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        store
            .set_secret(
                "my_hotp",
                "otpauth://hotp/my_hotp?secret=JBSWY3DPEHPK3PXP&counter=1",
            )
            .unwrap();
        let mut out = Vec::new();
        let res = watch_all(&store, &mut out, Some(1));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("github"));
        assert!(s.contains("my_hotp"));
        assert!(s.contains("[counter-based]"));
    }

    #[test]
    fn test_watch_all_with_invalid_secret() {
        let store = crate::storage::MemoryStore::new();
        store.set_secret("bad_acc", "???not valid base32!").unwrap();
        let mut out = Vec::new();
        let res = watch_all(&store, &mut out, Some(1));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("bad_acc"));
        assert!(s.contains("ERROR"));
        assert!(s.contains("[invalid secret]"));
    }
}
