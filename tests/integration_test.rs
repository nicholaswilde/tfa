use tfa::cli::{Cli, StorageBackend};
use tfa::clipboard::Clipboard;
use tfa::run_app;
use tfa::storage::{AccountStore, FileStore};

struct NoopClipboard;

impl Clipboard for NoopClipboard {
    fn set_text(&self, _text: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn clear_if_matches(&self, _expected: &str) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[test]
fn test_library_integration_basic_lifecycle() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let store_path = temp_dir.path().join("credentials.json");
    let store = FileStore::new(store_path);
    let clipboard = NoopClipboard;
    let mut out = Vec::new();

    // 1. Add account via CLI dispatch through tfa library API
    let cli = Cli::try_parse_from([
        "tfa",
        "--storage",
        "file",
        "add",
        "TestIntegrationAcct",
        "JBSWY3DPEHPK3PXP",
    ])
    .expect("Parse add command");
    run_app(cli, &store, &clipboard, &mut out).expect("run add");

    assert!(store.get_secret("TestIntegrationAcct").is_ok());

    // 2. Generate token
    out.clear();
    let cli = Cli::try_parse_from(["tfa", "generate", "TestIntegrationAcct", "--quiet"])
        .expect("Parse generate command");
    run_app(cli, &store, &clipboard, &mut out).expect("run generate");

    let token = String::from_utf8(out.clone())
        .expect("utf8 token")
        .trim()
        .to_string();
    assert_eq!(token.len(), 6);
    assert!(token.chars().all(|c| c.is_ascii_digit()));

    // 3. Show account details
    out.clear();
    let cli = Cli::try_parse_from(["tfa", "show", "TestIntegrationAcct", "--raw"])
        .expect("Parse show command");
    run_app(cli, &store, &clipboard, &mut out).expect("run show");

    let show_output = String::from_utf8(out.clone()).expect("utf8 output");
    assert!(show_output.contains("secret=JBSWY3DPEHPK3PXP"));

    // 4. Delete account
    out.clear();
    let cli = Cli::try_parse_from(["tfa", "delete", "TestIntegrationAcct"])
        .expect("Parse delete command");
    run_app(cli, &store, &clipboard, &mut out).expect("run delete");

    assert!(store.get_secret("TestIntegrationAcct").is_err());
}

#[test]
fn test_library_integration_storage_backend_enum() {
    assert_eq!(StorageBackend::default(), StorageBackend::Auto);
}
