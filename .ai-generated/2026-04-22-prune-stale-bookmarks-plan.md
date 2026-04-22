# Prune Stale Bookmarks + Grey-out in Picker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `goto --prune [--yes]` to remove bookmarks whose directories no longer exist, and dim stale entries in the interactive picker.

**Architecture:** Staleness logic lives in `bookmarks.rs` (`Bookmark::is_valid()`, `BookmarkCollection::stale()`, `BookmarkCollection::prune()`). The prune command is handled in `app.rs`. `selector.rs` gains a `strip_ansi()` helper and dims stale entries in `format_items`.

**Tech Stack:** Rust, clap 4.5 (derive), dialoguer 0.11, tempfile 3.10 (dev)

---

## File Map

| File | Change |
|------|--------|
| `src/bookmarks.rs` | Add `Bookmark::is_valid()`, `BookmarkCollection::stale()`, `BookmarkCollection::prune()` |
| `src/cli.rs` | Add `--prune` (bool) and `--yes` (bool) flags |
| `src/app.rs` | Add prune command branch |
| `src/selector.rs` | Add `strip_ansi()`, dim stale entries in `format_items`, strip ANSI before path extraction |
| `tests/cli_smoke.rs` | Add 4 integration tests for prune |
| `README.md` | Add `--prune` and `--prune --yes` to usage block |

---

### Task 1: `Bookmark::is_valid()`

**Files:**
- Modify: `src/bookmarks.rs`

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)]` block at the bottom of `src/bookmarks.rs`:

```rust
#[test]
fn is_valid_returns_true_for_existing_directory() {
    let dir = tempfile::tempdir().unwrap();
    let b = Bookmark::new("test", dir.path().to_str().unwrap());
    assert!(b.is_valid());
}

