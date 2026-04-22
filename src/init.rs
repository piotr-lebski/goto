//! Shell detection and snippet emission for `goto --init`.
//!
//! Public surface:
//! - [`Shell`] — supported shell variants
//! - [`parse_shell`] — parse a CLI string into a `Shell`
//! - [`snippet`] — return the shell wrapper snippet as a `&'static str`
//! - [`detect_shell`] — auto-detect shell from environment variables

const BASH_ZSH_SNIPPET: &str = "\
goto() {
    if [ $# -eq 0 ]; then
        local selected
        selected=$(command goto)
        local exit_code=$?
        if [ $exit_code -ne 0 ]; then
            return $exit_code
        fi
        if [ -n \"$selected\" ]; then
            cd \"$selected\" || return 1
        fi
    else
        command goto \"$@\"
    fi
}";

const FISH_SNIPPET: &str = "\
function goto
    if test (count $argv) -eq 0
        set selected (command goto)
        if test $status -eq 0; and test -n \"$selected\"
            builtin cd $selected
        end
    else
        command goto $argv
    end
end";

const POWERSHELL_SNIPPET: &str = "\
function Invoke-GoTo {
    param(
        [Parameter(ValueFromRemainingArguments)]
        [string[]]$Arguments
    )

    $binary = Get-Command -Name 'goto' -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1

    if (-not $binary) {
        Write-Error \"goto binary not found in PATH. Make sure goto is installed.\"
        return
    }

    if ($Arguments.Count -eq 0) {
        $selected = & $binary.Source
        if ($LASTEXITCODE -eq 0 -and $selected) {
            Set-Location $selected
        }
    } else {
        & $binary.Source @Arguments
    }
}

Set-Alias -Name goto -Value Invoke-GoTo -Scope Global";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

pub fn parse_shell(name: &str) -> Result<Shell, String> {
    match name.to_lowercase().as_str() {
        "bash" => Ok(Shell::Bash),
        "zsh" => Ok(Shell::Zsh),
        "fish" => Ok(Shell::Fish),
        "powershell" | "pwsh" => Ok(Shell::PowerShell),
        other => Err(format!(
            "Unknown shell '{other}'. Supported: bash, zsh, fish, powershell (or pwsh)"
        )),
    }
}

pub fn snippet(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash | Shell::Zsh => BASH_ZSH_SNIPPET,
        Shell::Fish => FISH_SNIPPET,
        Shell::PowerShell => POWERSHELL_SNIPPET,
    }
}

pub fn detect_shell() -> Result<Shell, String> {
    detect_shell_from_env(|key| std::env::var(key).ok())
}

