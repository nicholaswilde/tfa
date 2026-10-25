
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('generate', 'generate', [CompletionResultType]::ParameterValue, 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account')
            [CompletionResult]::new('instant', 'instant', [CompletionResultType]::ParameterValue, 'Generate a 2FA token instantly from raw secret, URI, or stdin without storage')
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
            [CompletionResult]::new('namespace', 'namespace', [CompletionResultType]::ParameterValue, 'Manage and isolate accounts across multi-tenant namespaces')
            [CompletionResult]::new('password', 'password', [CompletionResultType]::ParameterValue, 'Manage master password protection and unlock sessions')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;generate' {
            [CompletionResult]::new('--secret', '--secret', [CompletionResultType]::ParameterName, 'Generate token instantly from secret, URI, or "-" (stdin) without saving to storage')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('--copy', '--copy', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('--counter', '--counter', [CompletionResultType]::ParameterName, 'Counter value for counter-based (HOTP) token generation')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 5, 6, 8)')
            [CompletionResult]::new('--digits', '--digits', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 5, 6, 8)')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('--period', '--period', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512, steam)')
            [CompletionResult]::new('--algorithm', '--algorithm', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512, steam)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Watch and continuously refresh code with progress bar')
            [CompletionResult]::new('--watch', '--watch', [CompletionResultType]::ParameterName, 'Watch and continuously refresh code with progress bar')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;instant' {
            [CompletionResult]::new('--secret', '--secret', [CompletionResultType]::ParameterName, 'Explicit secret key or otpauth:// URI flag')
            [CompletionResult]::new('--qr', '--qr', [CompletionResultType]::ParameterName, 'Decode secret and parameters from a QR code image')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 5, 6, 8)')
            [CompletionResult]::new('--digits', '--digits', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 5, 6, 8)')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('--period', '--period', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512, steam)')
            [CompletionResult]::new('--algorithm', '--algorithm', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512, steam)')
            [CompletionResult]::new('--counter', '--counter', [CompletionResultType]::ParameterName, 'Initial/current counter for counter-based (HOTP) token generation')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('--copy', '--copy', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Watch and continuously refresh code with progress bar')
            [CompletionResult]::new('--watch', '--watch', [CompletionResultType]::ParameterName, 'Watch and continuously refresh code with progress bar')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;watch' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Copy token to clipboard on every cycle (single account only)')
            [CompletionResult]::new('--copy', '--copy', [CompletionResultType]::ParameterName, 'Copy token to clipboard on every cycle (single account only)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Watch all accounts in the active namespace')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Watch all accounts in the active namespace')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('all', 'all', [CompletionResultType]::ParameterValue, 'Watch all accounts in the active namespace')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;watch;all' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;watch;help' {
            [CompletionResult]::new('all', 'all', [CompletionResultType]::ParameterValue, 'Watch all accounts in the active namespace')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;watch;help;all' {
            break
        }
        'tfa;watch;help;help' {
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;list' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;show' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
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
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;man' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;namespace' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all namespaces and indicate the active/default one')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new namespace')
            [CompletionResult]::new('rename', 'rename', [CompletionResultType]::ParameterValue, 'Rename an existing namespace, safely migrating all contained accounts')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a namespace and its contained accounts')
            [CompletionResult]::new('set-default', 'set-default', [CompletionResultType]::ParameterValue, 'Switch and persist default active namespace in configuration file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;namespace;list' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;namespace;create' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;namespace;rename' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;namespace;delete' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Force deletion without interactive confirmation')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Force deletion without interactive confirmation')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;namespace;set-default' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;namespace;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all namespaces and indicate the active/default one')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new namespace')
            [CompletionResult]::new('rename', 'rename', [CompletionResultType]::ParameterValue, 'Rename an existing namespace, safely migrating all contained accounts')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a namespace and its contained accounts')
            [CompletionResult]::new('set-default', 'set-default', [CompletionResultType]::ParameterValue, 'Switch and persist default active namespace in configuration file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;namespace;help;list' {
            break
        }
        'tfa;namespace;help;create' {
            break
        }
        'tfa;namespace;help;rename' {
            break
        }
        'tfa;namespace;help;delete' {
            break
        }
        'tfa;namespace;help;set-default' {
            break
        }
        'tfa;namespace;help;help' {
            break
        }
        'tfa;password' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Set master password and enable password protection')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove master password and disable password protection')
            [CompletionResult]::new('lock', 'lock', [CompletionResultType]::ParameterValue, 'Lock the active session immediately')
            [CompletionResult]::new('unlock', 'unlock', [CompletionResultType]::ParameterValue, 'Unlock session for the configured timeout')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current password protection and session unlock status')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;password;add' {
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Timeout in seconds before unlock expires (default: 300)')
            [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'Timeout in seconds before unlock expires (default: 300)')
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'Master password (prompted securely if omitted)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;password;remove' {
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'Current master password (prompted securely if omitted)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Skip password verification prompt')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Skip password verification prompt')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;password;lock' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;password;unlock' {
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'Master password (prompted securely if omitted)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;password;status' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to configuration file (TOML, YAML, or JSON)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('--clipboard', '--clipboard', [CompletionResultType]::ParameterName, 'Clipboard copy provider (system or osc52)')
            [CompletionResult]::new('--clip-command', '--clip-command', [CompletionResultType]::ParameterName, 'Custom clipboard command to receive token via standard input (e.g. wl-copy, pbcopy, xclip)')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Target namespace for account scoping (e.g. personal, work, home)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;password;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Set master password and enable password protection')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove master password and disable password protection')
            [CompletionResult]::new('lock', 'lock', [CompletionResultType]::ParameterValue, 'Lock the active session immediately')
            [CompletionResult]::new('unlock', 'unlock', [CompletionResultType]::ParameterValue, 'Unlock session for the configured timeout')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current password protection and session unlock status')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;password;help;add' {
            break
        }
        'tfa;password;help;remove' {
            break
        }
        'tfa;password;help;lock' {
            break
        }
        'tfa;password;help;unlock' {
            break
        }
        'tfa;password;help;status' {
            break
        }
        'tfa;password;help;help' {
            break
        }
        'tfa;help' {
            [CompletionResult]::new('generate', 'generate', [CompletionResultType]::ParameterValue, 'Generate a 2FA token (TOTP, Steam, or HOTP) for an account')
            [CompletionResult]::new('instant', 'instant', [CompletionResultType]::ParameterValue, 'Generate a 2FA token instantly from raw secret, URI, or stdin without storage')
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
            [CompletionResult]::new('namespace', 'namespace', [CompletionResultType]::ParameterValue, 'Manage and isolate accounts across multi-tenant namespaces')
            [CompletionResult]::new('password', 'password', [CompletionResultType]::ParameterValue, 'Manage master password protection and unlock sessions')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;help;generate' {
            break
        }
        'tfa;help;instant' {
            break
        }
        'tfa;help;watch' {
            [CompletionResult]::new('all', 'all', [CompletionResultType]::ParameterValue, 'Watch all accounts in the active namespace')
            break
        }
        'tfa;help;watch;all' {
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
        'tfa;help;namespace' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all namespaces and indicate the active/default one')
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create a new namespace')
            [CompletionResult]::new('rename', 'rename', [CompletionResultType]::ParameterValue, 'Rename an existing namespace, safely migrating all contained accounts')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a namespace and its contained accounts')
            [CompletionResult]::new('set-default', 'set-default', [CompletionResultType]::ParameterValue, 'Switch and persist default active namespace in configuration file')
            break
        }
        'tfa;help;namespace;list' {
            break
        }
        'tfa;help;namespace;create' {
            break
        }
        'tfa;help;namespace;rename' {
            break
        }
        'tfa;help;namespace;delete' {
            break
        }
        'tfa;help;namespace;set-default' {
            break
        }
        'tfa;help;password' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Set master password and enable password protection')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove master password and disable password protection')
            [CompletionResult]::new('lock', 'lock', [CompletionResultType]::ParameterValue, 'Lock the active session immediately')
            [CompletionResult]::new('unlock', 'unlock', [CompletionResultType]::ParameterValue, 'Unlock session for the configured timeout')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current password protection and session unlock status')
            break
        }
        'tfa;help;password;add' {
            break
        }
        'tfa;help;password;remove' {
            break
        }
        'tfa;help;password;lock' {
            break
        }
        'tfa;help;password;unlock' {
            break
        }
        'tfa;help;password;status' {
            break
        }
        'tfa;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