#[test]
fn is_valid_returns_false_for_nonexistent_path() {
    let b = Bookmark::new("test", "/nonexistent/path/xyzzy123");
    assert!(!b.is_valid());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test is_valid --locked 2>&1 | tail -20
```

Expected: compile error — `is_valid` not found.

- [ ] **Step 3: Add `is_valid()` to `Bookmark`**

Add the method to the `impl Bookmark` block in `src/bookmarks.rs` (after the `new` method):

```rust
pub fn is_valid(&self) -> bool {
    std::path::Path::new(&self.path).is_dir()
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test is_valid --locked 2>&1 | tail -10
```

Expected: `test bookmarks::tests::is_valid_returns_false_for_nonexistent_path ... ok`  
Expected: `test bookmarks::tests::is_valid_returns_true_for_existing_directory ... ok`

- [ ] **Step 5: Commit**

```bash
git add src/bookmarks.rs
git commit -m "feat: add Bookmark::is_valid()"
```

---

### Task 2: `BookmarkCollection::stale()` and `prune()`

**Files:**
- Modify: `src/bookmarks.rs`

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)]` block in `src/bookmarks.rs`:

```rust
#[test]
fn stale_returns_only_invalid_bookmarks() {
    let dir = tempfile::tempdir().unwrap();
    let c = col(&[
        ("valid", dir.path().to_str().unwrap()),
        ("stale", "/nonexistent/path/xyzzy123"),
    ]);
    let stale_names: Vec<_> = c.stale().map(|b| b.name.as_str()).collect();
    assert_eq!(stale_names, vec!["stale"]);
}

#[test]
fn prune_removes_stale_entries_and_returns_count() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = col(&[
        ("valid", dir.path().to_str().unwrap()),
        ("stale", "/nonexistent/path/xyzzy123"),
    ]);
    let removed = c.prune();
    assert_eq!(removed, 1);
    let names: Vec<_> = c.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names, vec!["valid"]);
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test "stale\|prune" --locked 2>&1 | tail -20
```

Expected: compile error — `stale` and `prune` not found.

- [ ] **Step 3: Add `stale()` and `prune()` to `BookmarkCollection`**

Add both methods to the `impl BookmarkCollection` block in `src/bookmarks.rs` (after `remove`):

```rust
pub fn stale(&self) -> impl Iterator<Item = &Bookmark> {
    self.bookmarks.iter().filter(|b| !b.is_valid())
}

pub fn prune(&mut self) -> usize {
    let before = self.bookmarks.len();
    self.bookmarks.retain(|b| b.is_valid());
    before - self.bookmarks.len()
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --locked 2>&1 | tail -15
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/bookmarks.rs
git commit -m "feat: add BookmarkCollection::stale() and prune()"
```

---

### Task 3: CLI flags `--prune` and `--yes`

**Files:**
- Modify: `src/cli.rs`

- [ ] **Step 1: Add the two flags to `Cli`**

In `src/cli.rs`, add both flags after the `remove` field:

```rust
/// Remove bookmarks whose directories no longer exist
#[arg(long, conflicts_with_all = ["list", "add", "replace", "remove", "init"])]
pub prune: bool,

/// Skip the confirmation prompt (use with --prune)
#[arg(long, conflicts_with_all = ["list", "add", "replace", "remove", "init"])]
pub yes: bool,
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build --locked 2>&1 | tail -10
```

Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add src/cli.rs
git commit -m "feat: add --prune and --yes CLI flags"
```

---

### Task 4: Prune command logic in `app.rs`

**Files:**
- Modify: `src/app.rs`
- Modify: `tests/cli_smoke.rs`

- [ ] **Step 1: Write the failing integration tests**

Add the following 4 tests to `tests/cli_smoke.rs`. Also add `use std::process::Stdio;` at the top of the file if not already present.

```rust
#[test]
fn goto_prune_on_empty_store_prints_nothing_to_prune() {
    let temp = tempfile::tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--prune")
        .env("GOTO_CONFIG_HOME", temp.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Nothing to prune"),
        "expected nothing-to-prune message: {stderr}"
    );
}

#[test]
fn goto_prune_with_all_valid_bookmarks_prints_nothing_to_prune() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let target_dir = tempfile::tempdir().unwrap();
    let bookmarks_dir = config_dir.join("goto");
    std::fs::create_dir_all(&bookmarks_dir).unwrap();
    std::fs::write(
        bookmarks_dir.join("bookmarks.json"),
        format!(
            r#"{{"bookmarks":[{{"name":"valid","path":"{}"}}]}}"#,
            target_dir.path().to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--prune")
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Nothing to prune"),
        "expected nothing-to-prune message: {stderr}"
    );
}

#[test]
fn goto_prune_yes_removes_stale_bookmarks() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let valid_dir = tempfile::tempdir().unwrap();
    let bookmarks_dir = config_dir.join("goto");
    std::fs::create_dir_all(&bookmarks_dir).unwrap();
    std::fs::write(
        bookmarks_dir.join("bookmarks.json"),
        format!(
            r#"{{"bookmarks":[{{"name":"valid","path":"{}"}},{{"name":"stale","path":"/nonexistent/path/xyzzy123"}}]}}"#,
            valid_dir.path().to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--prune", "--yes"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("stale") && stderr.contains("xyzzy123"),
        "expected stale bookmark listed before removal: {stderr}"
    );
    assert!(
        stderr.contains("Pruned 1 bookmark(s)"),
        "expected pruned message: {stderr}"
    );

    let list_output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--list")
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        !list_stdout.contains("stale"),
        "stale bookmark should be removed: {list_stdout}"
    );
    assert!(
        list_stdout.contains("valid"),
        "valid bookmark should remain: {list_stdout}"
    );
}

#[test]
fn goto_prune_yes_on_all_stale_leaves_empty_store() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let bookmarks_dir = config_dir.join("goto");
    std::fs::create_dir_all(&bookmarks_dir).unwrap();
    std::fs::write(
        bookmarks_dir.join("bookmarks.json"),
        r#"{"bookmarks":[{"name":"stale","path":"/nonexistent/path/xyzzy123"}]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--prune", "--yes"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Pruned 1 bookmark(s)"),
        "expected pruned message: {stderr}"
    );

    let list_output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--list")
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&list_output.stdout), "");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test goto_prune --locked 2>&1 | tail -20
```

Expected: tests compile but fail — `--prune` flag exists but does nothing yet, so the "Nothing to prune" message won't appear.

- [ ] **Step 3: Implement the prune branch in `app.rs`**

In `src/app.rs`, add the prune branch after the `--remove` branch (before the `// No args — interactive selection` comment). Also add `use dialoguer::Confirm;` at the top of the file.

