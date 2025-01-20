use chrono::DateTime;
use filetime::FileTime;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::cache::Cache;
use crate::error::Result;
use crate::link::Link;

use ini::Ini;

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

    pub fn cache_bookmarks(&self, cache: &mut Cache) -> Result<()> {
        let links = self.bookmark_links()?;
        for link in links {
            cache.add(link)?;
        }
        Ok(())
    }

    pub fn bookmark_links(&self) -> Result<Vec<Link>> {
        let mut links = vec![];
        let file = File::open(self.bookmarks_path())?;
        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader)?;

        fn traverse(node: &Value, links: &mut Vec<Link>) {
            if let Some(obj) = node.as_object() {
                // Firefox bookmarks have different JSON structure than Chrome
                if obj.contains_key("type") && obj["type"] == "bookmark" {
                    if let (Some(title), Some(uri)) = (
                        obj.get("title").and_then(Value::as_str),
                        obj.get("uri").and_then(Value::as_str),
                    ) {
                        let date_added =
                            obj.get("dateAdded").and_then(Value::as_i64).unwrap_or(0) / 1000; // Convert from milliseconds to seconds

                        links.push(Link {
                            title: title.to_string(),
                            url: uri.to_string(),
                            subtitle: None, // Firefox doesn't have folder paths like Chrome
                            timestamp: DateTime::from_timestamp(date_added, 0)
                                .expect("Failed to convert timestamp"),
                            ..Default::default()
                        });
                    }
                }

                // Recursively process children
                if let Some(children) = obj.get("children").and_then(Value::as_array) {
                    for child in children {
                        traverse(child, links);
                    }
                }
            }
        }

        if let Some(children) = json.get("children").and_then(Value::as_array) {
            for child in children {
                traverse(child, &mut links);
            }
        }

        Ok(links)
    }

    fn bookmarks_path(&self) -> PathBuf {
        // Firefox stores bookmarks in places.sqlite, but also maintains a JSON backup
        self.profile_dir
            .join("bookmarkbackups")
            .join("bookmark-backup.json")
    }

    /*
    pub fn search_bookmarks_directly(&self, query: &str) -> Result<Vec<Link>> {
        fn get_fuzzy_score(query: &str, title: &str) -> Option<isize> {
            let query = query.to_lowercase();
            let title = title.to_lowercase();

            if title.contains(&query) {
                // Exact match is best
                Some(0)
            } else {
                // Try character-by-character matching
                let mut score = 0isize;
                let mut title_chars = title.chars().peekable();

                for query_char in query.chars() {
                    loop {
                        match title_chars.next() {
                            Some(title_char) => {
                                if query_char == title_char {
                                    break;
                                }
                                score -= 1;
                            }
                            None => return None,
                        }
                    }
                }

                Some(score)
            }
        }

        let mut links = self.bookmark_links()?;

        // Filter and sort by fuzzy match score
        links.retain_mut(|link| {
            if let Some(score) = get_fuzzy_score(query, &link.title) {
                link.score = Some(score); // Fix: score is an Option<isize>
                true
            } else {
                false
            }
        });

        links.sort_by_key(|link| link.score);
        Ok(links)
    }
    */

    /// Creates a backup of the Firefox places SQLite database. This is
    /// necessary because the browser itself has a read lock on the SQLite
    /// database, preventing us from opening a connection to it.
    ///
    pub(crate) fn create_places_replica(&self) -> Result<()> {
        let source = self.places_path();
        let dest = self.places_replica_path();
        std::fs::copy(source, dest)?;

        // Manually set the modification time of the new file to now
        filetime::set_file_times(self.places_replica_path(), FileTime::now(), FileTime::now())?;
        Ok(())
    }

    pub(crate) fn places_path(&self) -> PathBuf {
        self.profile_dir.join("places.sqlite")
    }

    pub(crate) fn places_replica_path(&self) -> PathBuf {
        self.places_path().with_file_name("places.linkcache.sqlite")
    }

    pub fn default_config_dir() -> Result<PathBuf> {
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

    /// Returns the default Firefox profile directory for the current user.
    ///
    pub fn default_profile_dir() -> Result<PathBuf> {
        let profile_dir = Self::find_default_release_dir()?;
        Ok(profile_dir)
    }

    pub fn find_default_release_dir() -> Result<PathBuf> {
        let config_dir = Self::default_config_dir()?;

        let conf = Ini::load_from_file(config_dir.join("profiles.ini"))?;
        for section in conf.sections().flatten() {
            if section.starts_with("Install") {
                if let Some(default_path) = conf.get_from(Some(section), "Default") {
                    let profile_path = config_dir.join(default_path);
                    println!("{:?}", profile_path);
                    return Ok(profile_path);
                }
            }
        }

        Ok("/tmp".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_places_replica() {
        let browser = Browser::new().expect("Failed to create browser");
        let res = browser.create_places_replica();
        assert!(res.is_ok());
    }

    #[test]
    fn test_find_default_release_dir() {
        let path = Browser::find_default_release_dir().expect("Shouldn't fail");
        assert!(path.exists(), "Directory should exist")
    }

    #[test]
    #[ignore = "CI environments don't have a Firefox home directory"]
    fn test_default_profile_dir() {
        let default_dir = Browser::default_profile_dir().unwrap();
        println!("Default profile directory: {:?}", default_dir);
        assert!(default_dir.exists());
    }
}
