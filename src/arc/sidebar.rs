use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Space(Space),
    Folder(Folder),
    Bookmark(Bookmark),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SidebarState {
    pub sidebar_sync_state: Value,
    pub version: i64,
    pub sidebar: Sidebar,
    pub firebase_sync_state: Value,

    #[serde(default)]
    pub item_map: HashMap<String, Node>,

    space_title_map: Option<HashMap<String, String>>,
    folder_title_map: Option<HashMap<String, String>>,
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
    Space(Space),
    Value(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Space {
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
    Folder(Folder),
    Bookmark(Bookmark),
    Value(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: Option<String>,
    pub data: SidebarTabData,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub title: Option<String>,
    pub data: serde_json::Value,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
    #[serde(rename = "childrenIds")]
    pub children_ids: Vec<String>,
    #[serde(rename = "isUnread")]
    pub is_unread: Option<bool>,
    #[serde(rename = "originatingDevice")]
    pub originating_device: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<f64>,
}

impl Folder {
    pub fn parent_id(&self) -> Option<String> {
        if let Some(parent_id) = &self.parent_id {
            Some(parent_id.clone())
        } else {
            if let Some(parent_id) = self
                .data
                .pointer("/itemContainer/containerType/spaceItems/_0")
            {
                Some(parent_id.as_str().unwrap().to_string())
            } else {
                None
            }
        }
    }
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

impl SidebarState {
    pub fn ancestor_titles(&mut self, id: &str) -> Result<String> {
        self.build_item_map()?;

        let mut titles: Vec<String> = vec![];
        let mut current_id = id.to_string();
        while let Some(node) = self.item_map.get(current_id.as_str()) {
            match node {
                Node::Folder(folder) => {
                    let title = folder.title.clone().unwrap_or_default();
                    if !title.is_empty() {
                        titles.insert(0, title);
                    }
                    match folder.parent_id().clone() {
                        Some(pid) => {
                            current_id = pid.clone();
                        }
                        None => {
                            break;
                        }
                    }
                }
                Node::Space(space) => {
                    let title = space.title.clone().unwrap_or_default();
                    if !title.is_empty() {
                        titles.insert(0, title);
                    }
                    break;
                }
                Node::Bookmark(bookmark) => {
                    current_id = bookmark.parent_id.clone().unwrap_or_default();
                }
            }
        }
        Ok(titles.join(" / "))
    }

    pub fn build_item_map(&mut self) -> Result<()> {
        if !self.item_map.is_empty() {
            return Ok(());
        }
        for container in &self.sidebar.containers {
            match container {
                SidebarContainer::SpacesAndItems(spaces_and_items) => {
                    for space in &spaces_and_items.spaces {
                        match space {
                            SpaceType::Space(sidebar_space) => {
                                self.item_map.insert(
                                    sidebar_space.id.clone(),
                                    Node::Space(sidebar_space.clone()),
                                );
                            }
                            SpaceType::Value(_) => {}
                        }
                    }
                    for item in &spaces_and_items.items {
                        match item {
                            SidebarItemType::Folder(folder) => {
                                self.item_map
                                    .insert(folder.id.clone(), Node::Folder(folder.clone()));
                            }
                            SidebarItemType::Bookmark(bookmark) => {
                                self.item_map
                                    .insert(bookmark.id.clone(), Node::Bookmark(bookmark.clone()));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Returns a list of all bookmarks in the entire SidebarState
    pub fn bookmarks(&self) -> Vec<Bookmark> {
        let mut bookmarks: Vec<Bookmark> = vec![];

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

impl Bookmark {
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
        if value.pointer("/data/list").is_some() || value.pointer("/data/itemContainer").is_some() {
            if let Ok(folder) = serde_json::from_value::<Folder>(value.clone()) {
                return Ok(SidebarItemType::Folder(folder));
            }
        }
        if value.pointer("/data/tab").is_some() {
            if let Ok(bookmark) = serde_json::from_value::<Bookmark>(value.clone()) {
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
    fn test_bookmark_title_fallback() {
        let bookmark = Bookmark {
            id: "123".to_string(),
            title: None,
            data: SidebarTabData {
                tab: Tab {
                    saved_title: Some("Saved Title".to_string()),
                    saved_url: None,
                },
            },
            parent_id: None,
        };
        assert_eq!(bookmark.title(), Some("Saved Title".to_string()));
    }

    #[test]
    fn test_bookmark_title_some() {
        let bookmark = Bookmark {
            id: "123".to_string(),
            title: Some("Human Title".to_string()),
            data: SidebarTabData {
                tab: Tab {
                    saved_title: Some("Saved Title".to_string()),
                    saved_url: None,
                },
            },
            parent_id: None,
        };
        assert_eq!(bookmark.title(), Some("Human Title".to_string()));
    }
}