```rust
use dialoguer::Confirm;
```

And the prune branch:

```rust
if cli.prune {
    let mut collection = store.load()?;
    let stale: Vec<crate::bookmarks::Bookmark> = collection.stale().cloned().collect();

    if stale.is_empty() {
        eprintln!("Nothing to prune, all bookmarks are valid.");
        return Ok(());
    }

    eprintln!("The following bookmarks will be removed:");
    for b in &stale {
        eprintln!("  \u{2717} {} | {}", b.name, b.path);
    }

    if !cli.yes {
        let confirmed = Confirm::new()
            .with_prompt(format!("Remove {} bookmark(s)?", stale.len()))
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?;
        if !confirmed {
            return Ok(());
        }
    }

    let count = collection.prune();
    store.save(&collection)?;
    eprintln!("Pruned {count} bookmark(s).");
    return Ok(());
}
```

The full updated `src/app.rs` with the new import and branch:

```rust
use crate::bookmarks::Bookmark;
use crate::cli::Cli;
use crate::init;
use crate::selector;
use crate::store::Store;
use dialoguer::Confirm;

pub fn run(cli: Cli) -> Result<(), String> {
    if let Some(shell_name) = cli.init {
        let shell = if shell_name == "auto" {
            init::detect_shell()?
        } else {
            init::parse_shell(&shell_name)?
        };
        print!("{}", init::snippet(shell));
        return Ok(());
    }

    let store = Store::new();

    if cli.list {
        let collection = store.load()?;
        for bookmark in collection.iter() {
            println!("{} | {}", bookmark.name, bookmark.path);
        }
        return Ok(());
    }

    if let Some(name) = cli.add {
        let path = cwd()?;
        let mut collection = store.load()?;
        collection.add(name, path)?;
        store.save(&collection)?;
        return Ok(());
    }

    if let Some(name) = cli.replace {
        let path = cwd()?;
        let mut collection = store.load()?;
        collection.replace(&name, path)?;
        store.save(&collection)?;
        return Ok(());
    }

    if let Some(name) = cli.remove {
        let mut collection = store.load()?;
        collection.remove(&name)?;
        store.save(&collection)?;
        return Ok(());
    }

    if cli.prune {
        let mut collection = store.load()?;
        let stale: Vec<Bookmark> = collection.stale().cloned().collect();

        if stale.is_empty() {
            eprintln!("Nothing to prune, all bookmarks are valid.");
            return Ok(());
        }

        eprintln!("The following bookmarks will be removed:");
        for b in &stale {
            eprintln!("  \u{2717} {} | {}", b.name, b.path);
        }

        if !cli.yes {
            let confirmed = Confirm::new()
                .with_prompt(format!("Remove {} bookmark(s)?", stale.len()))
                .default(false)
                .interact()
                .map_err(|e| e.to_string())?;
            if !confirmed {
                return Ok(());
            }
        }

        let count = collection.prune();
        store.save(&collection)?;
        eprintln!("Pruned {count} bookmark(s).");
        return Ok(());
    }

    // No args — interactive selection
    let collection = store.load()?;
    if collection.iter().count() == 0 {
        return Err("No bookmarks saved. Use 'goto --add <name>' to save a bookmark.".to_string());
    }

    let selected_path = if std::env::var_os("GOTO_SELECT_FIRST").is_some() {
        collection.iter().next().map(|b| b.path.clone())
    } else {
        selector::select(&collection)?
    };

    if let Some(path) = selected_path {
        if !std::path::Path::new(&path).is_dir() {
            return Err(format!("'{path}': directory no longer exists"));
        }
        println!("{path}");
    }

    Ok(())
}

fn cwd() -> Result<String, String> {
    std::env::current_dir()
        .map_err(|e| e.to_string())
        .map(|p| p.to_string_lossy().to_string())
}
```

