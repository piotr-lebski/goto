use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "goto", about = "Bookmark-based directory navigation")]
pub struct Cli {
    /// Print shell integration snippet and exit.
    /// Auto-detects the shell when SHELL is omitted.
    /// Supported: bash, zsh, fish, powershell
    #[arg(
        long,
        value_name = "SHELL",
        num_args = 0..=1,
        default_missing_value = "auto",
        conflicts_with_all = ["list", "add", "replace", "remove"]
    )]
    pub init: Option<String>,

    /// Print all bookmarks as 'name | path'
    #[arg(long, conflicts_with_all = ["add", "replace", "remove"])]
    pub list: bool,

    /// Save the current directory as NAME
    #[arg(long, value_name = "NAME", conflicts_with_all = ["list", "replace", "remove"])]
    pub add: Option<String>,

    /// Update the saved path for an existing bookmark NAME
    #[arg(long, value_name = "NAME", conflicts_with_all = ["list", "add", "remove"])]
    pub replace: Option<String>,

    /// Delete bookmark NAME
    #[arg(long, value_name = "NAME", conflicts_with_all = ["list", "add", "replace"])]
    pub remove: Option<String>,
}
