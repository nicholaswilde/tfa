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

#[test]
fn test_integration_account_matching_and_resolution() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let store_path = temp_dir.path().join("credentials.json");
    let store = FileStore::new(store_path);
    let clipboard = NoopClipboard;
    let mut out = Vec::new();

    // Setup accounts
    let cli_add1 = Cli::try_parse_from([
        "tfa",
        "--storage",
        "file",
        "add",
        "GitHub",
        "JBSWY3DPEHPK3PXP",
    ])
    .expect("add GitHub");
    run_app(cli_add1, &store, &clipboard, &mut out).expect("run add");

    let cli_add2 = Cli::try_parse_from([
        "tfa",
        "--storage",
        "file",
        "add",
        "GitLab",
        "JBSWY3DPEHPK3PXP",
    ])
    .expect("add GitLab");
    run_app(cli_add2, &store, &clipboard, &mut out).expect("run add");

    // 1. Exact lookup
    assert_eq!(
        tfa::resolve_account(&store, Some("GitHub"), "Prompt").expect("exact"),
        "GitHub"
    );
    // Exact case-insensitive lookup
    assert_eq!(
        tfa::resolve_account(&store, Some("github"), "Prompt").expect("exact ci"),
        "GitHub"
    );

    // 2. Single-substring lookup
    assert_eq!(
        tfa::resolve_account(&store, Some("hub"), "Prompt").expect("substring"),
        "GitHub"
    );
    assert_eq!(
        tfa::resolve_account(&store, Some("lab"), "Prompt").expect("substring"),
        "GitLab"
    );

    // Single-substring CLI call (generate)
    out.clear();
    let cli_gen = Cli::try_parse_from(["tfa", "generate", "hub", "--quiet"]).expect("parse");
    run_app(cli_gen, &store, &clipboard, &mut out).expect("generate with substring");
    let code = String::from_utf8(out.clone()).unwrap().trim().to_string();
    assert_eq!(code.len(), 6);

    // 3. Ambiguous lookup (non-TTY in tests)
    let err_amb = tfa::resolve_account(&store, Some("git"), "Prompt").unwrap_err();
    let err_str = err_amb.to_string();
    assert!(err_str.contains("Multiple accounts match 'git'"));
    assert!(err_str.contains("GitHub") && err_str.contains("GitLab"));

    // Ambiguous CLI call (show)
    let cli_show_amb = Cli::try_parse_from(["tfa", "show", "git"]).expect("parse");
    let err_show = run_app(cli_show_amb, &store, &clipboard, &mut Vec::new()).unwrap_err();
    assert!(err_show
        .to_string()
        .contains("Multiple accounts match 'git'"));

    // 4. No-match lookup
    let err_none = tfa::resolve_account(&store, Some("nonexistent"), "Prompt").unwrap_err();
    assert!(err_none.to_string().contains("not found"));

    let cli_show_none = Cli::try_parse_from(["tfa", "show", "nonexistent"]).expect("parse");
    assert!(run_app(cli_show_none, &store, &clipboard, &mut Vec::new()).is_err());
}

#[test]
fn test_integration_import_canonical_labeling_dedup_and_auto_suffixing() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let store_path = temp_dir.path().join("credentials.json");
    let store = FileStore::new(store_path);
    let clipboard = NoopClipboard;
    let mut out = Vec::new();

    // 1. Initial account setup: GitHub:alice@example.com with secret SEC1
    let sec1 = "JBSWY3DPEHPK3PXP";
    let cli_add = Cli::try_parse_from([
        "tfa",
        "--storage",
        "file",
        "add",
        "GitHub:alice@example.com",
        sec1,
    ])
    .expect("add initial account");
    run_app(cli_add, &store, &clipboard, &mut out).expect("run add");

    // 2. Import payload containing:
    //    - Identical secret for GitHub:alice@example.com (true duplicate -> safely skipped)
    //    - Distinct secret SEC2 for GitHub:alice@example.com (collision -> auto-suffix to -2)
    //    - Distinct secret SEC3 for GitHub:alice@example.com (collision -> auto-suffix to -3)
    //    - OTPAuth URI with issuer in query: otpauth://totp/bob@example.com?secret=JBSWY3DPEHPK3PXP&issuer=GitLab
    //      (canonical label -> GitLab:bob@example.com)
    let sec2 = "GEZDGNBVGY3TQOJQ";
    let sec3 = "MZXW6YTBOI======";
    let payload = format!(
        "otpauth://totp/GitHub:alice@example.com?secret={sec1}&issuer=GitHub\n\
         otpauth://totp/GitHub:alice@example.com?secret={sec2}&issuer=GitHub\n\
         otpauth://totp/GitHub:alice@example.com?secret={sec3}&issuer=GitHub\n\
         otpauth://totp/bob@example.com?secret={sec1}&issuer=GitLab\n"
    );

    out.clear();
    let cli_import = Cli::try_parse_from(["tfa", "import", &payload]).expect("parse import");
    run_app(cli_import, &store, &clipboard, &mut out).expect("run import");

    let out_str = String::from_utf8(out.clone()).unwrap();
    assert!(
        out_str.contains("Skipping existing account 'GitHub:alice@example.com' (identical secret)")
    );
    assert!(out_str.contains("Imported account 'GitHub:alice@example.com-2' (auto-suffixed collision for 'GitHub:alice@example.com')"));
    assert!(out_str.contains("Imported account 'GitHub:alice@example.com-3' (auto-suffixed collision for 'GitHub:alice@example.com')"));
    assert!(out_str.contains("Imported account 'GitLab:bob@example.com'"));
    assert!(out_str.contains("Summary: 3 imported, 0 updated, 1 skipped."));

    // Verify accounts in store
    let accounts = store.list_accounts().expect("list accounts");
    assert!(accounts.contains(&"GitHub:alice@example.com".to_string()));
    assert!(accounts.contains(&"GitHub:alice@example.com-2".to_string()));
    assert!(accounts.contains(&"GitHub:alice@example.com-3".to_string()));
    assert!(accounts.contains(&"GitLab:bob@example.com".to_string()));

    // 3. Re-import with --skip flag
    let payload_conflict =
        format!("otpauth://totp/GitLab:bob@example.com?secret={sec2}&issuer=GitLab");
    out.clear();
    let cli_skip =
        Cli::try_parse_from(["tfa", "import", &payload_conflict, "--skip"]).expect("parse skip");
    run_app(cli_skip, &store, &clipboard, &mut out).expect("run import skip");
    let skip_str = String::from_utf8(out.clone()).unwrap();
    assert!(skip_str.contains(
        "Skipping existing account 'GitLab:bob@example.com' (use --overwrite to replace)"
    ));
    assert!(skip_str.contains("Summary: 0 imported, 0 updated, 1 skipped."));

    // 4. Re-import with --overwrite flag
    out.clear();
    let cli_overwrite = Cli::try_parse_from(["tfa", "import", &payload_conflict, "--overwrite"])
        .expect("parse overwrite");
    run_app(cli_overwrite, &store, &clipboard, &mut out).expect("run import overwrite");
    let ow_str = String::from_utf8(out.clone()).unwrap();
    assert!(ow_str.contains("Updated account 'GitLab:bob@example.com'"));
    assert!(ow_str.contains("Summary: 0 imported, 1 updated, 0 skipped."));
}