- [ ] **Step 4: Run all tests**

```bash
cargo test --locked 2>&1 | tail -20
```

Expected: all tests pass including the 4 new `goto_prune_*` tests.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs tests/cli_smoke.rs
git commit -m "feat: implement --prune command"
```

---

### Task 5: Dim stale entries in the interactive picker

**Files:**
- Modify: `src/selector.rs`

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)]` block in `src/selector.rs`:

```rust
#[test]
fn strip_ansi_removes_escape_sequences() {
    assert_eq!(strip_ansi("\x1b[2mhello\x1b[0m"), "hello");
    assert_eq!(strip_ansi("plain text"), "plain text");
    assert_eq!(strip_ansi("\x1b[2mname\x1b[0m | /path"), "name | /path");
}

#[test]
fn format_items_dims_stale_bookmarks_and_leaves_valid_ones_plain() {
    use crate::bookmarks::{Bookmark, BookmarkCollection};
    let dir = tempfile::tempdir().unwrap();
    let collection = BookmarkCollection::from_vec(vec![
        Bookmark::new("valid", dir.path().to_str().unwrap()),
        Bookmark::new("stale", "/nonexistent/path/xyzzy123"),
    ])
    .unwrap();
    let items = format_items(&collection);
    let valid_item = items.iter().find(|s| s.contains("valid")).unwrap();
    let stale_item = items.iter().find(|s| s.contains("stale")).unwrap();
    assert!(
        !valid_item.starts_with("\x1b[2m"),
        "valid item must not be dimmed: {valid_item:?}"
    );
    assert!(
        stale_item.starts_with("\x1b[2m"),
        "stale item must be dimmed: {stale_item:?}"
    );
    assert!(
        stale_item.ends_with("\x1b[0m"),
        "stale item must end with ANSI reset: {stale_item:?}"
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test "strip_ansi\|dims_stale" --locked 2>&1 | tail -20
```

Expected: compile error — `strip_ansi` not found; `format_items_dims_stale` fails.

- [ ] **Step 3: Replace `src/selector.rs` with the updated version**

Replace the entire content of `src/selector.rs` with:

