
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'tfa' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'tfa'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'tfa' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('generate', 'generate', [CompletionResultType]::ParameterValue, 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account')
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Live watch mode displaying continuous TOTP countdown with progress bar')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new account secret or OTPAuth URL')
            [CompletionResult]::new('qr', 'qr', [CompletionResultType]::ParameterValue, 'Render account as a terminal QR code for mobile authenticator scanning')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all stored accounts or live token status')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Live overview status dashboard of all accounts and tokens (alias for list --codes)')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete an account secret')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show account details and stored secret')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export accounts to an encrypted backup file or stdout')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import accounts from a backup file, migration URI, or JSON payload')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Generate UNIX roff man page')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;generate' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('--copy', '--copy', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('--counter', '--counter', [CompletionResultType]::ParameterName, 'Counter value for counter-based (HOTP) token generation')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Watch and continuously refresh code with progress bar')
            [CompletionResult]::new('--watch', '--watch', [CompletionResultType]::ParameterName, 'Watch and continuously refresh code with progress bar')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;watch' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Copy token to clipboard on every cycle')
            [CompletionResult]::new('--copy', '--copy', [CompletionResultType]::ParameterName, 'Copy token to clipboard on every cycle')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;add' {
            [CompletionResult]::new('--qr', '--qr', [CompletionResultType]::ParameterName, 'Decode account secret and parameters from a QR code image')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 5, 6, 8)')
            [CompletionResult]::new('--digits', '--digits', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 5, 6, 8)')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('--period', '--period', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512, steam)')
            [CompletionResult]::new('--algorithm', '--algorithm', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512, steam)')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Initial counter for counter-based (HOTP) accounts')
            [CompletionResult]::new('--counter', '--counter', [CompletionResultType]::ParameterName, 'Initial counter for counter-based (HOTP) accounts')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;qr' {
            [CompletionResult]::new('--light', '--light', [CompletionResultType]::ParameterName, 'Invert terminal QR blocks (for light terminal backgrounds)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;list' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Include live TOTP codes and remaining time-to-live')
            [CompletionResult]::new('--codes', '--codes', [CompletionResultType]::ParameterName, 'Include live TOTP codes and remaining time-to-live')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output overview in JSON format')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;status' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output overview in JSON format')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;delete' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;show' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Print only the raw secret or URI')
            [CompletionResult]::new('--raw', '--raw', [CompletionResultType]::ParameterName, 'Print only the raw secret or URI')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;export' {
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'Master passphrase for encryption (prompted securely if omitted)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--unencrypted', '--unencrypted', [CompletionResultType]::ParameterName, 'Export without encryption (plain JSON)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;import' {
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'Master passphrase for encrypted backup (prompted securely if omitted)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Preview accounts to import without saving them')
            [CompletionResult]::new('--overwrite', '--overwrite', [CompletionResultType]::ParameterName, 'Overwrite existing accounts if conflicts occur')
            [CompletionResult]::new('--skip', '--skip', [CompletionResultType]::ParameterName, 'Skip existing accounts if conflicts occur (default)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;completions' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;man' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;help' {
            [CompletionResult]::new('generate', 'generate', [CompletionResultType]::ParameterValue, 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account')
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Live watch mode displaying continuous TOTP countdown with progress bar')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new account secret or OTPAuth URL')
            [CompletionResult]::new('qr', 'qr', [CompletionResultType]::ParameterValue, 'Render account as a terminal QR code for mobile authenticator scanning')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all stored accounts or live token status')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Live overview status dashboard of all accounts and tokens (alias for list --codes)')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete an account secret')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show account details and stored secret')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export accounts to an encrypted backup file or stdout')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import accounts from a backup file, migration URI, or JSON payload')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Generate UNIX roff man page')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;help;generate' {
            break
        }
        'tfa;help;watch' {
            break
        }
        'tfa;help;add' {
            break
        }
        'tfa;help;qr' {
            break
        }
        'tfa;help;list' {
            break
        }
        'tfa;help;status' {
            break
        }
        'tfa;help;delete' {
            break
        }
        'tfa;help;show' {
            break
        }
        'tfa;help;export' {
            break
        }
        'tfa;help;import' {
            break
        }
        'tfa;help;completions' {
            break
        }
        'tfa;help;man' {
            break
        }
        'tfa;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
