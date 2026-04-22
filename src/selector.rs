use crate::bookmarks::BookmarkCollection;

fn format_items(collection: &BookmarkCollection) -> Vec<String> {
    let max_name_len = collection.iter().map(|b| b.name.len()).max().unwrap_or(0);
    collection
        .iter()
        .map(|b| format!("{:<width$} | {}", b.name, b.path, width = max_name_len))
        .collect()
}

/// Returns the selected directory path, or `None` if the user cancelled.
/// Items are shown as `name | path` with names padded to align the `|`,
/// sorted alphabetically.
pub fn select(collection: &BookmarkCollection) -> Result<Option<String>, String> {
    let items = format_items(collection);

    let selected_line = if is_fzf_available() {
        select_with_fzf(&items)?
    } else {
        select_builtin(&items)?
    };

    Ok(selected_line.map(|line| {
        line.split_once(" | ")
            .map(|(_, path)| path.to_string())
            .unwrap_or(line)
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
            .map(|s| s.find(" | ").expect("formatted item must contain ' | ' separator"))
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
}
