# goto

[![Release](https://img.shields.io/github/v/release/piotr-lebski/goto?display_name=tag&sort=semver)](https://github.com/piotr-lebski/goto/releases)
[![License](https://img.shields.io/github/license/piotr-lebski/goto)](https://github.com/piotr-lebski/goto/blob/main/LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-2ea44f)](https://github.com/piotr-lebski/goto/actions/workflows/ci.yml)

A bookmark-based directory navigation tool. Save named shortcuts to directories
and jump to them instantly from any terminal.

## Install

Build from source (requires [Rust](https://rustup.rs)):

```sh
cargo build --release
cp target/release/goto ~/.local/bin/goto   # any directory on your PATH
```

## Shell Integration

`goto` works by printing the selected directory path to stdout. A thin shell
wrapper captures that and calls `cd`. Without the wrapper, `goto` still manages
bookmarks — the wrapper is what enables directory changes.

Add the appropriate line to your shell config so it is sourced on every new
session, then restart your shell or re-source the config file to apply
immediately (e.g. `source ~/.bashrc`).

### Bash

Add to `~/.bashrc`:

```sh
eval "$(goto --init)"
```

### Zsh

Add to `~/.zshrc`:

```sh
eval "$(goto --init)"
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
goto --init | source
```

### PowerShell

Add to your PowerShell profile (`$PROFILE`):

```powershell
Invoke-Expression (& goto --init)
```

### Auto-detection

`goto --init` auto-detects your shell. If detection fails, pass the shell name
explicitly:

```sh
goto --init bash      # or: zsh, fish, powershell
```

## Usage

```sh
goto                     # Interactive picker — jumps to the selected bookmark
goto --add <name>        # Save the current directory as <name>
goto --replace <name>    # Update an existing bookmark to the current directory
goto --remove <name>     # Delete a bookmark by name
goto --list              # Print all bookmarks as 'name | path'
goto --prune             # Remove bookmarks whose directories no longer exist
goto --prune --yes       # Same, without the confirmation prompt
```

## Interactive Selection

If [`fzf`](https://github.com/junegunn/fzf) is installed, `goto` uses it for
fuzzy-searching bookmarks. Otherwise it uses a built-in keyboard-driven selector.

## Bookmark Storage

Bookmarks are stored as JSON:

| Platform | Path                             |
|----------|----------------------------------|
| All      | `~/.config/goto/bookmarks.json`  |

Set `XDG_CONFIG_HOME` to use a custom XDG base directory, or `GOTO_CONFIG_HOME` to override the config directory entirely.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution guidelines.
