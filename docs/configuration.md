# Configuration

`tfa` supports optional configuration files in **TOML**, **YAML**, or **JSON** format. A fully commented template documenting all supported options and defaults is provided in [`assets/config.example.toml`](https://github.com/nicholaswilde/tfa/blob/main/assets/config.example.toml) (bundled in release packages as `config.example.toml`).

To quickly initialize configuration from the template:

```bash
mkdir -p ~/.config/tfa
cp config.example.toml ~/.config/tfa/config.toml
```

## :1234: Hierarchical Precedence

Settings resolve in strict hierarchical precedence order (highest to lowest):

1. **Command Line Arguments** (e.g. `--storage`, `--age-key`, `--ssh-key`, `--gpg`, `--clipboard`, `--clip-command`, `--clear-after`, `--no-clear`, `--namespace`, `--digits`, `--period`, `--algorithm`, `--copy`, `--light`, `--separator`)
2. **Environment Variables** (e.g. `TFA_STORAGE`, `TFA_AGE_KEY`, `TFA_AGE_KEY_FILE`, `TFA_SSH_KEY`, `TFA_SSH_KEY_FILE`, `TFA_AGE_SSH_KEY`, `TFA_AGE_RECIPIENTS`, `TFA_AGE_PASSPHRASE`, `TFA_GPG_FINGERPRINT`, `TFA_GPG_RECIPIENT`, `TFA_GPG_KEY`, `TFA_GPG_BIN`, `TFA_CLIPBOARD`, `TFA_CLIP_COMMAND`, `TFA_CLIPBOARD_TIMEOUT`, `TFA_CLIPBOARD_CLEAR`, `TFA_NAMESPACE`, `TFA_CONFIG`, `TFA_DEFAULT_COMMAND`, `TFA_DIGITS`, `TFA_PERIOD`, `TFA_STEP`, `TFA_ALGORITHM`, `TFA_AUTO_COPY`, `TFA_LIGHT`, `TFA_RENAME_SEPARATOR`, `TFA_PASSWORD_ENABLED`, `TFA_PASSWORD_TIMEOUT`, `TFA_PASSWORD`, `TFA_SECRET`, `TOTP_TOKEN`, `TFA_AUTH_FILE`, `TFA_SESSION_FILE`)
3. **Configuration File** (`-C <path>` / `--config <path>`, or default locations: `~/.config/tfa/config.{toml,yaml,yml,json}`)
4. **Hardcoded Defaults** (storage: `auto`, clipboard: `system`, clipboard_timeout: disabled, namespace: `default`, default_command: `list`, digits: `6`, period: `30`, algorithm: `sha1`, auto_copy: `false`, light: `false`, rename_separator: `"-"`, gpg_bin: `"gpg"`)

## :floppy_disk: Storage Backends & Headless Environments

`tfa` supports four storage options via `storage` in config, `--storage`, or `TFA_STORAGE`:

- **`auto` (default):** Attempts the OS system keyring first (Secret Service on Linux, Keychain on macOS, Credential Manager on Windows). If the keyring daemon is unreachable, locked, or running in a headless/SSH session (`SS error`), `tfa` displays a notice and automatically falls back to local user file storage (`0600` permissions at `~/.local/share/tfa/credentials.json`).

    ```text
    [notice] Keyring unavailable (Failed saving secret in keyring: Couldn't access platform storage: SS error: result not returned from SS API); saving to local file store
    ```

- **`file`:** Directly reads and writes to `~/.local/share/tfa/credentials.json` without querying DBus or system keyring, silencing fallback notices in headless, SSH, or server environments.

- **`keyring`:** Strictly requires the OS system keyring and returns a hard error if unavailable.

- **`age`:** Identity-based encrypted file storage (`~/.local/share/tfa/credentials.age`) using [`age`](https://github.com/FiloSottile/age). Encrypts credentials with age public keys, SSH keys (`~/.ssh/id_ed25519.pub`, `~/.ssh/id_rsa.pub`), or a passphrase.

### Age Storage Key Resolution

When using `--storage age`, `tfa` resolves decryption identities and encryption recipients in the following order:

1. Explicit age key argument or environment variable: `--age-key`, `TFA_AGE_KEY`, `TFA_AGE_KEY_FILE`, `AGE_KEY_FILE`, or `SOPS_AGE_KEY_FILE`.
2. Explicit SSH key file location: `--ssh-key` (`--ssh-key-file`, `--age-ssh-key`), `TFA_SSH_KEY_FILE`, `TFA_SSH_KEY`, or `ssh_key` in configuration.
3. SOPS age key fallback: `~/.config/sops/age/keys.txt` (or `%APPDATA%\sops\age\keys.txt`).
4. Default age key fallback: `~/.config/age/keys.txt` (or `$XDG_CONFIG_HOME/age/keys.txt`, `%APPDATA%\age\keys.txt`).
5. Default SSH keys: `~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, or any valid key pair in `~/.ssh/`.
6. Passphrase fallback: `TFA_AGE_PASSPHRASE` environment variable, or interactive prompt if no identity files exist.

### Storage Backend Migration

Transfer accounts directly across storage backends (`keyring`, `file`, `age`) in memory without intermediate plaintext exports:

```bash
# Migrate from keyring to age-encrypted storage
tfa migrate --from keyring --to age

# Preview migration without modifying destination or source
tfa migrate --from file --to keyring --dry-run

# Overwrite conflicting accounts in destination backend
tfa migrate --from keyring --to file --overwrite

# Transfer accounts and delete them from the source backend
tfa migrate --from file --to age --purge-source
```

## :gear: Example Configuration Files

A ready-to-use, fully commented template is available at [`assets/config.example.toml`](https://github.com/nicholaswilde/tfa/blob/main/assets/config.example.toml).

### TOML (`~/.config/tfa/config.toml`)

=== "Flat Format"

    ```toml title="~/.config/tfa/config.toml"
    storage = "keyring"           # auto, keyring, file, or age
    age_key = "~/.config/sops/age/keys.txt" # optional age key file or secret key
    ssh_key = "~/.ssh/id_ed25519" # optional SSH private key file
    age_recipients = ["age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"] # optional age/SSH recipient keys for encryption
    gpg_fingerprint = "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3" # optional default GPG recipient key
    gpg_bin = "gpg"               # optional path or binary name for GPG (default: "gpg")
    clipboard = "system"          # system or osc52 (terminal ANSI escape sequence)
    clip_command = "wl-copy"      # optional custom command receiving token via stdin
    clipboard_clear = true        # enable or disable automatic clipboard clearing (true/false)
    clipboard_timeout = 30        # clear copied token after N seconds (0 or false to disable)
    default_namespace = "personal"# active default namespace (default: "default")
    default_command = "list"      # default command when invoked without args (list, status, watch all)
    digits = 6                    # 6 or 8
    period = 30                   # 30 or 60
    algorithm = "sha1"            # sha1, sha256, sha512, or steam
    auto_copy = true              # automatically copy generated tokens
    light = false                 # terminal QR light theme
    rename_separator = "-"        # replacement char for spaces in batch rename (default: "-")
    password = true               # enable master password protection
    password_timeout = 300        # unlock cache TTL in seconds (default: 300)
    ```

=== "Nested Format"

    ```toml title="~/.config/tfa/config.toml"
    storage = "keyring"           # auto, keyring, file, or age
    age_key = "~/.config/sops/age/keys.txt" # optional age key file or secret key
    ssh_key = "~/.ssh/id_ed25519" # optional SSH private key file
    age_recipients = ["age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"] # optional age/SSH recipient keys for encryption
    gpg_fingerprint = "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3" # optional default GPG recipient key
    gpg_bin = "gpg"               # optional path or binary name for GPG (default: "gpg")
    clipboard = "system"          # system or osc52 (terminal ANSI escape sequence)
    clip_command = "wl-copy"      # optional custom command receiving token via stdin
    default_namespace = "personal"# active default namespace (default: "default")
    default_command = "list"      # default command when invoked without args (list, status, watch all)
    digits = 6                    # 6 or 8
    period = 30                   # 30 or 60
    algorithm = "sha1"            # sha1, sha256, sha512, or steam
    auto_copy = true              # automatically copy generated tokens
    light = false                 # terminal QR light theme
    rename_separator = "-"        # replacement char for spaces in batch rename (default: "-")

    [password]
    enabled = true                # enable master password protection
    timeout = 300                 # unlock cache TTL in seconds (default: 300)

    [clipboard_timeout]
    enabled = true                # enable or disable automatic clipboard clearing
    timeout = 30                  # clear copied token after N seconds (default: 30)
    ```

### YAML (`~/.config/tfa/config.yaml`)

=== "Flat Format"

    ```yaml title="~/.config/tfa/config.yaml"
    storage: auto
    age_key: ~/.config/sops/age/keys.txt
    ssh_key: ~/.ssh/id_ed25519
    age_recipients:
      - age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
    gpg_fingerprint: "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3"
    gpg_bin: gpg
    clipboard: system
    clip_command: "xclip -selection clipboard"
    clipboard_clear: true
    clipboard_timeout: 30
    default_namespace: personal
    default_command: list
    digits: 6
    period: 30
    algorithm: sha1
    auto_copy: true
    light: false
    rename_separator: "-"
    password: true
    password_timeout: 300
    ```

=== "Nested Format"

    ```yaml title="~/.config/tfa/config.yaml"
    storage: auto
    age_key: ~/.config/sops/age/keys.txt
    ssh_key: ~/.ssh/id_ed25519
    age_recipients:
      - age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
    gpg_fingerprint: "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3"
    gpg_bin: gpg
    clipboard: system
    clip_command: "xclip -selection clipboard"
    default_namespace: personal
    default_command: list
    digits: 6
    period: 30
    algorithm: sha1
    auto_copy: true
    light: false
    rename_separator: "-"

    password:
      enabled: true
      timeout: 300

    clipboard_timeout:
      enabled: true
      timeout: 30
    ```

### JSON (`~/.config/tfa/config.json`)

=== "Flat Format"

    ```json title="~/.config/tfa/config.json"
    {
      "storage": "auto",
      "age_key": "~/.config/sops/age/keys.txt",
      "ssh_key": "~/.ssh/id_ed25519",
      "age_recipients": [
        "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"
      ],
      "gpg_fingerprint": "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3",
      "gpg_bin": "gpg",
      "clipboard": "system",
      "clip_command": "wl-copy",
      "clipboard_clear": true,
      "clipboard_timeout": 30,
      "default_namespace": "personal",
      "default_command": "list",
      "digits": 6,
      "period": 30,
      "algorithm": "sha1",
      "auto_copy": true,
      "light": false,
      "rename_separator": "-",
      "password": true,
      "password_timeout": 300
    }
    ```

=== "Nested Format"

    ```json title="~/.config/tfa/config.json"
    {
      "storage": "auto",
      "age_key": "~/.config/sops/age/keys.txt",
      "ssh_key": "~/.ssh/id_ed25519",
      "age_recipients": [
        "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"
      ],
      "gpg_fingerprint": "0C0F36434657A3EDA0556C61B1D0FD0AD678DCC3",
      "gpg_bin": "gpg",
      "clipboard": "system",
      "clip_command": "wl-copy",
      "default_namespace": "personal",
      "default_command": "list",
      "digits": 6,
      "period": 30,
      "algorithm": "sha1",
      "auto_copy": true,
      "light": false,
      "rename_separator": "-",
      "password": {
        "enabled": true,
        "timeout": 300
      },
      "clipboard_timeout": {
        "enabled": true,
        "timeout": 30
      }
    }
    ```

## :earth_americas: Environment Variables Reference

| Variable | Config Option | CLI Flag | Description |
| :--- | :--- | :--- | :--- |
| `TFA_STORAGE` | `storage` | `--storage` | Storage backend: `auto`, `keyring`, `file`, or `age` |
| `TFA_CONFIG` | N/A | `-C`, `--config` | Custom path to configuration file (`.toml`, `.yaml`, `.json`) |
| `TFA_DEFAULT_COMMAND` | `default_command` | N/A | Default command when invoked with no args (`list`, `status`, `watch all`) |
| `TFA_NAMESPACE` | `default_namespace` | `-n`, `--namespace` | Target namespace for credential scoping |
| `TFA_DIGITS` | `digits` | `-d`, `--digits` | Number of digits in generated TOTP tokens (`6` or `8`) |
| `TFA_PERIOD` / `TFA_STEP` | `period` | `-p`, `--period` | TOTP time-step period in seconds (`30` or `60`) |
| `TFA_ALGORITHM` | `algorithm` | `-a`, `--algorithm` | HMAC algorithm (`sha1`, `sha256`, `sha512`, `steam`) |
| `TFA_AUTO_COPY` | `auto_copy` | `-c`, `--copy` | Automatically copy generated tokens to clipboard (`true`/`false`) |
| `TFA_CLIPBOARD` | `clipboard` | `--clipboard` | Clipboard provider (`system` or `osc52`) |
| `TFA_CLIP_COMMAND` | `clip_command` | `--clip-command` | Custom external command receiving token via stdin (e.g. `wl-copy`) |
| `TFA_CLIPBOARD_CLEAR` | `clipboard_clear` | `--no-clear` | Enable or disable automatic clipboard clearing (`true`/`false`) |
| `TFA_CLIPBOARD_TIMEOUT` | `clipboard_timeout` | `--clear-after` | Seconds before clearing copied token (`0` or `false` to disable) |
| `TFA_LIGHT` | `light` | `--light` | Invert QR code colors for light terminal backgrounds |
| `TFA_RENAME_SEPARATOR` | `rename_separator` | `-s`, `--separator` | Space replacement character for `rename --batch` (default: `-`) |
| `TFA_AGE_KEY` / `TFA_AGE_KEY_FILE` | `age_key` | `--age-key` | Raw age identity string or path to age identity key file |
| `TFA_SSH_KEY` / `TFA_SSH_KEY_FILE` / `TFA_AGE_SSH_KEY` | `ssh_key` | `--ssh-key` | Path to SSH private key used as age identity |
| `TFA_AGE_RECIPIENTS` | `age_recipients` | N/A | Comma- or newline-separated recipient public keys for age encryption |
| `TFA_AGE_PASSPHRASE` | N/A (Secret) | N/A | Passphrase for non-interactive age storage decryption |
| `TFA_GPG_FINGERPRINT` / `TFA_GPG_RECIPIENT` / `TFA_GPG_KEY` | `gpg_fingerprint` | `--gpg` | Default recipient GPG key fingerprint for backups |
| `TFA_GPG_BIN` | `gpg_bin` | N/A | Path or binary name for GPG executable (default: `gpg`) |
| `TFA_PASSWORD_ENABLED` | `password` | N/A | Enable master password protection (`true`/`false`) |
| `TFA_PASSWORD_TIMEOUT` | `password_timeout` | N/A | Session TTL in seconds before reprompting for password (default: `300`) |
| `TFA_PASSWORD` | N/A (Secret) | `--password` | Master password string for non-interactive scripting |
| `TFA_SECRET` / `TOTP_TOKEN` | N/A (Secret) | `--secret` | Fallback raw secret or otpauth URI for `instant` command |
| `TFA_AUTH_FILE` | N/A (System) | N/A | Custom path for master password hash record file |
| `TFA_SESSION_FILE` | N/A (System) | N/A | Custom path for runtime session unlock token file |
