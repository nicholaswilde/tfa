
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
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('generate', 'generate', [CompletionResultType]::ParameterValue, 'Generate a TOTP token for an account')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new account secret or OTPAuth URL')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all stored accounts')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete an account secret')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;generate' {
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('--copy', '--copy', [CompletionResultType]::ParameterName, 'Copy generated token to clipboard')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;add' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 6 or 8)')
            [CompletionResult]::new('--digits', '--digits', [CompletionResultType]::ParameterName, 'Number of digits in generated code (e.g. 6 or 8)')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('--period', '--period', [CompletionResultType]::ParameterName, 'Time step period in seconds (e.g. 30 or 60)')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512)')
            [CompletionResult]::new('--algorithm', '--algorithm', [CompletionResultType]::ParameterName, 'Hash algorithm (sha1, sha256, sha512)')
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;list' {
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;delete' {
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;completions' {
            [CompletionResult]::new('--storage', '--storage', [CompletionResultType]::ParameterName, 'Credential storage backend')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'tfa;help' {
            [CompletionResult]::new('generate', 'generate', [CompletionResultType]::ParameterValue, 'Generate a TOTP token for an account')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new account secret or OTPAuth URL')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all stored accounts')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete an account secret')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell autocompletions script')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tfa;help;generate' {
            break
        }
        'tfa;help;add' {
            break
        }
        'tfa;help;list' {
            break
        }
        'tfa;help;delete' {
            break
        }
        'tfa;help;completions' {
            break
        }
        'tfa;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
