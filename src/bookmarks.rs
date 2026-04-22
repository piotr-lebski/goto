use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Bookmark {
    pub name: String,
    pub path: String,
}

impl Bookmark {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        std::path::Path::new(&self.path).is_dir()
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct BookmarkCollection {
    bookmarks: Vec<Bookmark>,
}

impl BookmarkCollection {
    pub fn from_vec(bookmarks: Vec<Bookmark>) -> Result<Self, String> {
        Ok(Self::from_unsorted_vec(bookmarks))
    }

    fn from_unsorted_vec(mut bookmarks: Vec<Bookmark>) -> Self {
        bookmarks.sort_by(|left, right| left.name.cmp(&right.name));
        Self { bookmarks }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Bookmark> {
        self.bookmarks.iter()
    }

    pub fn add(&mut self, name: impl Into<String>, path: impl Into<String>) -> Result<(), String> {
        let name = name.into();
        if self.bookmarks.iter().any(|b| b.name == name) {
            return Err(format!(
                "bookmark '{name}' already exists; use --replace to update it"
            ));
        }
        self.bookmarks.push(Bookmark::new(name, path));
        self.bookmarks.sort_by(|l, r| l.name.cmp(&r.name));
        Ok(())
    }

    pub fn replace(&mut self, name: &str, path: impl Into<String>) -> Result<(), String> {
        let bookmark = self
            .bookmarks
            .iter_mut()
            .find(|b| b.name == name)
            .ok_or_else(|| format!("bookmark '{name}' does not exist; use --add to create it"))?;
        bookmark.path = path.into();
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<(), String> {
        let pos = self
            .bookmarks
            .iter()
            .position(|b| b.name == name)
            .ok_or_else(|| format!("bookmark '{name}' does not exist"))?;
        self.bookmarks.remove(pos);
        Ok(())
    }

    pub fn stale(&self) -> impl Iterator<Item = &Bookmark> {
        self.bookmarks.iter().filter(|b| !b.is_valid())
    }

    /// Removes all stale bookmarks in a single pass and returns them.
    /// Prefer this over calling [`stale`] followed by [`prune`] to avoid
    /// scanning the filesystem twice.
    pub fn drain_stale(&mut self) -> Vec<Bookmark> {
        let mut removed = Vec::new();
        self.bookmarks.retain(|b| {
            if b.is_valid() {
                true
            } else {
                removed.push(b.clone());
                false
            }
        });
        removed
    }

    pub fn prune(&mut self) -> usize {
        self.drain_stale().len()
    }
}

#[derive(Deserialize)]
struct BookmarkCollectionRepr {
    bookmarks: Vec<Bookmark>,
}

impl<'de> Deserialize<'de> for BookmarkCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = BookmarkCollectionRepr::deserialize(deserializer)?;
        Ok(Self::from_unsorted_vec(repr.bookmarks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(pairs: &[(&str, &str)]) -> BookmarkCollection {
        BookmarkCollection::from_vec(pairs.iter().map(|(n, p)| Bookmark::new(*n, *p)).collect())
            .unwrap()
    }

    #[test]
    fn add_inserts_and_maintains_alphabetical_order() {
        let mut c = col(&[("beta", "/b")]);
        c.add("alpha", "/a").unwrap();
        let names: Vec<_> = c.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn add_rejects_duplicate_name() {
        let mut c = col(&[("alpha", "/a")]);
        let err = c.add("alpha", "/a2").unwrap_err();
        assert!(err.contains("already exists"), "got: {err}");
        assert!(err.contains("--replace"), "got: {err}");
    }

    #[test]
    fn replace_updates_path_for_existing_bookmark() {
        let mut c = col(&[("alpha", "/a")]);
        c.replace("alpha", "/a2").unwrap();
        assert_eq!(c.iter().next().unwrap().path, "/a2");
    }

    #[test]
    fn replace_rejects_missing_name() {
        let mut c = col(&[]);
        let err = c.replace("ghost", "/x").unwrap_err();
        assert!(err.contains("does not exist"), "got: {err}");
        assert!(err.contains("--add"), "got: {err}");
    }

    #[test]
    fn remove_deletes_named_bookmark() {
        let mut c = col(&[("alpha", "/a"), ("beta", "/b")]);
        c.remove("alpha").unwrap();
        let names: Vec<_> = c.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["beta"]);
    }

    #[test]
    fn remove_rejects_missing_name() {
        let mut c = col(&[]);
        let err = c.remove("ghost").unwrap_err();
        assert!(err.contains("does not exist"), "got: {err}");
    }

    #[test]
    fn is_valid_returns_true_for_existing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let b = Bookmark::new("test", dir.path().to_str().unwrap());
        assert!(b.is_valid());
    }

    #[test]
    fn is_valid_returns_false_for_nonexistent_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");
        let b = Bookmark::new("test", missing.to_str().unwrap());
        assert!(!b.is_valid());
    }

    #[test]
    fn stale_returns_only_invalid_bookmarks() {
        let valid_dir = tempfile::tempdir().unwrap();
        let stale_base = tempfile::tempdir().unwrap();
        let missing = stale_base.path().join("missing");
        let c = col(&[
            ("valid", valid_dir.path().to_str().unwrap()),
            ("stale", missing.to_str().unwrap()),
        ]);
        let stale_names: Vec<_> = c.stale().map(|b| b.name.as_str()).collect();
        assert_eq!(stale_names, vec!["stale"]);
    }

    #[test]
    fn prune_removes_stale_entries_and_returns_count() {
        let valid_dir = tempfile::tempdir().unwrap();
        let stale_base = tempfile::tempdir().unwrap();
        let missing = stale_base.path().join("missing");
        let mut c = col(&[
            ("valid", valid_dir.path().to_str().unwrap()),
            ("stale", missing.to_str().unwrap()),
        ]);
        let removed = c.prune();
        assert_eq!(removed, 1);
        let names: Vec<_> = c.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["valid"]);
    }

    #[test]
    fn drain_stale_returns_removed_bookmarks_and_leaves_valid_ones() {
        let valid_dir = tempfile::tempdir().unwrap();
        let stale_base = tempfile::tempdir().unwrap();
        let missing = stale_base.path().join("missing");
        let mut c = col(&[
            ("valid", valid_dir.path().to_str().unwrap()),
            ("stale", missing.to_str().unwrap()),
        ]);
        let removed = c.drain_stale();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].name, "stale");
        let names: Vec<_> = c.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["valid"]);
    }
}