fn detect_shell_from_env<F: Fn(&str) -> Option<String>>(env: F) -> Result<Shell, String> {
    if env("BASH_VERSION").is_some() {
        return Ok(Shell::Bash);
    }
    if env("ZSH_VERSION").is_some() {
        return Ok(Shell::Zsh);
    }
    if env("FISH_VERSION").is_some() {
        return Ok(Shell::Fish);
    }
    if env("PSModulePath").is_some() {
        return Ok(Shell::PowerShell);
    }
    // PowerShell is not in the $SHELL fallback — it sets PSModulePath instead.
    if let Some(shell_path) = env("SHELL") {
        let name = shell_path.rsplit('/').next().unwrap_or("").to_lowercase();
        return match name.as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            other => Err(format!(
                "Unrecognized shell '{other}' in $SHELL. \
                 Please specify with: goto --init <bash|zsh|fish|powershell>"
            )),
        };
    }
    Err("Could not detect shell. \
         Please specify with: goto --init <bash|zsh|fish|powershell>"
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_shell ---

    #[test]
    fn parse_shell_bash() {
        assert_eq!(parse_shell("bash").unwrap(), Shell::Bash);
    }

    #[test]
    fn parse_shell_zsh() {
        assert_eq!(parse_shell("zsh").unwrap(), Shell::Zsh);
    }

    #[test]
    fn parse_shell_fish() {
        assert_eq!(parse_shell("fish").unwrap(), Shell::Fish);
    }

    #[test]
    fn parse_shell_powershell() {
        assert_eq!(parse_shell("powershell").unwrap(), Shell::PowerShell);
    }

    #[test]
    fn parse_shell_pwsh_alias() {
        assert_eq!(parse_shell("pwsh").unwrap(), Shell::PowerShell);
    }

    #[test]
    fn parse_shell_is_case_insensitive() {
        assert_eq!(parse_shell("Bash").unwrap(), Shell::Bash);
        assert_eq!(parse_shell("ZSH").unwrap(), Shell::Zsh);
    }

    #[test]
    fn parse_shell_unknown_returns_error() {
        let err = parse_shell("nushell").unwrap_err();
        assert!(
            err.contains("nushell"),
            "error should name the bad value: {err}"
        );
    }

    // --- snippet ---

    #[test]
    fn bash_snippet_contains_wrapper_function() {
        let s = snippet(Shell::Bash);
        assert!(s.contains("goto()"), "expected function definition: {s}");
        assert!(s.contains("command goto"), "expected binary call: {s}");
    }

    #[test]
    fn zsh_snippet_is_identical_to_bash() {
        assert_eq!(snippet(Shell::Bash), snippet(Shell::Zsh));
    }

    #[test]
    fn fish_snippet_contains_wrapper_function() {
        let s = snippet(Shell::Fish);
        assert!(
            s.contains("function goto"),
            "expected function definition: {s}"
        );
        assert!(s.contains("command goto"), "expected binary call: {s}");
    }

    #[test]
    fn powershell_snippet_contains_wrapper_function() {
        let s = snippet(Shell::PowerShell);
        assert!(s.contains("Invoke-GoTo"), "expected PS function: {s}");
        assert!(s.contains("Set-Alias"), "expected alias: {s}");
    }

    // --- detect_shell_from_env ---

    #[test]
    fn detects_bash_from_bash_version() {
        let shell = detect_shell_from_env(|key| {
            if key == "BASH_VERSION" {
                Some("5.0".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Bash);
    }

    #[test]
    fn detects_zsh_from_zsh_version() {
        let shell = detect_shell_from_env(|key| {
            if key == "ZSH_VERSION" {
                Some("5.9".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Zsh);
    }

    #[test]
    fn detects_fish_from_fish_version() {
        let shell = detect_shell_from_env(|key| {
            if key == "FISH_VERSION" {
                Some("3.7".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Fish);
    }

    #[test]
    fn detects_powershell_from_ps_module_path() {
        let shell = detect_shell_from_env(|key| {
            if key == "PSModulePath" {
                Some("/path/modules".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::PowerShell);
    }

    #[test]
    fn bash_version_takes_priority_over_zsh_version() {
        let shell = detect_shell_from_env(|key| match key {
            "BASH_VERSION" => Some("5.0".to_string()),
            "ZSH_VERSION" => Some("5.9".to_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(shell, Shell::Bash);
    }

    #[test]
    fn detects_bash_from_shell_env_path() {
        let shell = detect_shell_from_env(|key| {
            if key == "SHELL" {
                Some("/usr/bin/bash".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Bash);
    }

    #[test]
    fn detects_zsh_from_shell_env_path() {
        let shell = detect_shell_from_env(|key| {
            if key == "SHELL" {
                Some("/bin/zsh".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Zsh);
    }

    #[test]
    fn detects_fish_from_shell_env_path() {
        let shell = detect_shell_from_env(|key| {
            if key == "SHELL" {
                Some("/usr/local/bin/fish".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Fish);
    }

    #[test]
    fn returns_error_for_unrecognized_shell_env_path() {
        let err = detect_shell_from_env(|key| {
            if key == "SHELL" {
                Some("/usr/bin/nushell".to_string())
            } else {
                None
            }
        })
        .unwrap_err();
        assert!(
            err.contains("nushell"),
            "error should name the shell: {err}"
        );
    }

    #[test]
    fn returns_error_when_no_shell_hints_present() {
        let err = detect_shell_from_env(|_| None).unwrap_err();
        assert!(
            err.contains("Could not detect shell"),
            "expected detection failure message: {err}"
        );
    }

    #[test]
    fn version_env_var_takes_priority_over_shell_path() {
        let shell = detect_shell_from_env(|key| match key {
            "BASH_VERSION" => Some("5.0".to_string()),
            "SHELL" => Some("/bin/zsh".to_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(shell, Shell::Bash);
    }

    #[test]
    fn detects_bash_from_bare_shell_env_name() {
        let shell = detect_shell_from_env(|key| {
            if key == "SHELL" {
                Some("bash".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Bash);
    }

    #[test]
    fn detects_bash_from_mixed_case_shell_env_path() {
        let shell = detect_shell_from_env(|key| {
            if key == "SHELL" {
                Some("/usr/bin/Bash".to_string())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(shell, Shell::Bash);
    }
}
