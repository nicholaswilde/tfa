# Installation

`tfa` is distributed as a prebuilt Rust binary via Homebrew, APT, Scoop, and GitHub Releases, or can be compiled from source with Cargo.

## :beer: Homebrew (macOS / Linux)

Install `tfa` via [Homebrew](https://brew.sh/) using the [`nicholaswilde/homebrew-tap`](https://github.com/nicholaswilde/homebrew-tap):

```bash
brew install nicholaswilde/homebrew-tap/tfa
```

Or tap the repository first:

```bash
brew tap nicholaswilde/homebrew-tap
brew install tfa
```

## :package: APT (Debian / Ubuntu)

Install `.deb` packages (`amd64`, `arm64`, `armhf`, `armel`) from [`nicholaswilde/apt`](https://nicholaswilde.io/apt) (includes binary, man page, and shell completions):

```bash
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://nicholaswilde.io/apt/public.key | sudo gpg --dearmor -o /etc/apt/keyrings/nicholaswilde-apt.gpg
sudo chmod a+r /etc/apt/keyrings/nicholaswilde-apt.gpg

sudo tee /etc/apt/sources.list.d/nicholaswilde.sources << 'EOF'
Types: deb
URIs: https://nicholaswilde.io/apt
Suites: stable
Components: main
Signed-By: /etc/apt/keyrings/nicholaswilde-apt.gpg
EOF

sudo apt update
sudo apt install tfa
```

## :icecream: Scoop (Windows)

Install `tfa` via [Scoop](https://scoop.sh/) using [`nicholaswilde/scoop-bucket`](https://github.com/nicholaswilde/scoop-bucket):

```powershell
scoop bucket add nicholaswilde https://github.com/nicholaswilde/scoop-bucket
scoop install nicholaswilde/tfa
```

## :inbox_tray: GitHub Releases & Self-Update

Download prebuilt archives (`.tar.gz`, `.zip`) and `.deb` packages for Linux, macOS, and Windows from the [GitHub Releases](https://github.com/nicholaswilde/tfa/releases) page.

Update `tfa` in-place directly from `nicholaswilde/tfa` GitHub releases (useful when installed via GitHub release archives rather than a package manager):

```bash
# Check if a newer release is available without installing:
tfa self-update --check

# Update interactively to the latest release:
tfa self-update

# Update without confirmation prompt:
tfa self-update --yes

# Install a specific release version or tag:
tfa self-update --version 0.1.18 --yes
```

## :crab: Build or Install from Source

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.88+)
- [Task](https://taskfile.dev/) (optional, recommended)

### Install or Build Locally

```bash title="Install with Cargo"
cargo install --git https://github.com/nicholaswilde/tfa
```

```bash title="Build locally"
git clone https://github.com/nicholaswilde/tfa.git
cd tfa
task build:local
# Or directly with Cargo:
cargo build --release
```

The compiled binary will be placed at `target/release/tfa`.

## :books: Shell Completions

`tfa` features dynamic autocompletion for stored account names, configured namespaces, commands, and options. Shell scripts can be generated for Bash, Zsh, Fish, PowerShell, and Elvish:

=== "Bash"

    ```bash
    # Add to ~/.bashrc for interactive sessions:
    eval "$(tfa completions bash)"

    # Or install to standard completion directory:
    tfa completions bash > ~/.local/share/bash-completion/completions/tfa
    ```

=== "Zsh"

    ```bash
    # Add to ~/.zshrc (after compinit):
    source <(tfa completions zsh)

    # Or save to your fpath directory:
    tfa completions zsh > ~/.zfunc/_tfa
    # (Ensure fpath includes ~/.zfunc and autoload -Uz compinit && compinit is called)
    ```

=== "Fish"

    ```fish
    # Save to standard fish completion directory:
    tfa completions fish > ~/.config/fish/completions/tfa.fish

    # Or load directly in current session:
    tfa completions fish | source
    ```

=== "PowerShell"

    ```powershell
    # Append to your PowerShell profile:
    tfa completions powershell >> $PROFILE

    # Or load into active session:
    tfa completions powershell | Out-String | Invoke-Expression
    ```

=== "Elvish"

    ```elvish
    # Append to your Elvish rc:
    tfa completions elvish >> ~/.elvish/rc.elv

    # Or load into active session:
    eval (tfa completions elvish | slurp)
    ```

!!! tip
    Dynamic completions automatically query stored account names (for `generate`, `watch`, `show`, `qr`, `edit`, `delete`, `rename`, `move`) and namespace names (for `-n` / `--namespace`, `--to`, and `tfa namespace` subcommands) on `<TAB>`. Queries fail gracefully (returning empty candidates) if storage is locked or unreachable. Pass `--static` to `tfa completions` to generate static shell completion scripts instead.

Pre-generated completion scripts are also available in the [`completions/`](https://github.com/nicholaswilde/tfa/tree/main/completions) folder.

## :page_facing_up: Man Page

Generate or view the UNIX man page:

```bash
# Generate man page
task man
# Or with binary
tfa man > assets/tfa.1

# Preview formatted man page
man -l assets/tfa.1
```
