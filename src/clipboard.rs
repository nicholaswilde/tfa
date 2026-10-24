use anyhow::Result;

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

/// In-memory clipboard for headless testing
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
}
