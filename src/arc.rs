use crate::error::Result;
use crate::Link;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::num::ParseIntError;
use std::path::PathBuf;

pub struct Browser {
    profile_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarState {
    pub sidebar_sync_state: Value,
    pub version: i64,
    pub sidebar: Sidebar,
    pub firebase_sync_state: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sidebar {
    pub containers: Vec<SidebarContainer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SidebarContainer {
    SpacesAndItems(SidebarSpacesAndItemsContainer),
    Value(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarSpacesAndItemsContainer {
    #[serde(rename = "spaces")]
    pub spaces: Vec<SpaceType>,
    #[serde(rename = "topAppsContainerIDs")]
    pub top_apps_container_ids: Value,
    #[serde(rename = "items")]
    pub items: Vec<SidebarItemType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpaceType {
    Space(SidebarSpace),
    Value(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarSpace {
    pub id: String,
    pub title: Option<String>,
    pub custom_info: Value,
    #[serde(rename = "newContainerIDs")]
    pub new_container_ids: Value,
    pub profile: Value,
    #[serde(rename = "containerIDs")]
    pub container_ids: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SidebarItemType {
    Folder(SidebarFolder),
    Bookmark(SidebarBookmark),
    Value(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarBookmark {
    pub id: String,
    pub title: Option<String>,
    pub data: SidebarTabData,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarFolder {
    pub id: String,
    pub title: Option<String>,
    pub data: serde_json::Value,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarTabData {
    pub tab: Tab,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    #[serde(rename = "savedTitle")]
    pub saved_title: Option<String>,
    #[serde(rename = "savedURL")]
    pub saved_url: Option<String>,
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

    pub fn sidebar_links(&self) -> Result<Vec<Link>> {
        // Data values
        let state = self.sidebar_json()?;
        let folder_titles = state.folder_title_map();
        // let space_titles = state.space_title_map();
        let bookmarks = state.bookmarks();

        let mut links: Vec<Link> = vec![];

        for bookmark in bookmarks {
            let title = bookmark.title().unwrap_or_default();
            let url = bookmark.data.tab.saved_url.unwrap_or_default();
            let mut subtitle = String::new();
            let mut link = Link::new(url, title);
            if let Some(parent_id) = bookmark.parent_id {
                if let Some(folder_title) = folder_titles.get(&parent_id) {
                    subtitle = folder_title.clone();
                }
                link = link.with_subtitle(subtitle);
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

impl SidebarState {
    /// Returns a map to lookup the names of each space in the entire SidebarState
    /// by their IDs.
    ///
    pub fn space_title_map(&self) -> HashMap<String, String> {
        let mut space_titles: HashMap<String, String> = HashMap::new();
        for container in &self.sidebar.containers {
            match container {
                SidebarContainer::SpacesAndItems(spaces_and_items) => {
                    for space in &spaces_and_items.spaces {
                        match space {
                            SpaceType::Space(sidebar_space) => {
                                space_titles.insert(
                                    sidebar_space.id.clone(),
                                    sidebar_space.title.clone().unwrap_or_default(),
                                );
                            }
                            SpaceType::Value(_) => {}
                        }
                    }
                }
                _ => {}
            }
        }
        space_titles
    }

    /// Returns a map to lookup the names of each folder in the entire
    /// SidebarState by their IDs.
    ///
    pub fn folder_title_map(&self) -> HashMap<String, String> {
        let mut folder_titles: HashMap<String, String> = HashMap::new();
        for container in &self.sidebar.containers {
            match container {
                SidebarContainer::SpacesAndItems(spaces_and_items) => {
                    for item in &spaces_and_items.items {
                        match item {
                            SidebarItemType::Folder(folder) => {
                                folder_titles.insert(
                                    folder.id.clone(),
                                    folder.title.clone().unwrap_or_default(),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        folder_titles
    }

    /// Returns a list of all bookmarks in the entire SidebarState
    pub fn bookmarks(&self) -> Vec<SidebarBookmark> {
        let mut bookmarks: Vec<SidebarBookmark> = vec![];
        for container in &self.sidebar.containers {
            match container {
                SidebarContainer::SpacesAndItems(spaces_and_items) => {
                    for item in &spaces_and_items.items {
                        match item {
                            SidebarItemType::Bookmark(bookmark) => {
                                bookmarks.push(bookmark.clone());
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        bookmarks
    }
}

impl SidebarBookmark {
    /// Returns the title of the bookmark, preferring the human-set title
    /// and falling back to the title saved from the page. Will return an
    /// empty
    pub fn title(&self) -> Option<String> {
        match self.title {
            Some(ref title) => {
                if title.is_empty() {
                    self.data.tab.saved_title.clone()
                } else {
                    Some(title.clone())
                }
            }
            None => self.data.tab.saved_title.clone(),
        }
    }
}

/// Deserialize SidebarItemType from JSON
/// This is a custom deserializer designed to disambiguate between the
/// three kinds of tiems in the items array in Arc's StorableSidebar.json
/// file. The three kinds of items are:
/// - Folder (a folder which may contain other folders or bookmarks)
/// - Bookmark (an individual bookmark link)
/// - Value (typically String identifiers, but this is a catch-all)
///
impl<'de> Deserialize<'de> for SidebarItemType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if value.get("data").and_then(|d| d.get("list")).is_some() {
            if let Ok(folder) = serde_json::from_value::<SidebarFolder>(value.clone()) {
                return Ok(SidebarItemType::Folder(folder));
            }
        }
        if value.get("data").and_then(|d| d.get("tab")).is_some() {
            if let Ok(bookmark) = serde_json::from_value::<SidebarBookmark>(value.clone()) {
                return Ok(SidebarItemType::Bookmark(bookmark));
            }
        }
        Ok(SidebarItemType::Value(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_title_map() -> Result<()> {
        let browser = Browser::new().with_profile_dir(PathBuf::from("./test_data"));
        let state = browser.sidebar_json()?;
        let space_titles = state.space_title_map();
        for (id, title) in space_titles {
            println!("{}: {}", id, title);
        }
        Ok(())
    }

    #[test]
    fn test_folder_title_map() -> Result<()> {
        let browser = Browser::new().with_profile_dir(PathBuf::from("./test_data"));
        let state = browser.sidebar_json()?;
        let space_titles = state.folder_title_map();
        for (id, title) in space_titles {
            println!("{}: {}", id, title);
        }
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

    #[test]
    fn test_bookmark_title_fallback() {
        let bookmark = SidebarBookmark {
            id: "123".to_string(),
            title: None,
            data: SidebarTabData {
                tab: Tab {
                    saved_title: Some("Saved Title".to_string()),
                    saved_url: None,
                },
            },
        };
        assert_eq!(bookmark.title(), Some("Saved Title".to_string()));
    }

    #[test]
    fn test_bookmark_title_some() {
        let bookmark = SidebarBookmark {
            id: "123".to_string(),
            title: Some("Human Title".to_string()),
            data: SidebarTabData {
                tab: Tab {
                    saved_title: Some("Saved Title".to_string()),
                    saved_url: None,
                },
            },
        };
        assert_eq!(bookmark.title(), Some("Human Title".to_string()));
    }
}
