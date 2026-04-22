use std::process::Command;

#[test]
fn goto_list_exits_successfully_for_empty_store() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--list")
        .env("GOTO_CONFIG_HOME", tempfile::tempdir().unwrap().path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
}

#[test]
fn goto_list_prints_bookmarks_sorted_as_name_pipe_path() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let bookmarks_dir = config_dir.join("goto");
    std::fs::create_dir_all(&bookmarks_dir).unwrap();
    std::fs::write(
        bookmarks_dir.join("bookmarks.json"),
        r#"{"bookmarks":[{"name":"work","path":"/home/user/work"},{"name":"alpha","path":"/tmp/alpha"}]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--list")
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "alpha | /tmp/alpha\nwork | /home/user/work\n");
}

#[test]
fn goto_add_saves_bookmark_for_current_directory() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let target_dir = tempfile::tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--add", "mydir"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .current_dir(target_dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let list_output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--list")
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("mydir |"), "bookmark not listed: {stdout}");
    assert!(
        stdout.contains(target_dir.path().to_string_lossy().as_ref()),
        "path not listed: {stdout}"
    );
}

#[test]
fn goto_add_fails_for_duplicate_name() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();

    Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--add", "mydir"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .current_dir(temp.path())
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--add", "mydir"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists"),
        "expected duplicate error: {stderr}"
    );
}

#[test]
fn goto_replace_updates_existing_bookmark() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();

    Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--add", "mydir"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .current_dir(dir_a.path())
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--replace", "mydir"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .current_dir(dir_b.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let list_output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--list")
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    let expected_path = dir_b.path().to_string_lossy();
    assert!(
        stdout.contains(expected_path.as_ref()),
        "expected {expected_path} in: {stdout}"
    );
}

#[test]
fn goto_replace_fails_for_missing_bookmark() {
    let temp = tempfile::tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--replace", "nonexistent"])
        .env("GOTO_CONFIG_HOME", temp.path())
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist"),
        "expected missing-bookmark error: {stderr}"
    );
}

#[test]
fn goto_remove_deletes_existing_bookmark() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();

    Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--add", "mydir"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .current_dir(temp.path())
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--remove", "mydir"])
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let list_output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--list")
        .env("GOTO_CONFIG_HOME", config_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        !stdout.contains("mydir"),
        "bookmark should be gone: {stdout}"
    );
}

#[test]
fn goto_remove_fails_for_missing_bookmark() {
    let temp = tempfile::tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--remove", "nonexistent"])
        .env("GOTO_CONFIG_HOME", temp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist"),
        "expected missing-bookmark error: {stderr}"
    );
}

#[test]
fn goto_no_args_prints_error_message_when_no_bookmarks() {
    let temp = tempfile::tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .env("GOTO_CONFIG_HOME", temp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No bookmarks"),
        "expected empty-state message: {stderr}"
    );
}

#[test]
fn goto_no_args_prints_path_to_stdout_on_selection() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let target_dir = tempfile::tempdir().unwrap();
    let bookmarks_dir = config_dir.join("goto");
    std::fs::create_dir_all(&bookmarks_dir).unwrap();
    std::fs::write(
        bookmarks_dir.join("bookmarks.json"),
        format!(
            r#"{{"bookmarks":[{{"name":"target","path":"{}"}}]}}"#,
            target_dir.path().to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .env("GOTO_CONFIG_HOME", config_dir)
        .env("GOTO_SELECT_FIRST", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, target_dir.path().to_string_lossy().as_ref());
}

#[test]
fn goto_no_args_exits_nonzero_for_stale_bookmark() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path();
    let bookmarks_dir = config_dir.join("goto");
    std::fs::create_dir_all(&bookmarks_dir).unwrap();
    std::fs::write(
        bookmarks_dir.join("bookmarks.json"),
        r#"{"bookmarks":[{"name":"stale","path":"/nonexistent/path/aabbcc"}]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .env("GOTO_CONFIG_HOME", config_dir)
        .env("GOTO_SELECT_FIRST", "1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no longer exists"),
        "expected stale-path error: {stderr}"
    );
}

#[test]
fn goto_init_bash_prints_shell_function() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--init", "bash"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("goto()"),
        "expected bash function: {stdout}"
    );
    assert!(
        stdout.contains("command goto"),
        "expected binary call: {stdout}"
    );
}

#[test]
fn goto_init_zsh_prints_shell_function() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--init", "zsh"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("goto()"), "expected zsh function: {stdout}");
    assert!(
        stdout.contains("command goto"),
        "expected binary call: {stdout}"
    );
}

#[test]
fn goto_init_fish_prints_shell_function() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--init", "fish"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("function goto"),
        "expected fish function: {stdout}"
    );
    assert!(
        stdout.contains("command goto"),
        "expected binary call: {stdout}"
    );
}

#[test]
fn goto_init_powershell_prints_shell_function() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--init", "powershell"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Invoke-GoTo"),
        "expected PS function: {stdout}"
    );
    assert!(stdout.contains("Set-Alias"), "expected alias: {stdout}");
}

#[test]
fn goto_init_unknown_shell_exits_nonzero_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(["--init", "nushell"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("nushell"),
        "expected error naming the bad shell: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "expected no stdout on error, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn goto_init_auto_detects_bash_from_env() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--init")
        .env("BASH_VERSION", "5.0.0")
        .env_remove("ZSH_VERSION")
        .env_remove("FISH_VERSION")
        .env_remove("PSModulePath")
        .env_remove("SHELL")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("goto()"),
        "expected bash function: {stdout}"
    );
}

#[test]
fn goto_init_auto_detects_zsh_from_env() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--init")
        .env("ZSH_VERSION", "5.9")
        .env_remove("BASH_VERSION")
        .env_remove("FISH_VERSION")
        .env_remove("PSModulePath")
        .env_remove("SHELL")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("goto()"), "expected zsh function: {stdout}");
}

#[test]
fn goto_init_auto_detects_fish_from_env() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--init")
        .env("FISH_VERSION", "3.7")
        .env_remove("BASH_VERSION")
        .env_remove("ZSH_VERSION")
        .env_remove("PSModulePath")
        .env_remove("SHELL")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("function goto"),
        "expected fish function: {stdout}"
    );
}

#[test]
fn goto_init_auto_detect_fails_when_no_shell_env() {
    let output = Command::new(env!("CARGO_BIN_EXE_goto"))
        .arg("--init")
        .env_remove("BASH_VERSION")
        .env_remove("ZSH_VERSION")
        .env_remove("FISH_VERSION")
        .env_remove("PSModulePath")
        .env_remove("SHELL")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Could not detect shell"),
        "expected detection failure message: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "expected no stdout on error, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

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
