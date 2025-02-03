use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use super::sidebar::SidebarState;
use crate::error::Result;
use crate::Link;

pub struct Browser {
    profile_dir: PathBuf,
}

impl Browser {
    /// Default constructor which creates a new Arc Browser with the default path
    /// to the Arc profile directory.
    pub fn new() -> Self {
        Browser {
            profile_dir: Self::default_profile_dir(),
        }
    }

    /// Alternate constructor that allows the user to specify a custom path to
    /// the directory where the Arc profile (including the StorableSidebar.json
    /// file) is stored.
    pub fn with_profile_dir(mut self, dir: PathBuf) -> Self {
        self.profile_dir = dir;
        self
    }

    /// Sidebar links builds a Link object for each item in the Arc sidebar
    ///
    pub fn sidebar_links(&self) -> Result<Vec<Link>> {
        // Data values
        let mut state = self.sidebar_json()?;
        let bookmarks = state.bookmarks();

        let mut links: Vec<Link> = vec![];

        for bookmark in bookmarks {
            let title = bookmark.title().unwrap_or_default();
            let url = bookmark.data.tab.saved_url.unwrap_or_default();
            let mut link = Link::new(format!("arc-{}", url), url, title);
            if let Some(parent_id) = bookmark.parent_id {
                let ancestor_titles = state.ancestor_titles(&parent_id)?;
                if !ancestor_titles.is_empty() {
                    link = link.with_subtitle(ancestor_titles);
                }
            }
            links.push(link);
        }

        Ok(links)
    }

    fn sidebar_json(&self) -> Result<SidebarState> {
        let file = File::open(self.sidebar_path())?;
        let reader = BufReader::new(file);
        let state = serde_json::from_value::<SidebarState>(serde_json::from_reader(reader)?)?;
        Ok(state)
    }

    /// Returns the path on disk where the StorableSidebar.json file is stored.
    /// This file stores the state of the entire pinned site/bookmark sidebar
    /// in the Arc browser.
    ///
    fn sidebar_path(&self) -> PathBuf {
        self.profile_dir.join("StorableSidebar.json")
    }

    /// Returns the directory of the Default Arc profile directory based on the
    /// user's operating system and detected home directory.
    pub fn default_profile_dir() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let arc_data_dir = match std::env::consts::OS {
            "macos" => home_dir.join("Library/Application Support/Arc"),
            // TODO linux is untested
            "linux" => home_dir.join(".config/arc"),
            // TODO windows is untested
            "windows" => home_dir.join("AppData/Local/Arc"),
            _ => home_dir.join(".config/arc"),
        };
        arc_data_dir
    }
}

impl Default for Browser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_browser() -> Browser {
        Browser::new().with_profile_dir(PathBuf::from("./test_data"))
    }

    #[test]
    fn test_sidebar_links() -> Result<()> {
        let browser = test_browser();
        let links = browser.sidebar_links()?;

        for link in &links {
            println!("{}: {}", link.title, link.url);
        }

        // TODO This test is brittle and will break if the test data
        // changes. It would be better to test the structure of the
        // data rather than the specific values.
        assert_eq!(links.len(), 9);
        let script_filter_link = links.first().unwrap();
        assert_eq!(script_filter_link.title, "Script Filter JSON Format");
        assert_eq!(
            script_filter_link.url,
            "https://www.alfredapp.com/help/workflows/inputs/script-filter/json/"
        );
        assert_eq!(
            script_filter_link.subtitle,
            Some("Work / Areas / Alfred".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_storable_sidebar() -> Result<()> {
        let browser = Browser::new().with_profile_dir(PathBuf::from("./test_data"));
        let links = browser.sidebar_links()?;
        for link in links {
            println!("{}: {}", link.title, link.url);
        }
        Ok(())
    }
}
