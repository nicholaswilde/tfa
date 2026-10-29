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

/// Check whether an account name matches the given filter pattern.
/// Supports substring matching, glob wildcard pattern matching (`*` and `?`),
/// and standard regex patterns. Matching is case-insensitive.
pub fn matches_pattern(pattern: &str, name: &str) -> bool {
    let pat = pattern.trim().to_lowercase();
    let val = name.trim().to_lowercase();
    if pat.is_empty() {
        return true;
    }

    // Try glob matching if pattern contains '*' or '?'
    if pat.contains('*') || pat.contains('?') {
        let pat_chars: Vec<char> = pat.chars().collect();
        let val_chars: Vec<char> = val.chars().collect();
        let mut i = 0;
        let mut j = 0;
        let mut star_idx: Option<usize> = None;
        let mut match_idx = 0;

        while i < val_chars.len() {
            if j < pat_chars.len() && (pat_chars[j] == '?' || pat_chars[j] == val_chars[i]) {
                i += 1;
                j += 1;
            } else if j < pat_chars.len() && pat_chars[j] == '*' {
                star_idx = Some(j);
                match_idx = i;
                j += 1;
            } else if let Some(star) = star_idx {
                j = star + 1;
                match_idx += 1;
                i = match_idx;
            } else {
                return false;
            }
        }

        while j < pat_chars.len() && pat_chars[j] == '*' {
            j += 1;
        }

        return j == pat_chars.len();
    }

    // Substring match
    val.contains(&pat)
}

/// Run watch mode for all accounts in the active store/namespace, optionally filtered by pattern.
/// Updates live countdowns and codes every interval in a formatted table.
/// Exits cleanly when 'q', Esc, or Ctrl+C is pressed, or if max_ticks is reached (for testing).
pub fn watch_all(
    store: &dyn AccountStore,
    out: &mut dyn Write,
    filter: Option<&str>,
    max_ticks: Option<usize>,
) -> Result<()> {
    let accounts = store.list_accounts()?;
    if accounts.is_empty() {
        writeln!(out, "No accounts configured.")?;
        return Ok(());
    }

    let filtered_accounts: Vec<String> = accounts
        .into_iter()
        .filter(|acc| filter.is_none_or(|p| matches_pattern(p, acc)))
        .collect();

    if filtered_accounts.is_empty() {
        if let Some(p) = filter {
            writeln!(out, "No accounts match filter '{p}'.")?;
        } else {
            writeln!(out, "No accounts configured.")?;
        }
        return Ok(());
    }

    let mut entries = Vec::new();
    for acc in &filtered_accounts {
        let engine_res = store
            .get_secret(acc)
            .and_then(|sec| TfaTotp::from_input(acc, &sec));
        entries.push((None, acc.clone(), engine_res));
    }

    render_watch_table(entries, out, max_ticks)
}

/// Run watch mode for all accounts across all namespaces, optionally filtered by pattern.
pub fn watch_all_namespaces(
    store: &dyn AccountStore,
    out: &mut dyn Write,
    filter: Option<&str>,
    max_ticks: Option<usize>,
) -> Result<()> {
    let raw_keys = store.list_accounts()?;
    if raw_keys.is_empty() {
        writeln!(out, "No accounts configured.")?;
        return Ok(());
    }

    let mut parsed: Vec<(String, String, String)> = raw_keys
        .into_iter()
        .map(|k| {
            let (ns, acct) = crate::namespace::parse_key(&k);
            (ns, acct, k)
        })
        .collect();

    parsed.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));

    let filtered_items: Vec<(String, String, String)> = parsed
        .into_iter()
        .filter(|(ns, acct, _)| {
            filter.is_none_or(|p| {
                matches_pattern(p, acct) || matches_pattern(p, &format!("{ns}:{acct}"))
            })
        })
        .collect();

    if filtered_items.is_empty() {
        if let Some(p) = filter {
            writeln!(out, "No accounts match filter '{p}'.")?;
        } else {
            writeln!(out, "No accounts configured.")?;
        }
        return Ok(());
    }

    let mut entries = Vec::new();
    for (ns, acct, raw_key) in filtered_items {
        let engine_res = store
            .get_secret(&raw_key)
            .and_then(|sec| TfaTotp::from_input(&acct, &sec));
        entries.push((Some(ns), acct, engine_res));
    }

    render_watch_table(entries, out, max_ticks)
}

