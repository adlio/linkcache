use crate::ddl::apply_migrations;
use crate::Cache;
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct CacheBuilder {
    data_dir: Option<PathBuf>,
}

impl CacheBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    ///
    pub fn with_data_dir<P: AsRef<Path>>(mut self, data_dir: P) -> Self {
        let path: PathBuf = data_dir.as_ref().to_path_buf();
        self.data_dir = Some(path);
        self
    }

    pub fn build(self) -> crate::Result<Cache> {
        // Ensure all storage directories exist
        let data_dir = self.data_dir.unwrap_or_else(Self::default_data_dir);

        // Create the connection to the SQLite database
        let db_path = data_dir.join("linkcache.sqlite");
        let mut conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        apply_migrations(&mut conn)?;

        Ok(Cache { conn })
    }

    /// Returns the default data directory to be used when creating Cache
    /// instances.
    ///
    /// We use ~/.linkcache by default unless the user's home directory cannot
    /// be determined, in which case we use /tmp/.linkcache instead.
    ///
    #[must_use]
    fn default_data_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".linkcache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    fn test_cache_instance() -> (Cache, TempDir) {
        let binding = tempdir().expect("Failed to create temp dir");
        let temp_dir = binding.path();
        let cache = Cache::builder()
            .with_data_dir(temp_dir)
            .build()
            .expect("Failed to create test cache");
        (cache, binding)
    }

    #[test]
    fn test_build_with_custom_data_dir() {
        let (_cache, _temp_dir) = test_cache_instance();
    }

    #[test]
    fn test_default_data_dir() {
        let dir = CacheBuilder::default_data_dir();
        assert!(dir.exists(), "Expected default_data_dir to exist");
    }
}
