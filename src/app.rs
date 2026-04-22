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
            if collection.iter().count() == 0 {
                eprintln!("Nothing to prune (no bookmarks saved).");
            } else {
                eprintln!("Nothing to prune, all bookmarks are valid.");
            }
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
