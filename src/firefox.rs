use filetime::FileTime;
use log::error;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::error::Result;

use crate::{Error, Link};
use ini::Ini;

/// Browser represents a particular instance of a Firefox profile for a specific
/// user. At its core, this is a wrapper around the profile directory that stores
/// all the profile preference files ( the directory is usually randomly named ).
///
/// The default new() constructor will attempt to determine the currently
/// logged-in user's default Firefox profile and instantiate with that. This
/// default can be overridden via the with_profile_dir() constructor instead.
///
pub struct Browser {
    profile_dir: PathBuf,
}

impl Browser {
    pub fn new() -> Result<Self> {
        Ok(Browser {
            profile_dir: Self::default_profile_dir()?,
        })
    }

    pub fn with_profile_dir(mut self, dir: PathBuf) -> Self {
        self.profile_dir = dir;
        self
    }

    /// Searches the places.sqlite database (actually a replica of it that we manage)
    /// for all bookmarks that loosely match the provided string.
    ///
    pub fn search_bookmarks_directly(&self, query: impl ToString) -> Result<Vec<Link>> {
        self.create_places_replica()?;

        let path = self.places_replica_path();
        match Connection::open(path) {
            Err(err) => Err(err.into()),
            Ok(conn) => {
                let mut stmt = conn.prepare(
                    "SELECT p.url, b.title, 1 as rank
                    FROM moz_bookmarks b 
                    JOIN moz_places p ON b.fk = p.id 
                    WHERE b.type = 1 AND (p.url LIKE ?1 OR b.title LIKE ?1)
                    UNION ALL
                    SELECT p.url, p.title, 2 AS rank
                    FROM moz_places p
                    WHERE (p.url LIKE ?1 OR p.title LIKE ?1)
                    ORDER BY rank, p.url, p.title
                ",
                )?;
                let query: String = query.to_string();
                error!("Executing query with parameter: {}", query); // Debug statement
                let mut seen_urls = HashSet::new();
                let links: Vec<Link> = stmt
                    .query_map(params![format!("%{}%", query)], |row| {
                        let url: String = row.get(0)?;
                        let title: String = row.get(1)?;
                        error!("Found row: url={}, title={}", url, title); // Debug statement
                        if seen_urls.insert(url.clone()) {
                            Ok(Some(Link {
                                url,
                                title,
                                ..Default::default()
                            }))
                        } else {
                            Ok(None)
                        }
                    })?
                    .filter_map(|link| link.ok().flatten())
                    .collect();
                error!("Total links found: {}", links.len()); // Debug statement
                Ok(links)
            }
        }
    }

    /// Creates a backup of the Firefox places SQLite database. This is
    /// necessary because the browser itself has a read lock on the SQLite
    /// database, preventing us from opening a connection to it. This function
    /// replaces any existing replica file regardless of its age.
    ///
    pub(crate) fn create_places_replica(&self) -> Result<()> {
        let source = self.places_path();
        let dest = self.places_replica_path();
        std::fs::copy(source, dest)?;

        // Manually set the modification time of the new file to now
        filetime::set_file_times(self.places_replica_path(), FileTime::now(), FileTime::now())?;
        Ok(())
    }

    /// Returns the full path to the places.sqlite database
    ///
    pub(crate) fn places_path(&self) -> PathBuf {
        self.profile_dir.join("places.sqlite")
    }

    /// Returns the full path to the places.sqlite replica. Unlike places_path,
    /// we should actually be allowed to run SQL queries on this database.
    ///
    pub(crate) fn places_replica_path(&self) -> PathBuf {
        self.places_path().with_file_name("places.linkcache.sqlite")
    }

    /// Returns the default Firefox profile directory for the current user.
    ///
    pub fn default_profile_dir() -> Result<PathBuf> {
        let config_dir = Self::default_firefox_profiles_dir()?;

        // Find the first Install* section that contains a Default key
        // inside it. The default profile path will the profile_dir joined
        // with the value of the Default key.
        let conf = Ini::load_from_file(config_dir.join("profiles.ini"))?;
        for section in conf.sections().flatten() {
            if section.starts_with("Install") {
                if let Some(default_path) = conf.get_from(Some(section), "Default") {
                    let profile_path = config_dir.join(default_path);
                    return Ok(profile_path);
                }
            }
        }

        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find default profile directory.",
        )))
    }

    /// Attempts to identify the location of the top-level container for all
    /// Firefox profiles. On Mac, its ~/Library/Application Support/Firefox.
    /// On Linux, it's ~/.mozilla/firefox.
    ///
    /// Will error if the user's home directory could not be determined, or
    /// if the expected Firefox directory for the OS doesn't exist.
    ///
    pub fn default_firefox_profiles_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine home directory.",
            )
        })?;

        let os = std::env::consts::OS;
        let config_dir = match os {
            "macos" => home_dir.join("Library/Application Support/Firefox"),
            "linux" => home_dir.join(".mozilla/firefox"),
            "windows" => home_dir.join("AppData/Roaming/Mozilla/Firefox"),
            unsupported => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!("Unsupported operating system: {}", unsupported),
                )
                .into());
            }
        };

        Ok(config_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_bookmarks_directly() {
        let browser = Browser::new().expect("Failed to create browser");
        let res = browser.search_bookmarks_directly("Wiki");
        assert!(res.is_ok());
        let links = res.unwrap();
        assert!(!links.is_empty());
        for link in links {
            println!("{}: {}", link.title, link.url);
        }
    }

    #[test]
    fn test_create_places_replica() {
        let browser = Browser::new().expect("Failed to create browser");
        let res = browser.create_places_replica();
        assert!(res.is_ok());
    }

    #[test]
    fn test_find_default_release_dir() {
        let path = Browser::default_profile_dir().expect("Shouldn't fail");
        assert!(path.exists(), "Directory should exist")
    }

    #[test]
    #[ignore = "CI environments don't have a Firefox home directory"]
    fn test_default_profile_dir() {
        let default_dir = Browser::default_profile_dir().unwrap();
        assert!(default_dir.exists());
    }
}
