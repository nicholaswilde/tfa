# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_tfa_global_optspecs
    string join \n storage= h/help V/version
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

complete -c tfa -n "__fish_tfa_needs_command" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_needs_command" -s V -l version -d 'Print version'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "generate" -d 'Generate a TOTP token for an account'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "add" -d 'Add a new account secret or OTPAuth URL'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "list" -d 'List all stored accounts'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "delete" -d 'Delete an account secret'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "show" -d 'Show account details and stored secret'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "completions" -d 'Generate shell autocompletions script'
complete -c tfa -n "__fish_tfa_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s c -l copy -d 'Copy generated token to clipboard'
complete -c tfa -n "__fish_tfa_using_subcommand generate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand add" -s d -l digits -d 'Number of digits in generated code (e.g. 6 or 8)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s p -l period -d 'Time step period in seconds (e.g. 30 or 60)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -s a -l algorithm -d 'Hash algorithm (sha1, sha256, sha512)' -r
complete -c tfa -n "__fish_tfa_using_subcommand add" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand list" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand delete" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand show" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand show" -s r -l raw -d 'Print only the raw secret or URI'
complete -c tfa -n "__fish_tfa_using_subcommand show" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand completions" -l storage -d 'Credential storage backend' -r -f -a "auto\t'Attempt OS keyring first; fallback to local file store if unavailable'
keyring\t'Strictly use system keyring (Secret Service, Keychain, Credential Manager)'
file\t'Use local file storage (~/.local/share/tfa/credentials.json)'"
complete -c tfa -n "__fish_tfa_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate add list delete show completions help" -f -a "generate" -d 'Generate a TOTP token for an account'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate add list delete show completions help" -f -a "add" -d 'Add a new account secret or OTPAuth URL'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate add list delete show completions help" -f -a "list" -d 'List all stored accounts'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate add list delete show completions help" -f -a "delete" -d 'Delete an account secret'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate add list delete show completions help" -f -a "show" -d 'Show account details and stored secret'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate add list delete show completions help" -f -a "completions" -d 'Generate shell autocompletions script'
complete -c tfa -n "__fish_tfa_using_subcommand help; and not __fish_seen_subcommand_from generate add list delete show completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
