use rusqlite::Connection;
use std::path::PathBuf;

use crate::CacheBuilder;
use crate::{error::Result, Link};

#[derive(Debug)]
pub struct Cache {
    pub data_dir: PathBuf,
    pub(crate) conn: Connection,
}

impl Cache {
    /// The primary entry point to create a new Cache instance. This function
    /// will create a new Cache instance with the default data directory (~/.linkcache).
    /// If you want to use a custom data directory, use the builder() function
    /// instead.
    ///
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Builder pattern constructor. Use this to override the data directory
    /// and other settings for the Cache.
    ///
    pub fn builder() -> CacheBuilder {
        CacheBuilder::new()
    }

    /// Adds a new link to the index. The url field is used as the unique
    /// key. This function removes any existing link with the same url before
    /// saving a new one. The commit() function must be called after adding
    /// to persist the changes. Batch updates should call add() many times
    /// and commit() once.
    pub fn add(&mut self, link: Link) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO links (
                guid, url, title,
                subtitle, source,
                timestamp
            ) VALUES (
                ?1, ?2, ?3,
                ?4, ?5,
                ?6
            )",
            (
                &link.guid,
                &link.url,
                &link.title,
                &link.subtitle,
                &link.source,
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
            "SELECT
             links.guid, links.url, links.title,
             links.subtitle, links.source,
             links.timestamp
             FROM links_fts
             JOIN links ON links_fts.guid = links.guid
             WHERE links_fts MATCH ?1
             ORDER BY rank",
        )?;

        let links_iter = stmt.query_map([query], |row| {
            Ok(Link {
                guid: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                subtitle: row.get(3)?,
                source: row.get(4)?,
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
            "SELECT guid, url, title, subtitle, source, timestamp
             FROM links
             ORDER BY timestamp DESC 
             LIMIT ?",
        )?;

        let links_iter = stmt.query_map([n], |row| {
            Ok(Link {
                guid: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                subtitle: row.get(3)?,
                source: row.get(4)?,
                timestamp: row.get(5)?,
                ..Default::default()
            })
        })?;

        links_iter
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| e.into())
    }
}

/// Creates a Cache instance with the default storage directory.
/// This will panic if the cache fails to create in the default location.
impl Default for Cache {
    fn default() -> Self {
        Self::builder()
            .build()
            .expect("Failed to create default cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils;

    fn add_link_fixtures(cache: &mut Cache) -> Result<()> {
        // Add with explicit timestamp to ensure consistent ordering
        let now = chrono::Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        
        cache.add(Link {
            guid: "test-guid-1".to_string(),
            title: "Visual Studio Code".to_string(),
            url: "https://code.visualstudio.com".to_string(),
            timestamp: one_hour_ago,
            ..Default::default()
        })?;

        cache.add(Link {
            guid: "test-guid-2".to_string(),
            title: "Sublime Text".to_string(),
            url: "https://www.sublimetext.com".to_string(),
            timestamp: now,
            ..Default::default()
        })?;

        Ok(())
    }

    #[test]
    fn test_get_top_n() -> Result<()> {
        let (mut cache, _temp_dir) = testutils::create_test_cache();
        add_link_fixtures(&mut cache)?;
        let results = cache.get_latest_n(2)?;
        assert_eq!(results.len(), 2); // We add two links in add_link_fixtures
        Ok(())
    }

    #[test]
    fn test_removal() -> Result<()> {
        let (mut cache, _temp_dir) = testutils::create_test_cache();
        add_link_fixtures(&mut cache)?;
        cache.remove(&Link {
            url: "https://www.sublimetext.com".to_string(),
            ..Default::default()
        })?;
        let results = cache.search("Sublime")?;
        assert!(results.is_empty());
        Ok(())
    }

    #[test]
    fn test_search_empty() -> Result<()> {
        let (mut cache, _temp_dir) = testutils::create_test_cache();
        add_link_fixtures(&mut cache)?;
        let results = cache.search("")?;
        assert!(!results.is_empty());
        Ok(())
    }

    #[test]
    fn test_search_fuzzy() -> Result<()> {
        let (mut cache, _temp_dir) = testutils::create_test_cache();
        add_link_fixtures(&mut cache)?;
        
        // Add a link that will definitely match our fuzzy search
        cache.add(Link {
            title: "Visual Studio Code Editor".to_string(),
            url: "https://code.visualstudio.com/editor".to_string(),
            ..Default::default()
        })?;
        
        let results = cache.search("Vis studio")?;
        assert!(!results.is_empty(), "Should find results with fuzzy search");
        assert!(results[0].title.contains("Visual Studio"), "First result should contain 'Visual Studio'");
        Ok(())
    }
}
