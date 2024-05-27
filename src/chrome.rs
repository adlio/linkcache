use crate::error::Result;
use crate::{Cache, Link};

use filetime::FileTime;
use itertools::Itertools;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use sublime_fuzzy::best_match;

pub struct Browser {
    profile_dir: PathBuf,
}

impl Browser {
    /// Default constructor for a Browser. Uses the default Chrome profile
    /// from the current user's home directory as the profile directory.
    pub fn new() -> Result<Self> {
        Ok(Browser {
            profile_dir: Self::default_profile_dir()?,
        })
    }

    /// Constructor that overrides the path to the Chrome profile to be
    /// in a different location.
    pub fn with_profile_dir(mut self, dir: PathBuf) -> Self {
        self.profile_dir = dir;
        self
    }

    /// Adds every bookmark from this browser to the provided Cache.
    ///
    pub fn cache_bookmarks(&self, cache: &mut Cache) -> Result<()> {
        let links = self.bookmark_links()?;
        for link in links {
            cache.add(link)?;
        }
        cache.commit()?;
        Ok(())
    }

    /// Adds every record in the History form this browser to the provided
    /// Cache.
    pub fn cache_history(&self, cache: &mut Cache) -> Result<()> {
        self.create_history_replica()?;
        let links = self.history_links()?;
        for link in links {
            cache.add(link)?;
        }
        cache.commit()?;
        Ok(())
    }

    /// TODO Possibly Remove? This function provides an alternative mechanism
    /// to scanning the file and adding all bookmarks to the index and instead
    /// just searches them directly using the sublime_fuzzy algorithm.
    ///
    pub fn search_bookmarks_directly(&self, query: &str) -> Result<Vec<Link>> {
        fn get_fuzzy_score(query: &str, title: &str) -> Option<isize> {
            let score = best_match(query, title).map(|m| m.score()).unwrap_or(0);
            if score > 0 {
                Some(score)
            } else {
                None
            }
        }

        let links: Vec<Link> = self
            .bookmark_links()?
            .into_iter()
            .filter_map(|link| {
                match get_fuzzy_score(
                    query,
                    format!(
                        "{} {}",
                        link.title,
                        link.subtitle.clone().unwrap_or_default()
                    )
                    .as_str(),
                ) {
                    Some(score) if score > 0 => Some((score, link)),
                    _ => None,
                }
            })
            .sorted_by(|a, b| b.0.cmp(&a.0))
            .map(|(_, link)| link)
            .collect();

        Ok(links)
    }

    /// TODO Possibly remove?
    pub fn search_history_directly(&self, query: &str) -> Result<Vec<Link>> {
        self.create_history_replica()?;
        let path = self.history_replica_path();
        match Connection::open(path) {
            Err(err) => Err(err.into()),
            Ok(conn) => {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, url, title,
                    last_visit_time,
                    visit_count, typed_count
                    FROM urls
                    WHERE title LIKE ?1 OR url LIKE ?1
                    ORDER BY
                    typed_count >= 1 DESC,
                    last_visit_time DESC,
                    visit_count DESC,
                    typed_count DESC
                    LIMIT 20
                    "#,
                )?;
                let links = stmt
                    .query_map(params![format!("%{}%", query)], |row| {
                        Ok(Link {
                            id: row.get(0)?,
                            url: row.get(1)?,
                            title: row.get(2)?,
                            visit_count: row.get(4)?,
                            typed_count: row.get(5)?,
                            short_title: None,
                            long_title: None,
                            subtitle: None,
                            score: Some(0 as f32),
                        })
                    })?
                    .filter_map(|link| link.ok())
                    .collect();
                Ok(links)
            }
        }
    }

    /// Parses the Bookmarks file (a JSON blob) in the browser profile
    /// directory and processes it recursively, returning each non-folder
    /// bookmark entry as a Link.
    ///
    pub fn bookmark_links(&self) -> Result<Vec<Link>> {
        let mut links = vec![];

        let file = File::open(self.bookmarks_path())?;
        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader)?;

        fn traverse(node: &Value, links: &mut Vec<Link>, subtitle: &str) {
            if let Some(my_title) = node.get("name").and_then(Value::as_str) {
                if let Some(url) = node.get("url").and_then(Value::as_str) {
                    links.push(Link {
                        title: my_title.to_string(),
                        url: url.to_string(),
                        subtitle: Some(subtitle.to_string()),
                        ..Default::default()
                    });
                }

                if let Some(children) = node.get("children").and_then(Value::as_array) {
                    for child in children {
                        traverse(
                            child,
                            links,
                            format!("{}/{}", &subtitle, &my_title).as_str(),
                        );
                    }
                }
            }
        }

        if let Some(roots) = json.get("roots").and_then(Value::as_object) {
            for (key, value) in roots {
                if key == "bookmark_bar" || key == "other" || key == "synced" {
                    traverse(value, &mut links, "");
                }
            }
        }

        Ok(links)
    }

    /// Scans the copy of the browser history file (this function assumes it
    /// already exists) and returns a Link struct for each entry in the
    /// database.
    ///
    pub fn history_links(&self) -> Result<Vec<Link>> {
        let path = self.history_replica_path();
        match Connection::open(path) {
            Err(err) => Err(err.into()),
            Ok(conn) => {
                let mut stmt = conn.prepare(
                    r#"
                        SELECT id, url, title,
                        last_visit_time,
                        visit_count, typed_count
                        FROM urls
                        ORDER BY last_visit_time ASC
                    "#,
                )?;
                let links: Vec<Link> = stmt
                    // Map the query to a result per row
                    .query_map(params![], |row| {
                        Ok(Link {
                            id: row.get(0)?,
                            url: row.get(1)?,
                            title: row.get(2)?,
                            visit_count: row.get(4)?,
                            typed_count: row.get(5)?,
                            ..Default::default()
                        })
                    })?
                    // Remove erroneous rows
                    .filter_map(|link| link.ok())
                    .collect();
                Ok(links)
            }
        }
    }

    /// Creates a backup of the Chrome browser's history file. This is
    /// necessary because the browser application has a read lock on
    /// the SQLite database preventing us from reading it.
    fn create_history_replica(&self) -> Result<()> {
        let source = self.history_path();
        let dest = self.history_replica_path();
        fs::copy(source, dest)?;

        // Manually set the modification time of the new file to now
        filetime::set_file_times(
            self.history_replica_path(),
            FileTime::now(),
            FileTime::now(),
        )?;
        Ok(())
    }

    fn bookmarks_path(&self) -> PathBuf {
        self.profile_dir.join("Bookmarks")
    }

    fn history_path(&self) -> PathBuf {
        self.profile_dir.join("History")
    }

    fn history_replica_path(&self) -> PathBuf {
        self.history_path().with_file_name("History.linkcache")
    }

    /// Returns the directory of the Default Chrome Profile based on the user's
    /// operating system and detected home directory.
    pub fn default_profile_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let chrome_data_dir = match std::env::consts::OS {
            "macos" => home_dir.join("Library/Application Support/Google/Chrome/Default"),
            "linux" => home_dir.join(".config/google-chrome/Default"),
            "windows" => home_dir.join("AppData/Local/Google/Chrome/User Data/Default"),
            _ => home_dir.join(".config/google-chrome/Default"),
        };
        Ok(chrome_data_dir)
    }
}
