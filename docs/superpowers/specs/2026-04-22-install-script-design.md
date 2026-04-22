# Install Script Design

**Date:** 2026-04-22  
**Repo:** piotr-lebski/goto  
**Status:** Approved

## Problem

Users currently must build `goto` from source (requiring Rust/Cargo). There is no way to install a prebuilt binary with a single command. The project already publishes release archives to GitHub Releases; we need install scripts that make those archives accessible.

## Goals

- Provide a `curl | bash` one-liner for Linux and macOS
- Provide an `iwr | iex` one-liner for Windows (PowerShell)
- Verify download integrity via SHA256 checksums
- Set up shell integration automatically
- Be idempotent (safe to run multiple times)
- Work without any dependencies beyond `curl` / PowerShell builtins

## Non-Goals

- Expanding the release target matrix (remains: Linux x86_64, macOS ARM64, Windows x86_64)
- Package manager distributions (Homebrew, APT, Scoop, etc.)
- Automatic updates

---

## Files

| File | Purpose |
|---|---|
| `install.sh` | Bash installer for Linux x86_64 and macOS Apple Silicon |
| `install.ps1` | PowerShell installer for Windows x86_64 |

---

## install.sh Design

### Flags

| Flag | Default | Description |
|---|---|---|
| `--version <ver>` | latest | Install a specific release tag (e.g. `v0.2.0`) |
| `--bin-dir <path>` | `~/.local/bin` | Directory to install the `goto` binary |
| `--no-shell-integration` | off | Skip appending the shell `eval` line |

### Platform Detection

Use `uname -s` (OS) and `uname -m` (arch) to select the Rust target triple:

| OS | Arch | Target |
|---|---|---|
| `Linux` | `x86_64` | `x86_64-unknown-linux-gnu` |
| `Darwin` | `arm64` | `aarch64-apple-darwin` |
| anything else | — | Unsupported: print error with link to build-from-source instructions and exit 1 |

### Version Resolution

```
GET https://api.github.com/repos/piotr-lebski/goto/releases/latest
```

Parse `tag_name` with `grep`/`sed` — no `jq` required.

### Download & Verification

Asset URL pattern:
```
https://github.com/piotr-lebski/goto/releases/download/<ver>/goto-<target>-<ver>.tar.gz
```

Steps:
1. Create a temp directory (`mktemp -d`); register a `trap` to remove it on exit (including errors)
2. Download archive + `.sha256` sidecar with `curl -fsSL`
3. Verify checksum: the `.sha256` file is produced by `openssl dgst -sha256 -r` (format: `<hash> *<filename>`). On Linux use `sha256sum --check`; on macOS use `shasum -a 256 -c`. Both tools accept the `*` prefix format.
4. Extract binary with `tar -xzf`
5. Move binary to `<bin-dir>/goto` (overwriting any existing installation); create `<bin-dir>` if needed
6. `chmod +x` the binary

### Shell Integration

Detect shell from `$SHELL`:

| Shell | Config file (Linux) | Config file (macOS) | Line appended |
|---|---|---|---|
| bash | `~/.bashrc` | `~/.bash_profile` | `eval "$(goto --init)"` |
| zsh | `~/.zshrc` | `~/.zshrc` | `eval "$(goto --init)"` |
| fish | `~/.config/fish/config.fish` | same | `goto --init \| source` |
| other | — | — | Print a warning with manual instructions |

Before appending, check whether `goto --init` already appears in the config file (idempotency).

Also ensure `<bin-dir>` is on `PATH`: if it is not already exported in the config file, prepend an `export PATH="<bin-dir>:$PATH"` line.

### Output

Use ANSI colour codes to print:
- `✓` green for each successful step
- `!` yellow for warnings (e.g. unsupported shell)
- `✗` red for errors

Always summarise what was changed at the end.

---

## install.ps1 Design

### Flags

| Parameter | Default | Description |
|---|---|---|
| `-Version <ver>` | latest | Specific release tag |
| `-BinDir <path>` | `$env:LOCALAPPDATA\goto\bin` | Install directory |
| `-NoShellIntegration` | off | Skip updating PowerShell profile |

### Platform Detection

Check `$env:PROCESSOR_ARCHITECTURE`. Only `AMD64` is supported (`x86_64-pc-windows-msvc`). Any other value prints a friendly error and exits.

### Version Resolution

```powershell
Invoke-RestMethod https://api.github.com/repos/piotr-lebski/goto/releases/latest
```

Read `.tag_name`.

### Download & Verification

Asset URL pattern:
```
https://github.com/piotr-lebski/goto/releases/download/<ver>/goto-<target>-<ver>.zip
```

Steps:
1. Download archive + `.sha256` to `$env:TEMP\goto-install-<random>\`
2. Verify with `Get-FileHash` — compare against the `.sha256` content
3. Extract with `Expand-Archive`
4. Move `goto.exe` to `<BinDir>`; create dir if needed

### Shell Integration

Read `$PROFILE`. Create the file if it doesn't exist. Check idempotently for `goto --init`; if not present, append:

```powershell
Invoke-Expression (& goto --init)
```

Also ensure `<BinDir>` is on `$env:PATH` in the current session and added persistently via `[System.Environment]::SetEnvironmentVariable`.

---

## README Changes

Update `## Install` to lead with the one-liners:

```markdown
## Install

### Unix (Linux / macOS)

    curl -fsSL https://raw.githubusercontent.com/piotr-lebski/goto/main/install.sh | bash

### Windows (PowerShell)

    iwr -useb https://raw.githubusercontent.com/piotr-lebski/goto/main/install.ps1 | iex

### Build from source

Requires [Rust](https://rustup.rs):

    cargo build --release
    cp target/release/goto ~/.local/bin/goto
```

Options such as `--no-shell-integration` and `--version` are documented inline in the scripts' help text.

---

## Error Handling

- Any `curl` / `Invoke-RestMethod` failure exits immediately with a clear message
- Checksum mismatch exits with an error and removes the download
- Unsupported platform exits cleanly before touching the filesystem
- All temp files are removed on exit (success or failure)

---

## Testing

The scripts are not unit-tested automatically (no CI shell harness is added). Manual smoke-test steps:

1. Install on a clean Linux x86_64 environment
2. Install on macOS Apple Silicon
3. Install on Windows via PowerShell
4. Run with `--no-shell-integration` and verify no config files are modified
5. Run a second time and verify idempotency (no duplicate lines in config)
6. Run with an explicit `--version` tag
