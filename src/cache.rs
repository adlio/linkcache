use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

use crate::{error::Result, Link};

pub struct Cache {
    pub(crate) conn: Connection,
}

impl Cache {
    /// Create a new Cache instance with the SQLite database at the provided
    /// path. This could fail if the path doesn't exist, or the file isn't
    /// writeable, or the initialization process (creation of tables,
    /// triggers, etc) fails.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        let cache = Cache { conn };
        cache.initialize()?;
        Ok(cache)
    }

    pub fn default() -> Result<Self> {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".linkcache");
        std::fs::create_dir_all(&cache_dir)?;
        let db_path = cache_dir.join("linkcache.sqlite");
        Self::new(db_path)
    }

    /// Adds a new link to the index. The url field is used as the unique
    /// key. This function removes any existing link with the same url before
    /// saving a new one. The commit() function must be called after adding
    /// to persist the changes. Batch updates should call add() many times
    /// and commit() once.
    pub fn add(&mut self, link: Link) -> Result<()> {
        // let json_str = to_string(&link)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO links (
                url, title, subtitle,
                source, author,
                timestamp
            ) VALUES (
                ?1, ?2, ?3,
                ?4, ?5,
                ?6
            )",
            (
                &link.url,
                &link.title,
                &link.subtitle,
                &link.source,
                &link.author,
                &link.timestamp,
            ),
        )?;
        Ok(())
    }

    /// Removes a Link from the index. The url field is used as the unique key.
    pub fn remove(&mut self, link: &Link) -> Result<()> {
        self.conn
            .execute("DELETE FROM links WHERE url = ?1", [&link.url])?;

        Ok(())
    }

    /// Searches the index for linkx matching the query
    pub fn search(&self, query: &str) -> Result<Vec<Link>> {
        if query.is_empty() {
            return self.get_latest_n(50);
        }

        let mut stmt = self.conn.prepare(
            "SELECT links.* FROM links_fts
             JOIN links ON links_fts.url = links.url
             WHERE links_fts MATCH ?1
             ORDER BY rank",
        )?;

        let links_iter = stmt.query_map([query], |row| {
            Ok(Link {
                url: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get(2)?,
                source: row.get(3)?,
                author: row.get(4)?,
                timestamp: row.get(5)?,
                ..Default::default()
            })
        })?;

        links_iter
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| e.into())
    }

    pub fn get_latest_n(&self, n: u32) -> Result<Vec<Link>> {
        let mut stmt = self.conn.prepare(
            "SELECT url, title, subtitle, source, author, timestamp 
             FROM links 
             ORDER BY timestamp DESC 
             LIMIT ?",
        )?;

        let links_iter = stmt.query_map([n], |row| {
            Ok(Link {
                url: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get(2)?,
                source: row.get(3)?,
                author: row.get(4)?,
                timestamp: row.get(5)?,
                ..Default::default()
            })
        })?;

        links_iter
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| e.into())
    }
}

/// Defines the Default implementaton for Cache.
impl Default for Cache {
    fn default() -> Self {
        Self::default().expect("Failed to create default cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    fn test_cache_instance() -> (Cache, TempDir) {
        let binding = tempdir().expect("Failed to create temp dir");
        let temp_dir = binding.path();
        let cache = Cache::new(temp_dir.join("test.sqlite")).expect("Failed to create test cache");
        (cache, binding)
    }

    #[test]
    fn test_add_and_search_fuzzy() -> Result<()> {
        let (mut cache, _temp_dir) = test_cache_instance();
        cache.add(Link {
            title: "Visual Studio Code".to_string(),
            url: "https://code.visualstudio.com".to_string(),
            ..Default::default()
        })?;
        cache.add(Link {
            title: "Sublime Text".to_string(),
            url: "https://www.sublimetext.com".to_string(),
            ..Default::default()
        })?;
        let results = cache.search("Vis studio")?;
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Visual Studio Code");
        Ok(())
    }
}
