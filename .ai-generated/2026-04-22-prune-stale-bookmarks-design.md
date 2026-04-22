# Design: Prune Stale Bookmarks + Grey-out in Picker

## Problem

Bookmarks can become stale when the saved directory is deleted or moved. There is no
way to bulk-remove invalid bookmarks, and the interactive picker gives no visual hint
that a bookmark is stale before the user selects it.

## Goals

1. `goto --prune` — remove all bookmarks whose directories no longer exist, with a
   confirmation prompt and an optional `--yes` flag to skip it.
2. In the interactive picker (`goto` with no args), stale bookmarks are visually
   dimmed (ANSI dim) so the user can see at a glance that they are stale. Selecting
   one still works — the existing error path fires (`"'<path>': directory no longer
   exists"`, exit non-zero).

## Out of Scope

- Automatically pruning on `goto` invocation without an explicit command.
- Any changes to bookmark serialisation / storage format.

---

## Architecture

### `bookmarks.rs` — staleness in the data model

```
Bookmark::is_valid() -> bool
    std::path::Path::new(&self.path).is_dir()

BookmarkCollection::stale() -> impl Iterator<Item = &Bookmark>
    bookmarks where !b.is_valid()

BookmarkCollection::prune() -> usize
    retains only valid bookmarks; returns count removed
```

### `cli.rs` — new flags

| Flag | Type | Conflicts with |
|------|------|---------------|
| `--prune` | bool | `add`, `replace`, `remove`, `list`, `init` |
| `--yes` | bool | `add`, `replace`, `remove`, `list`, `init` |

`--yes` is only meaningful with `--prune` but is not structurally restricted to it
(clap keeps the flag surface simple; `app.rs` ignores `--yes` when `--prune` is not set).

### `app.rs` — prune command flow

```
load collection
stale = collection.stale().collect()

if stale.is_empty():
    print "Nothing to prune, all bookmarks are valid."
    return Ok(())

print "The following bookmarks will be removed:"
for each stale bookmark:
    print "  ✗ name | path"

if not cli.yes:
    dialoguer::Confirm "Remove N bookmark(s)? [y/N]"
    if not confirmed: return Ok(())   // user aborted, exit 0

count = collection.prune()
store.save(&collection)
print "Pruned N bookmark(s)."
```

### `selector.rs` — dimmed rendering

`format_items` gains an `is_valid` check per bookmark:

- Valid bookmark: `"name   | path"` (unchanged)
- Stale bookmark: `"\x1b[2mname   | path\x1b[0m"` (ANSI dim)

ANSI dim codes are passed through to fzf unchanged — fzf renders them natively.
The path is parsed from the selected line by splitting on `" | "` (unchanged); ANSI
codes appear before the name, so the split still works correctly.

---

## Error Handling

| Scenario | Behaviour |
|----------|-----------|
| `--prune` on empty bookmark store | Prints "Nothing to prune" and exits 0 |
| `--prune` with no stale bookmarks | Prints "Nothing to prune" and exits 0 |
| User declines confirmation | Exits 0 with no changes |
| `--prune --yes` | Skips prompt, prunes, exits 0 |
| Store save failure | Returns existing `Err` (unchanged) |

---

## Testing

### Unit tests (`bookmarks.rs`)

- `is_valid()` returns `true` for a real temp dir, `false` for a nonexistent path
- `stale()` returns only the invalid bookmarks from a mixed collection
- `prune()` removes all stale entries and returns the correct removed count

### Unit tests (`selector.rs`)

- `format_items` wraps stale bookmark entries in ANSI dim codes
- `format_items` does not add ANSI codes to valid bookmark entries
- Pipe alignment is maintained for mixed valid/stale collections

### Integration tests (`tests/cli_smoke.rs`)

- `goto --prune` on all-valid bookmarks: prints "Nothing to prune", exits 0
- `goto --prune` on mixed bookmarks: lists stale entries, then aborts on `n` input
- `goto --prune --yes` on mixed bookmarks: removes stale entries, exits 0, removed entries absent from `--list`
- `goto --prune` on empty store: prints "Nothing to prune", exits 0

---

## Documentation

`README.md` usage block gains:

```
goto --prune         # Remove bookmarks whose directories no longer exist
goto --prune --yes   # Same, without the confirmation prompt
```
