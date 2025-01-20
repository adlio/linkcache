use chrono::DateTime;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::cache::Cache;
use crate::error::Result;
use crate::link::Link;

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

    fn places_path(&self) -> PathBuf {
        self.profile_dir.join("places.sqlite")
    }

    fn places_replica_path(&self) -> PathBuf {
        self.places_path().with_file_name("places.linkcache.sqlite")
    }


    /// Returns the default Firefox profile directory for the current user.
    ///
    pub fn default_profile_dir() -> Result<PathBuf> {
        let parent_dir = Self::default_profile_parent_dir()?;
        let profile_dir = Self::find_default_release_dir(parent_dir)?;
        Ok(profile_dir)
    }

    /// Given the top-level Firefox Profiles parent directory, this function finds the
    /// subdirectory which ends with .default-release, which is the convention Firefox
    /// uses to indicate the default/first-created profile.
    ///
    pub fn find_default_release_dir(parent_dir: PathBuf) -> Result<PathBuf> {
        let profile_dir = std::fs::read_dir(&parent_dir)?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".default-release")
            })
            .map(|entry| entry.path())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Could not find a path ending with .default-release under {:?}",
                        &parent_dir,
                    ),
                )
            })?;
        Ok(profile_dir)
    }

    /// Returns the OS-aware parent directory for Firefox profiles (i.e. the
    /// directory which contains the <randchars>.default-release directory
    /// which will be the current user's default Firefox profile.
    ///
    pub fn default_profile_parent_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine home directory",
            )
        })?;

        let os = std::env::consts::OS;
        let profile_parent_dir = match os {
            "macos" => home_dir.join("Library/Application Support/Firefox/Profiles"),
            "linux" => home_dir.join(".mozilla/firefox"),
            "windows" => home_dir.join("AppData/Roaming/Mozilla/Firefox/Profiles"),
            unsupported => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!("Unsupported operating system: {}", unsupported),
                )
                .into());
            }
        };
        Ok(profile_parent_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_default_release_dir() {
        let dir = Browser::find_default_release_dir(PathBuf::from("test_data/FirefoxProfileDir"))
            .expect("Should find the directory with the .default-release suffix");
        assert_eq!(
            "5abcyz0s.default-release",
            dir.file_name().expect("Directory should have a name"),
        );
        assert!(dir.exists());
    }

    #[test]
    #[ignore = "CI environments don't have a Firefox home directory"]
    fn test_default_profile_dir() {
        let default_dir = Browser::default_profile_dir().unwrap();
        println!("Default profile directory: {:?}", default_dir);
        assert!(default_dir.exists());
    }
}
