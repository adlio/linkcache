use crate::error::Result;
use crate::Link;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct Browser {
    profile_dir: PathBuf,
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarFolder {
    pub id: String,
    pub title: Option<String>,
    pub data: serde_json::Value,
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

impl Browser {
    pub fn new() -> Self {
        Browser {
            profile_dir: Self::default_profile_dir(),
        }
    }

    pub fn with_profile_dir(mut self, dir: PathBuf) -> Self {
        self.profile_dir = dir;
        self
    }

    pub fn sidebar_links(&self) -> Result<Vec<Link>> {
        let mut links: Vec<Link> = vec![];
        let file = File::open(self.storable_sidebar_path())?;
        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader)?;

        let mut space_titles: HashMap<String, String> = HashMap::new();

        if let Some(sidebar) = json.get("sidebar") {
            if let Some(containers) = sidebar.get("containers").and_then(Value::as_array) {
                for container in containers {
                    if let Some(spaces) = container.get("spaces").and_then(Value::as_array) {
                        // Store a lookup of each Space
                        for space in spaces {
                            if let Some(space_id) = space.get("id").and_then(Value::as_str) {
                                if let Some(space_title) =
                                    space.get("title").and_then(Value::as_str)
                                {
                                    space_titles
                                        .insert(space_id.to_string(), space_title.to_string());
                                }
                            }
                        }

                        if let Some(items) = container.get("items") {
                            match serde_json::from_value::<Vec<SidebarItemType>>(items.clone()) {
                                Ok(items) => {
                                    for item in items {
                                        match item {
                                            SidebarItemType::Bookmark(bookmark) => {
                                                if let Some(title) = bookmark.title {
                                                    if let Some(url) = bookmark.data.tab.saved_url {
                                                        links.push(Link {
                                                            title,
                                                            url,
                                                            ..Default::default()
                                                        });
                                                    }
                                                }
                                            }
                                            SidebarItemType::Folder(folder) => { /* No Op */ }
                                            SidebarItemType::Value(_id) => {
                                                // No-Op
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    // No op
                                    // println!("Error: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(links)
    }

    fn storable_sidebar_path(&self) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::*;

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
