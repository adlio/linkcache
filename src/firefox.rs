use filetime::FileTime;
use ini::Ini;
use log::error;
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::error::Result;

use crate::{Cache, Error, Link};

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

    pub fn cache_history(&self, cache: &mut Cache) -> Result<()> {
        for link in self.all_history(cache)? {
            cache.add(link)?;
        }
        Ok(())
    }

    pub fn cache_bookmarks(&self, cache: &mut Cache) -> Result<()> {
        let links = self.all_bookmarks(cache)?;
        for link in links {
            cache.add(link)?;
        }
        Ok(())
    }

    /// Extracts all Bookmarks from the Firefox Browser as Link objects. We require
    /// a non-mutable Cache because Firefox holds a read lock on the places.sqlite
    /// database, so we copy the file into the data_dir so that we can query from it.
    ///
    pub fn all_bookmarks(&self, cache: &Cache) -> Result<Vec<Link>> {
        let path = self.places_replica_path(cache);
        match Connection::open(path) {
            Ok(conn) => {
                let mut stmt = conn.prepare(include_str!("./queries/all_firefox_bookmarks.sql"))?;
                // TODO Don't repeat this
                let links: Vec<Link> = stmt
                    .query_map(params![], |row| {
                        let guid: String = row.get(0)?;
                        let url: String = row.get(1)?;
                        let title: String = row.get(2)?;
                        let subtitle: String = row.get(3)?;
                        let link = Link::new(guid, url, title).with_subtitle(subtitle);
                        Ok(Some(link))
                    })?
                    .filter_map(|link| link.ok().flatten())
                    .collect();
                error!("Total links found: {}", links.len()); // Debug statement
                Ok(links)
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Fetches an in-memory copy of the ENTIRE Firefox history information.
    /// TODO Use batched iteration over history data.
    pub fn all_history(&self, cache: &Cache) -> Result<Vec<Link>> {
        let path = self.places_replica_path(cache);
        match Connection::open(path) {
            Ok(conn) => {
                let mut stmt = conn.prepare(include_str!("./queries/all_firefox_history.sql"))?;
                // TODO Don't repeat this
                let links: Vec<Link> = stmt
                    .query_map(params![], |row| {
                        let guid: String = row.get(0)?;
                        let url: String = row.get(1)?;
                        let title: String = row.get(2)?;
                        let link = Link::new(guid, url, title);
                        Ok(Some(link))
                    })?
                    .filter_map(|link| link.ok().flatten())
                    .collect();
                error!("Total history links found: {}", links.len()); // Debug statement
                Ok(links)
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Creates a backup of the Firefox places SQLite database. This is
    /// necessary because the browser itself has a read lock on the SQLite
    /// database, preventing us from opening a connection to it. This function
    /// replaces any existing replica file regardless of its age.
    ///
    pub fn create_places_replica(&self, cache: &Cache) -> Result<()> {
        let source = self.places_path();
        let dest = self.places_replica_path(cache);
        std::fs::copy(source, &dest)?;

        // Manually set the modification time of the new file to now
        filetime::set_file_times(dest, FileTime::now(), FileTime::now())?;
        Ok(())
    }

    /// Returns the full path to the places.sqlite database
    ///
    pub fn places_path(&self) -> PathBuf {
        self.profile_dir.join("places.sqlite")
    }

    /// Returns the full path to the location of the places.sqlite replica file inside our cache.
    ///
    pub fn places_replica_path(&self, cache: &Cache) -> PathBuf {
        cache.data_dir.join("firefox-places.sqlite")
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
        // For testing purposes, check if the TEST_FIREFOX_PROFILE_DIR environment variable is set
        if let Ok(test_dir) = std::env::var("TEST_FIREFOX_PROFILE_DIR") {
            return Ok(PathBuf::from(test_dir));
        }

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
    use crate::testutils::create_test_cache;
    use std::env;

    /// Helper function to get the path to our test Firefox profile
    fn test_firefox_profile_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/firefox_profile/test.default");
        path
    }

    /// Helper function to get the path to our test Firefox profiles directory
    fn test_firefox_profiles_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/firefox_profile");
        path
    }

    #[test]
    fn test_create_places_replica() {
        let (cache, _tmpdir) = create_test_cache();
        let browser = Browser::new()
            .expect("Failed to create browser")
            .with_profile_dir(test_firefox_profile_dir());
        let res = browser.create_places_replica(&cache);
        assert!(res.is_ok());
        
        // Verify the replica file exists
        assert!(browser.places_replica_path(&cache).exists(), "Replica file should exist");
    }

    #[test]
    fn test_find_default_release_dir() {
        // Set the test environment variable
        env::set_var("TEST_FIREFOX_PROFILE_DIR", test_firefox_profiles_dir().to_str().unwrap());
        
        let path = Browser::default_firefox_profiles_dir().expect("Shouldn't fail");
        assert_eq!(path, test_firefox_profiles_dir(), "Should use test directory");
        
        // Clean up
        env::remove_var("TEST_FIREFOX_PROFILE_DIR");
    }

    #[test]
    fn test_default_profile_dir_with_test_data() {
        // Set the test environment variable
        env::set_var("TEST_FIREFOX_PROFILE_DIR", test_firefox_profiles_dir().to_str().unwrap());
        
        let default_dir = Browser::default_profile_dir().unwrap();
        assert_eq!(default_dir, test_firefox_profile_dir(), "Should use test profile directory");
        
        // Clean up
        env::remove_var("TEST_FIREFOX_PROFILE_DIR");
    }
    
    #[test]
    fn test_with_profile_dir() {
        let profile_dir = test_firefox_profile_dir();
        let browser = Browser::new()
            .expect("Failed to create browser")
            .with_profile_dir(profile_dir.clone());
        
        assert_eq!(browser.profile_dir, profile_dir, "Profile directory should be set correctly");
    }
    
    #[test]
    fn test_places_path() {
        let profile_dir = test_firefox_profile_dir();
        let browser = Browser::new()
            .expect("Failed to create browser")
            .with_profile_dir(profile_dir.clone());
        
        let expected_path = profile_dir.join("places.sqlite");
        assert_eq!(browser.places_path(), expected_path, "Places path should be correct");
    }
    
    #[test]
    fn test_places_replica_path() {
        let (cache, _tmpdir) = create_test_cache();
        let browser = Browser::new().expect("Failed to create browser");
        
        let expected_path = cache.data_dir.join("firefox-places.sqlite");
        assert_eq!(browser.places_replica_path(&cache), expected_path, "Replica path should be correct");
    }
    
    #[test]
    fn test_all_bookmarks() {
        let (cache, _tmpdir) = create_test_cache();
        let browser = Browser::new()
            .expect("Failed to create browser")
            .with_profile_dir(test_firefox_profile_dir());
        
        // Create a replica of the places database in the cache
        browser.create_places_replica(&cache).expect("Failed to create places replica");
        
        // Get all bookmarks
        let bookmarks = browser.all_bookmarks(&cache).expect("Failed to get all bookmarks");
        
        // We should have 3 bookmarks in our test data
        assert_eq!(bookmarks.len(), 3, "Should have 3 bookmarks");
        
        // Check that we have the expected bookmarks
        let titles: Vec<String> = bookmarks.iter().map(|b| b.title.clone()).collect();
        assert!(titles.contains(&"Mozilla".to_string()), "Should have Mozilla bookmark");
        assert!(titles.contains(&"Rust Programming Language".to_string()), "Should have Rust bookmark");
        assert!(titles.contains(&"GitHub".to_string()), "Should have GitHub bookmark");
    }
    
    #[test]
    fn test_all_history() {
        let (cache, _tmpdir) = create_test_cache();
        let browser = Browser::new()
            .expect("Failed to create browser")
            .with_profile_dir(test_firefox_profile_dir());
        
        // Create a replica of the places database in the cache
        browser.create_places_replica(&cache).expect("Failed to create places replica");
        
        // Get all history
        let history = browser.all_history(&cache).expect("Failed to get all history");
        
        // Our test data should have some history entries
        assert!(!history.is_empty(), "Should have history entries");
        
        // Check that we have the expected history entries
        let titles: Vec<&str> = history.iter().map(|h| h.title.as_str()).collect();
        assert!(titles.contains(&"Example Domain") || titles.contains(&"Wikipedia"), 
               "Should have at least one of the expected history entries");
    }
    
    #[test]
    fn test_cache_bookmarks() {
        let (mut cache, _tmpdir) = create_test_cache();
        let browser = Browser::new()
            .expect("Failed to create browser")
            .with_profile_dir(test_firefox_profile_dir());
        
        // Create a replica of the places database in the cache
        browser.create_places_replica(&cache).expect("Failed to create places replica");
        
        // Cache the bookmarks
        browser.cache_bookmarks(&mut cache).expect("Failed to cache bookmarks");
        
        // Search for a known bookmark
        let results = cache.search("Mozilla").expect("Search failed");
        assert!(!results.is_empty(), "Should find Mozilla bookmark");
        
        // Search for another known bookmark
        let results = cache.search("Rust").expect("Search failed");
        assert!(!results.is_empty(), "Should find Rust bookmark");
    }
    
    #[test]
    fn test_cache_history() {
        let (mut cache, _tmpdir) = create_test_cache();
        let browser = Browser::new()
            .expect("Failed to create browser")
            .with_profile_dir(test_firefox_profile_dir());
        
        // Create a replica of the places database in the cache
        browser.create_places_replica(&cache).expect("Failed to create places replica");
        
        // Cache the history
        browser.cache_history(&mut cache).expect("Failed to cache history");
        
        // Search for known history entries
        let results = cache.search("Example").expect("Search failed");
        assert!(!results.is_empty(), "Should find Example Domain history entry");
        
        let results = cache.search("Wikipedia").expect("Search failed");
        assert!(!results.is_empty(), "Should find Wikipedia history entry");
    }
}
