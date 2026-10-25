# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_tfa_global_optspecs
    string join \n C/config= storage= clipboard= h/help V/version
end

function __fish_tfa_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_tfa_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_tfa_using_subcommand
    set -l cmd (__fish_tfa_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c tfa -n "__fish_tfa_needs_command" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_needs_command" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_needs_command" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_needs_command" -s V -l version -d 'Print version'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "generate" -d 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "watch" -d 'Live watch mode displaying continuous TOTP countdown with progress bar'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "add" -d 'Add a new account secret or OTPAuth URL'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "qr" -d 'Render account as a terminal QR code for mobile authenticator scanning'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "list" -d 'List all stored accounts or live token status'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "status" -d 'Live overview status dashboard of all accounts and tokens (alias for list --codes)'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "delete" -d 'Delete an account secret'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "show" -d 'Show account details and stored secret'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "export" -d 'Export accounts to an encrypted backup file or stdout'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "import" -d 'Import accounts from a backup file, migration URI, or JSON payload'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "completions" -d 'Generate shell autocompletions script'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "man" -d 'Generate UNIX roff man page'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s c -l copy -d 'Copy generated token to clipboard' -r -f -a "true\t''
false\t''"
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l counter -d 'Counter value for counter-based (HOTP) token generation' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s w -l watch -d 'Watch and continuously refresh code with progress bar'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand watch" -s c -l copy -d 'Copy token to clipboard on every cycle' -r -f -a "true\t''
false\t''"
complete -c tfa -n "__fish_tfa_using_subcommand watch" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand watch" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand watch" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand watch" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand add" -l qr -d 'Decode account secret and parameters from a QR code image' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand add" -s d -l digits -d 'Number of digits in generated code (e.g. 5, 6, 8)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s p -l period -d 'Time step period in seconds (e.g. 30 or 60)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s a -l algorithm -d 'Hash algorithm (sha1, sha256, sha512, steam)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s c -l counter -d 'Initial counter for counter-based (HOTP) accounts' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand add" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand add" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l light -d 'Invert terminal QR blocks (for light terminal backgrounds)' -r -f -a "true\t''
false\t''"
complete -c tfa -n "__fish_tfa_using_subcommand qr" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand qr" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand list" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand list" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand list" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand list" -s c -l codes -d 'Include live TOTP codes and remaining time-to-live'
complete -c tfa -n "__fish_tfa_using_subcommand list" -l json -d 'Output overview in JSON format'
complete -c tfa -n "__fish_tfa_using_subcommand list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand status" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand status" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand status" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand status" -l json -d 'Output overview in JSON format'
complete -c tfa -n "__fish_tfa_using_subcommand status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand delete" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand show" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand show" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand show" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand show" -s r -l raw -d 'Print only the raw secret or URI'
complete -c tfa -n "__fish_tfa_using_subcommand show" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand export" -l password -d 'Master passphrase for encryption (prompted securely if omitted)' -r
complete -c tfa -n "__fish_tfa_using_subcommand export" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand export" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand export" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand export" -l unencrypted -d 'Export without encryption (plain JSON)'
complete -c tfa -n "__fish_tfa_using_subcommand export" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand import" -l password -d 'Master passphrase for encrypted backup (prompted securely if omitted)' -r
complete -c tfa -n "__fish_tfa_using_subcommand import" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand import" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand import" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand import" -l dry-run -d 'Preview accounts to import without saving them'
complete -c tfa -n "__fish_tfa_using_subcommand import" -l overwrite -d 'Overwrite existing accounts if conflicts occur'
complete -c tfa -n "__fish_tfa_using_subcommand import" -l skip -d 'Skip existing accounts if conflicts occur (default)'
complete -c tfa -n "__fish_tfa_using_subcommand import" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand completions" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand man" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand man" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand man" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand man" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "generate" -d 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "watch" -d 'Live watch mode displaying continuous TOTP countdown with progress bar'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "add" -d 'Add a new account secret or OTPAuth URL'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "qr" -d 'Render account as a terminal QR code for mobile authenticator scanning'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "list" -d 'List all stored accounts or live token status'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "status" -d 'Live overview status dashboard of all accounts and tokens (alias for list --codes)'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "delete" -d 'Delete an account secret'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "show" -d 'Show account details and stored secret'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "export" -d 'Export accounts to an encrypted backup file or stdout'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "import" -d 'Import accounts from a backup file, migration URI, or JSON payload'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "completions" -d 'Generate shell autocompletions script'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "man" -d 'Generate UNIX roff man page'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate watch add qr list status delete show export import completions man help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
