use anyhow::Result;
use base64ct::{Base64, Encoding};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

/// Clipboard backend provider
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardProvider {
    /// Native system clipboard (arboard with Wayland/X11/macOS/Windows)
    #[default]
    System,
    /// Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)
    Osc52,
}

pub trait Clipboard {
    fn set_text(&self, text: &str) -> Result<()>;
}

/// System clipboard using arboard
#[derive(Default)]
pub struct SystemClipboard;

impl Clipboard for SystemClipboard {
    fn set_text(&self, text: &str) -> Result<()> {
        let mut cb = arboard::Clipboard::new()
            .map_err(|e| anyhow::anyhow!("Failed accessing system clipboard: {e}"))?;
        cb.set_text(text)
            .map_err(|e| anyhow::anyhow!("Failed setting clipboard text: {e}"))
    }
}

/// Terminal clipboard using OSC 52 escape sequences.
/// Supports standard terminals, tmux DCS escaping, and GNU Screen chunks.
#[derive(Default)]
pub struct Osc52Clipboard {
    /// Limit length of OSC 52 sequence (default 100_000 matching ChromiumOS osc52.sh)
    pub max_sequence: usize,
}

impl Osc52Clipboard {
    pub const DEFAULT_MAX_SEQUENCE: usize = 100_000;

    pub fn new() -> Self {
        Self {
            max_sequence: Self::DEFAULT_MAX_SEQUENCE,
        }
    }

    /// Formats the raw payload into the proper terminal escape sequence
    pub fn format_sequence(text: &str, max_sequence: usize) -> Result<String> {
        let b64 = Base64::encode_string(text.as_bytes());
        if max_sequence > 0 && b64.len() > max_sequence {
            anyhow::bail!(
                "Selection too long to send to terminal: {max_sequence} limit, {} attempted",
                b64.len()
            );
        }

        let osc_seq = format!("\x1b]52;c;{b64}\x07");

        let term = std::env::var("TERM").unwrap_or_default();
        let tmux_env = std::env::var("TMUX").is_ok();

        if term.starts_with("screen") || term.starts_with("tmux") {
            if tmux_env || term.starts_with("tmux") {
                // tmux DCS wrapper: \033Ptmux;\033<seq>\033\\
                // In tmux DCS, inner ESC characters must be doubled
                let inner = osc_seq.replace("\x1b", "\x1b\x1b");
                Ok(format!("\x1bPtmux;{inner}\x1b\\"))
            } else {
                // GNU Screen DCS wrapper with 256-byte chunking (252 bytes payload)
                const CHUNK_LIMIT: usize = 252;
                let mut screen_out = String::new();
                for chunk in osc_seq.as_bytes().chunks(CHUNK_LIMIT) {
                    let chunk_str = std::str::from_utf8(chunk)
                        .map_err(|e| anyhow::anyhow!("UTF-8 error in chunk: {e}"))?;
                    screen_out.push_str(&format!("\x1bP{chunk_str}\x1b\\"));
                }
                Ok(screen_out)
            }
        } else {
            Ok(osc_seq)
        }
    }
}

impl Clipboard for Osc52Clipboard {
    fn set_text(&self, text: &str) -> Result<()> {
        let seq = Self::format_sequence(text, self.max_sequence)?;
        // Write escape sequence directly to stderr so it doesn't pollute stdout redirects
        let mut stderr = io::stderr();
        stderr.write_all(seq.as_bytes())?;
        stderr.flush()?;
        Ok(())
    }
}

/// Dynamic clipboard dispatcher based on ClipboardProvider
pub struct DispatchingClipboard {
    provider: ClipboardProvider,
}

impl DispatchingClipboard {
    pub fn new(provider: ClipboardProvider) -> Self {
        Self { provider }
    }
}

impl Clipboard for DispatchingClipboard {
    fn set_text(&self, text: &str) -> Result<()> {
        match self.provider {
            ClipboardProvider::System => SystemClipboard.set_text(text),
            ClipboardProvider::Osc52 => Osc52Clipboard::new().set_text(text),
        }
    }
}

/// In-memory clipboard for testing
#[cfg(test)]
#[derive(Default)]
pub struct MockClipboard {
    pub text: std::sync::Mutex<Option<String>>,
}

#[cfg(test)]
impl Clipboard for MockClipboard {
    fn set_text(&self, text: &str) -> Result<()> {
        let mut lock = self.text.lock().unwrap();
        *lock = Some(text.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_clipboard_stores_text() {
        let cb = MockClipboard::default();
        assert!(cb.text.lock().unwrap().is_none());

        cb.set_text("123456").unwrap();
        assert_eq!(cb.text.lock().unwrap().as_deref(), Some("123456"));
    }

    #[test]
    fn test_osc52_standard_sequence() {
        // Without tmux or screen TERM
        std::env::remove_var("TMUX");
        std::env::set_var("TERM", "xterm-256color");

        let seq = Osc52Clipboard::format_sequence("123456", 100_000).unwrap();
        let expected_b64 = Base64::encode_string(b"123456");
        assert_eq!(seq, format!("\x1b]52;c;{expected_b64}\x07"));
    }

    #[test]
    fn test_osc52_tmux_dcs_sequence() {
        std::env::set_var("TMUX", "/tmp/tmux-1000/default,1234,0");
        std::env::set_var("TERM", "screen-256color");

        let seq = Osc52Clipboard::format_sequence("hello", 100_000).unwrap();
        assert!(seq.starts_with("\x1bPtmux;\x1b\x1b]52;c;"));
        assert!(seq.ends_with("\x07\x1b\\"));

        std::env::remove_var("TMUX");
    }

    #[test]
    fn test_osc52_screen_chunking() {
        std::env::remove_var("TMUX");
        std::env::set_var("TERM", "screen");

        let seq = Osc52Clipboard::format_sequence("short", 100_000).unwrap();
        assert!(seq.starts_with("\x1bP"));
        assert!(seq.ends_with("\x1b\\"));
    }

    #[test]
    fn test_osc52_limit_exceeded_fails() {
        let err = Osc52Clipboard::format_sequence("this is too long", 5).unwrap_err();
        assert!(err.to_string().contains("Selection too long"));
    }
}
