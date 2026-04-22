use crate::bookmarks::BookmarkCollection;
use std::path::{Path, PathBuf};

pub struct Store {
    config_dir: PathBuf,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            config_dir: Self::resolve_config_dir(),
        }
    }

    pub fn from_override(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    pub fn resolve_config_dir() -> PathBuf {
        std::env::var_os("GOTO_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn bookmarks_path(&self) -> PathBuf {
        self.config_dir.join("goto").join("bookmarks.json")
    }

    pub fn load(&self) -> Result<BookmarkCollection, String> {
        let path = self.bookmarks_path();
        if !path.exists() {
            return Ok(BookmarkCollection::default());
        }

        let contents = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        serde_json::from_str(&contents)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))
    }

    pub fn save(&self, bookmarks: &BookmarkCollection) -> Result<(), String> {
        let path = self.bookmarks_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let json = serde_json::to_string_pretty(bookmarks).map_err(|error| error.to_string())?;
        std::fs::write(path, json).map_err(|error| error.to_string())
    }
}
