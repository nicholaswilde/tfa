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

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
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
    if engine.is_hotp() {
        anyhow::bail!("Watch mode is not supported for counter-based (HOTP) accounts");
    }
    let total_step = engine.step();

    // Guard raw mode so terminal state is always restored
    let _guard = RawModeGuard::new()?;

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
                account, code, bar, copy_indicator
            )?;
            out.flush()?;

            ticks += 1;
            if let Some(max) = max_ticks {
                if ticks >= max {
                    break;
                }
            }

            // Poll for keypress with 250ms timeout for smooth countdown
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q')
                        || key.code == KeyCode::Esc
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                    {
                        break;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
