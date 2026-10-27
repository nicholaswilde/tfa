# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_tfa_global_optspecs
    string join \n C/config= storage= age-key= ssh-key= clipboard= clip-command= n/namespace= h/help V/version
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
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_needs_command" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_needs_command" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_needs_command" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_needs_command" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_needs_command" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_needs_command" -s V -l version -d 'Print version'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "generate" -d 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "instant" -d 'Generate a 2FA token instantly from raw secret, URI, or stdin without storage'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "watch" -d 'Live watch mode displaying continuous TOTP countdown with progress bar'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "add" -d 'Add a new account secret or OTPAuth URL'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "qr" -d 'Render account as a terminal QR code for mobile authenticator scanning'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "list" -d 'List all stored accounts or live token status'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "status" -d 'Live overview status dashboard of all accounts and tokens (alias for list --codes)'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "delete" -d 'Delete an account secret'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "rename" -d 'Rename an account within the active namespace'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "move" -d 'Move an account to a different namespace'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "show" -d 'Show account details and stored secret'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "export" -d 'Export accounts to an encrypted backup file or stdout'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "import" -d 'Import accounts from a backup file, migration URI, or JSON payload'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "completions" -d 'Generate shell autocompletions script'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "man" -d 'Generate UNIX roff man page'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "namespace" -d 'Manage and isolate accounts across multi-tenant namespaces'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "password" -d 'Manage master password protection and unlock sessions'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "audit" -d 'Scan stored accounts and credentials file for security vulnerabilities'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l secret -d 'Generate token instantly from secret, URI, or "-" (stdin) without saving to storage' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s c -l copy -d 'Copy generated token to clipboard' -r -f -a "true\t''
false\t''"
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l counter -d 'Counter value for counter-based (HOTP) token generation' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s d -l digits -d 'Number of digits in generated code (e.g. 5, 6, 8)' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s p -l period -d 'Time step period in seconds (e.g. 30 or 60)' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s a -l algorithm -d 'Hash algorithm (sha1, sha256, sha512, steam)' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s w -l watch -d 'Watch and continuously refresh code with progress bar'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l next -d 'Preview upcoming token for next time step'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l json -d 'Output generated token details as JSON'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l secret -d 'Explicit secret key or otpauth:// URI flag' -r
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l qr -d 'Decode secret and parameters from a QR code image' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s d -l digits -d 'Number of digits in generated code (e.g. 5, 6, 8)' -r
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s p -l period -d 'Time step period in seconds (e.g. 30 or 60)' -r
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s a -l algorithm -d 'Hash algorithm (sha1, sha256, sha512, steam)' -r
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l counter -d 'Initial/current counter for counter-based (HOTP) token generation' -r
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s c -l copy -d 'Copy generated token to clipboard' -r -f -a "true\t''
false\t''"
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand instant" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s w -l watch -d 'Watch and continuously refresh code with progress bar'
complete -c tfa -n "__fish_tfa_using_subcommand instant" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -s f -l filter -d 'Filter accounts by substring or glob pattern (used with watch all)' -r
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -s c -l copy -d 'Copy token to clipboard on every cycle (single account only)' -r -f -a "true\t''
false\t''"
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -s a -l all -d 'Watch all accounts in the active namespace'
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -a "all" -d 'Watch all accounts in the active namespace'
complete -c tfa -n "__fish_tfa_using_subcommand watch; and not __fish_seen_subcommand_from all help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -s f -l filter -d 'Filter accounts by substring or glob pattern' -r
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from all" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from help" -f -a "all" -d 'Watch all accounts in the active namespace'
complete -c tfa -n "__fish_tfa_using_subcommand watch; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand add" -l qr -d 'Decode account secret and parameters from a QR code image' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand add" -s d -l digits -d 'Number of digits in generated code (e.g. 5, 6, 8)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s p -l period -d 'Time step period in seconds (e.g. 30 or 60)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s a -l algorithm -d 'Hash algorithm (sha1, sha256, sha512, steam)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s c -l counter -d 'Initial counter for counter-based (HOTP) accounts' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand add" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand add" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand add" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand add" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand add" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l light -d 'Invert terminal QR blocks (for light terminal backgrounds)' -r -f -a "true\t''
false\t''"
complete -c tfa -n "__fish_tfa_using_subcommand qr" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand qr" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand qr" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand qr" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand list" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand list" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand list" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand list" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand list" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand list" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand list" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand list" -s c -l codes -d 'Include live TOTP codes and remaining time-to-live'
complete -c tfa -n "__fish_tfa_using_subcommand list" -l json -d 'Output overview in JSON format'
complete -c tfa -n "__fish_tfa_using_subcommand list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand status" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand status" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand status" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand status" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand status" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand status" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand status" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand status" -l json -d 'Output overview in JSON format'
complete -c tfa -n "__fish_tfa_using_subcommand status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand delete" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand delete" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand rename" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand rename" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand rename" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand rename" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand rename" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand rename" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand rename" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand rename" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand move" -s t -l to -d 'Destination namespace' -r
complete -c tfa -n "__fish_tfa_using_subcommand move" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand move" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand move" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand move" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand move" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand move" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand move" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand move" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand show" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand show" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand show" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand show" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand show" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand show" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand show" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand show" -s r -l raw -d 'Print only the raw secret or URI'
complete -c tfa -n "__fish_tfa_using_subcommand show" -l json -d 'Output account details and metadata as JSON'
complete -c tfa -n "__fish_tfa_using_subcommand show" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand export" -l password -d 'Master passphrase for encryption (prompted securely if omitted)' -r
complete -c tfa -n "__fish_tfa_using_subcommand export" -l gpg -l gpg-recipient -l gpg-key -d 'Encrypt backup using GPG with optional recipient key fingerprint, ID, or email' -r
complete -c tfa -n "__fish_tfa_using_subcommand export" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand export" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand export" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand export" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand export" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand export" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand export" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand export" -l unencrypted -d 'Export without encryption (plain JSON)'
complete -c tfa -n "__fish_tfa_using_subcommand export" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand import" -l password -d 'Master passphrase for encrypted backup (prompted securely if omitted)' -r
complete -c tfa -n "__fish_tfa_using_subcommand import" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand import" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand import" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand import" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand import" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand import" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand import" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand import" -l dry-run -d 'Preview accounts to import without saving them'
complete -c tfa -n "__fish_tfa_using_subcommand import" -l overwrite -d 'Overwrite existing accounts if conflicts occur'
complete -c tfa -n "__fish_tfa_using_subcommand import" -l skip -d 'Skip existing accounts if conflicts occur (default)'
complete -c tfa -n "__fish_tfa_using_subcommand import" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand completions" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand completions" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand man" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand man" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand man" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand man" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand man" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand man" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand man" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand man" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -f -a "list" -d 'List all namespaces and indicate the active/default one'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -f -a "create" -d 'Create a new namespace'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -f -a "rename" -d 'Rename an existing namespace, safely migrating all contained accounts'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -f -a "delete" -d 'Delete a namespace and its contained accounts'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -f -a "set-default" -d 'Switch and persist default active namespace in configuration file'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and not __fish_seen_subcommand_from list create rename delete set-default help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -s f -l force -d 'Force deletion without interactive confirmation'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from set-default" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all namespaces and indicate the active/default one'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new namespace'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an existing namespace, safely migrating all contained accounts'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "delete" -d 'Delete a namespace and its contained accounts'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "set-default" -d 'Switch and persist default active namespace in configuration file'
complete -c tfa -n "__fish_tfa_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -f -a "add" -d 'Set master password and enable password protection'
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -f -a "remove" -d 'Remove master password and disable password protection'
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -f -a "lock" -d 'Lock the active session immediately'
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -f -a "unlock" -d 'Unlock session for the configured timeout'
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -f -a "status" -d 'Show current password protection and session unlock status'
complete -c tfa -n "__fish_tfa_using_subcommand password; and not __fish_seen_subcommand_from add remove lock unlock status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -s t -l timeout -d 'Timeout in seconds before unlock expires (default: 300)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -l password -d 'Master password (prompted securely if omitted)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -l password -d 'Current master password (prompted securely if omitted)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -s f -l force -d 'Skip password verification prompt'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from lock" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -l password -d 'Master password (prompted securely if omitted)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from unlock" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from help" -f -a "add" -d 'Set master password and enable password protection'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove master password and disable password protection'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from help" -f -a "lock" -d 'Lock the active session immediately'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from help" -f -a "unlock" -d 'Unlock session for the configured timeout'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show current password protection and session unlock status'
complete -c tfa -n "__fish_tfa_using_subcommand password; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand audit" -s C -l config -d 'Path to configuration file (TOML, YAML, or JSON)' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand audit" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'
age\t'Use age-encrypted file storage (~/.local/share/tfa/credentials.age)'"
complete -c tfa -n "__fish_tfa_using_subcommand audit" -l age-key -d 'Path to age key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand audit" -l ssh-key -l age-ssh-key -l ssh-key-file -d 'Path to SSH key file for age-encrypted storage' -r -F
complete -c tfa -n "__fish_tfa_using_subcommand audit" -l clipboard -d 'Clipboard copy provider (system or osc52)' -r -f -a "system\t'Native system clipboard (arboard with Wayland/X11/macOS/Windows)'
osc52\t'Terminal ANSI escape sequence OSC 52 (works over remote SSH, tmux, screen)'"
complete -c tfa -n "__fish_tfa_using_subcommand audit" -l clip-command -d 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)' -r
complete -c tfa -n "__fish_tfa_using_subcommand audit" -s n -l namespace -d 'Target namespace for account scoping (e.g. personal, work, home)' -r
complete -c tfa -n "__fish_tfa_using_subcommand audit" -l json -d 'Output audit report in JSON format'
complete -c tfa -n "__fish_tfa_using_subcommand audit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "generate" -d 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "instant" -d 'Generate a 2FA token instantly from raw secret, URI, or stdin without storage'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "watch" -d 'Live watch mode displaying continuous TOTP countdown with progress bar'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "add" -d 'Add a new account secret or OTPAuth URL'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "qr" -d 'Render account as a terminal QR code for mobile authenticator scanning'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "list" -d 'List all stored accounts or live token status'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "status" -d 'Live overview status dashboard of all accounts and tokens (alias for list --codes)'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "delete" -d 'Delete an account secret'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "rename" -d 'Rename an account within the active namespace'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "move" -d 'Move an account to a different namespace'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "show" -d 'Show account details and stored secret'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "export" -d 'Export accounts to an encrypted backup file or stdout'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "import" -d 'Import accounts from a backup file, migration URI, or JSON payload'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "completions" -d 'Generate shell autocompletions script'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "man" -d 'Generate UNIX roff man page'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "namespace" -d 'Manage and isolate accounts across multi-tenant namespaces'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "password" -d 'Manage master password protection and unlock sessions'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "audit" -d 'Scan stored accounts and credentials file for security vulnerabilities'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate instant watch add qr list status delete rename move show export import completions man namespace password audit help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from watch" -f -a "all" -d 'Watch all accounts in the active namespace'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "list" -d 'List all namespaces and indicate the active/default one'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "create" -d 'Create a new namespace'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "rename" -d 'Rename an existing namespace, safely migrating all contained accounts'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "delete" -d 'Delete a namespace and its contained accounts'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "set-default" -d 'Switch and persist default active namespace in configuration file'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from password" -f -a "add" -d 'Set master password and enable password protection'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from password" -f -a "remove" -d 'Remove master password and disable password protection'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from password" -f -a "lock" -d 'Lock the active session immediately'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from password" -f -a "unlock" -d 'Unlock session for the configured timeout'
complete -c tfa -n "__fish_tfa_using_subcommand help; and __fish_seen_subcommand_from password" -f -a "status" -d 'Show current password protection and session unlock status'