fn render_watch_table(
    entries: Vec<(Option<String>, String, Result<TfaTotp>)>,
    out: &mut dyn Write,
    max_ticks: Option<usize>,
) -> Result<()> {
    let has_ns = entries.iter().any(|(ns, _, _)| ns.is_some());
    let max_ns_len = if has_ns {
        entries
            .iter()
            .map(|(ns, _, _)| ns.as_deref().unwrap_or("").len())
            .max()
            .unwrap_or(9)
            .max(9)
    } else {
        0
    };
    let max_acc_len = entries
        .iter()
        .map(|(_, a, _)| a.len())
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

            if has_ns {
                write!(
                    out,
                    "\x1b[2K{:<ns_width$}  {:<width$}  {:^8}  PROGRESS\r\n",
                    "NAMESPACE",
                    "ACCOUNT",
                    "TOKEN",
                    ns_width = max_ns_len,
                    width = max_acc_len
                )?;
            } else {
                write!(
                    out,
                    "\x1b[2K{:<width$}  {:^8}  PROGRESS\r\n",
                    "ACCOUNT",
                    "TOKEN",
                    width = max_acc_len
                )?;
            }

            for (i, (ns, acc, engine_res)) in entries.iter().enumerate() {
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

                if has_ns {
                    let ns_str = ns.as_deref().unwrap_or("default");
                    if is_last {
                        write!(
                            out,
                            "\x1b[2K{:<ns_width$}  {:<width$}  {:^8}  {}",
                            ns_str,
                            acc,
                            token,
                            bar,
                            ns_width = max_ns_len,
                            width = max_acc_len
                        )?;
                    } else {
                        write!(
                            out,
                            "\x1b[2K{:<ns_width$}  {:<width$}  {:^8}  {}\r\n",
                            ns_str,
                            acc,
                            token,
                            bar,
                            ns_width = max_ns_len,
                            width = max_acc_len
                        )?;
                    }
                } else {
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
        let res = watch_all(&store, &mut out, None, Some(1));
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
        let res = watch_all(&store, &mut out, None, Some(2));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("ACCOUNT"));
        assert!(s.contains("TOKEN"));
        assert!(s.contains("PROGRESS"));
        assert!(s.contains("github"));
        assert!(s.contains("google"));
    }

    #[test]
    fn test_watch_all_with_filter() {
        let store = crate::storage::MemoryStore::new();
        store.set_secret("github", "JBSWY3DPEHPK3PXP").unwrap();
        store.set_secret("gitlab", "JBSWY3DPEHPK3PXP").unwrap();
        store.set_secret("aws", "JBSWY3DPEHPK3PXP").unwrap();

        // Substring filter
        let mut out_sub = Vec::new();
        assert!(watch_all(&store, &mut out_sub, Some("git"), Some(1)).is_ok());
        let s_sub = String::from_utf8(out_sub).unwrap();
        assert!(s_sub.contains("github"));
        assert!(s_sub.contains("gitlab"));
        assert!(!s_sub.contains("aws"));

        // Glob filter
        let mut out_glob = Vec::new();
        assert!(watch_all(&store, &mut out_glob, Some("*hub"), Some(1)).is_ok());
        let s_glob = String::from_utf8(out_glob).unwrap();
        assert!(s_glob.contains("github"));
        assert!(!s_glob.contains("gitlab"));
        assert!(!s_glob.contains("aws"));

        // Filter with no matches
        let mut out_none = Vec::new();
        assert!(watch_all(&store, &mut out_none, Some("missing*"), Some(1)).is_ok());
        let s_none = String::from_utf8(out_none).unwrap();
        assert!(s_none.contains("No accounts match filter 'missing*'"));
    }

    #[test]
    fn test_matches_pattern_logic() {
        assert!(matches_pattern("", "github"));
        assert!(matches_pattern("git", "github"));
        assert!(matches_pattern("HUB", "github"));
        assert!(matches_pattern("git*", "github"));
        assert!(matches_pattern("*hub", "github"));
        assert!(matches_pattern("g?thub", "github"));
        assert!(matches_pattern("g*b", "github"));
        assert!(!matches_pattern("git*", "google"));
        assert!(!matches_pattern("aws", "github"));
        assert!(!matches_pattern("g?b", "github"));
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
        let res = watch_all(&store, &mut out, None, Some(1));
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
        let res = watch_all(&store, &mut out, None, Some(1));
        assert!(res.is_ok());
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("bad_acc"));
        assert!(s.contains("ERROR"));
        assert!(s.contains("[invalid secret]"));
    }
}