```rust
use crate::bookmarks::BookmarkCollection;

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for ch in chars.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn format_items(collection: &BookmarkCollection) -> Vec<String> {
    let max_name_len = collection.iter().map(|b| b.name.len()).max().unwrap_or(0);
    collection
        .iter()
        .map(|b| {
            let line = format!("{:<width$} | {}", b.name, b.path, width = max_name_len);
            if b.is_valid() {
                line
            } else {
                format!("\x1b[2m{line}\x1b[0m")
            }
        })
        .collect()
}

/// Returns the selected directory path, or `None` if the user cancelled.
/// Items are shown as `name | path` with names padded to align the `|`,
/// sorted alphabetically. Stale bookmarks (directories that no longer exist)
/// are rendered dimmed.
pub fn select(collection: &BookmarkCollection) -> Result<Option<String>, String> {
    let items = format_items(collection);

    let selected_line = if is_fzf_available() {
        select_with_fzf(&items)?
    } else {
        select_builtin(&items)?
    };

    Ok(selected_line.map(|line| {
        let clean = strip_ansi(&line);
        clean
            .split_once(" | ")
            .map(|(_, path)| path.to_string())
            .unwrap_or(clean)
    }))
}

pub fn is_fzf_available() -> bool {
    std::process::Command::new("fzf")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn select_with_fzf(items: &[String]) -> Result<Option<String>, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("fzf")
        .arg("--ansi")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(mut stdin) = child.stdin.take() {
        for item in items {
            writeln!(stdin, "{item}").map_err(|e| e.to_string())?;
        }
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}

fn select_builtin(items: &[String]) -> Result<Option<String>, String> {
    use dialoguer::theme::ColorfulTheme;
    use dialoguer::Select;

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .default(0)
        .interact_opt()
        .map_err(|e| e.to_string())?;

    Ok(selection.map(|i| items[i].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_fzf_available_does_not_panic() {
        let _ = is_fzf_available();
    }

    #[test]
    fn format_items_aligns_pipe_at_same_column() {
        use crate::bookmarks::{Bookmark, BookmarkCollection};
        let collection = BookmarkCollection::from_vec(vec![
            Bookmark::new("short", "/a"),
            Bookmark::new("a-longer-name", "/b"),
            Bookmark::new("mid", "/c"),
        ])
        .unwrap();
        let items = format_items(&collection);
        let positions: Vec<usize> = items
            .iter()
            .map(|s| {
                let clean = strip_ansi(s);
                clean
                    .find(" | ")
                    .expect("formatted item must contain ' | ' separator")
            })
            .collect();
        assert!(
            positions.windows(2).all(|w| w[0] == w[1]),
            "pipes not aligned: {items:?}"
        );
    }

    #[test]
    fn format_items_returns_empty_vec_for_empty_collection() {
        use crate::bookmarks::BookmarkCollection;
        let collection = BookmarkCollection::from_vec(vec![]).unwrap();
        let items = format_items(&collection);
        assert!(items.is_empty());
    }

    #[test]
    fn strip_ansi_removes_escape_sequences() {
        assert_eq!(strip_ansi("\x1b[2mhello\x1b[0m"), "hello");
        assert_eq!(strip_ansi("plain text"), "plain text");
        assert_eq!(strip_ansi("\x1b[2mname\x1b[0m | /path"), "name | /path");
    }

    #[test]
    fn format_items_dims_stale_bookmarks_and_leaves_valid_ones_plain() {
        use crate::bookmarks::{Bookmark, BookmarkCollection};
        let dir = tempfile::tempdir().unwrap();
        let collection = BookmarkCollection::from_vec(vec![
            Bookmark::new("valid", dir.path().to_str().unwrap()),
            Bookmark::new("stale", "/nonexistent/path/xyzzy123"),
        ])
        .unwrap();
        let items = format_items(&collection);
        let valid_item = items.iter().find(|s| s.contains("valid")).unwrap();
        let stale_item = items.iter().find(|s| s.contains("stale")).unwrap();
        assert!(
            !valid_item.starts_with("\x1b[2m"),
            "valid item must not be dimmed: {valid_item:?}"
        );
        assert!(
            stale_item.starts_with("\x1b[2m"),
            "stale item must be dimmed: {stale_item:?}"
        );
        assert!(
            stale_item.ends_with("\x1b[0m"),
            "stale item must end with ANSI reset: {stale_item:?}"
        );
    }
}
```

Note: `--ansi` is added to the fzf invocation so fzf renders the ANSI dim codes.

- [ ] **Step 4: Run all tests**

```bash
cargo test --locked 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 5: Run CI checks**

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo build --locked && cargo test --locked
```

Expected: all pass with no warnings.

- [ ] **Step 6: Commit**

```bash
git add src/selector.rs
git commit -m "feat: dim stale bookmarks in interactive picker"
```

---

### Task 6: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add `--prune` to the usage block**

In `README.md`, find the usage block:

```
goto --remove <name>     # Delete a bookmark by name
goto --list              # Print all bookmarks as 'name | path'
```

Replace with:

```
goto --remove <name>     # Delete a bookmark by name
goto --list              # Print all bookmarks as 'name | path'
goto --prune             # Remove bookmarks whose directories no longer exist
goto --prune --yes       # Same, without the confirmation prompt
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: document --prune and --prune --yes in README"
```
