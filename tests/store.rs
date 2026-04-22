use goto::bookmarks::{Bookmark, BookmarkCollection};
use goto::store::Store;
use std::sync::Mutex;
use tempfile::tempdir;

// Mutex to serialize env-var tests and prevent data races
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn save_and_load_round_trip_keeps_alphabetical_order() {
    let temp = tempdir().unwrap();
    let store = Store::from_override(temp.path().to_path_buf());

    let bookmarks = BookmarkCollection::from_vec(vec![
        Bookmark::new("zeta", "/tmp/zeta"),
        Bookmark::new("alpha", "/tmp/alpha"),
    ])
    .unwrap();

    store.save(&bookmarks).unwrap();
    let loaded = store.load().unwrap();

    let names: Vec<_> = loaded
        .iter()
        .map(|bookmark| bookmark.name.as_str())
        .collect();
    assert_eq!(names, vec!["alpha", "zeta"]);
}

#[test]
fn load_returns_error_for_invalid_json() {
    let temp = tempdir().unwrap();
    let store = Store::from_override(temp.path().to_path_buf());
    std::fs::create_dir_all(store.bookmarks_path().parent().unwrap()).unwrap();
    std::fs::write(store.bookmarks_path(), "{not-json").unwrap();

    let error = store.load().unwrap_err();
    assert!(error.to_string().contains("failed to parse"));
}

#[test]
fn load_normalizes_unsorted_bookmarks_from_disk() {
    let temp = tempdir().unwrap();
    let store = Store::from_override(temp.path().to_path_buf());
    std::fs::create_dir_all(store.bookmarks_path().parent().unwrap()).unwrap();
    std::fs::write(
        store.bookmarks_path(),
        r#"{"bookmarks":[{"name":"zeta","path":"/tmp/zeta"},{"name":"alpha","path":"/tmp/alpha"}]}"#,
    )
    .unwrap();

    let loaded = store.load().unwrap();
    let names: Vec<_> = loaded
        .iter()
        .map(|bookmark| bookmark.name.as_str())
        .collect();
    assert_eq!(names, vec!["alpha", "zeta"]);
}

#[test]
fn resolve_config_dir_uses_goto_config_home_first() {
    let _guard = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var("GOTO_CONFIG_HOME", "/custom/goto/home");
        std::env::remove_var("XDG_CONFIG_HOME");
    }
    let result = std::panic::catch_unwind(Store::resolve_config_dir);
    unsafe {
        std::env::remove_var("GOTO_CONFIG_HOME");
        std::env::remove_var("XDG_CONFIG_HOME");
    }
    let result = result.unwrap();
    assert_eq!(result, std::path::PathBuf::from("/custom/goto/home"));
}

#[test]
fn resolve_config_dir_falls_back_to_xdg_config_home() {
    // Verifies that XDG_CONFIG_HOME is honoured on all platforms when GOTO_CONFIG_HOME is absent.
    // Previously, dirs::config_dir() did not respect XDG_CONFIG_HOME on macOS/Windows.
    let _guard = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("GOTO_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", "/custom/xdg");
    }
    let result = std::panic::catch_unwind(Store::resolve_config_dir);
    unsafe {
        std::env::remove_var("GOTO_CONFIG_HOME");
        std::env::remove_var("XDG_CONFIG_HOME");
    }
    let result = result.unwrap();
    assert_eq!(result, std::path::PathBuf::from("/custom/xdg"));
}

#[test]
fn resolve_config_dir_defaults_to_dot_config_under_home() {
    let _guard = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("GOTO_CONFIG_HOME");
        std::env::remove_var("XDG_CONFIG_HOME");
    }
    let result = std::panic::catch_unwind(Store::resolve_config_dir);
    unsafe {
        std::env::remove_var("GOTO_CONFIG_HOME");
        std::env::remove_var("XDG_CONFIG_HOME");
    }
    let result = result.unwrap();
    let expected = dirs::home_dir().unwrap().join(".config");
    assert_eq!(
        result,
        expected,
        "expected $HOME/.config, got: {}",
        result.display()
    );
}
