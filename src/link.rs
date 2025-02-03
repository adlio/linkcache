use chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Link represents the unique combination of a URL and Title.
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Link {
    /// Globally unique identifier for the Link. Firefox
    /// has native GUIDs for all its links. For Chrome
    /// and Markdown sources we'll need to identify a
    /// deterministic mechanism to create a GUID for
    /// each link.
    pub guid: String,

    /// The fully-qualified URL for this link
    pub url: String,

    /// The name displayed when linking to this URL.
    /// There can be more than one title for the same URL.
    pub title: String,

    /// Optional description of the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// Unique identifier for the place from where
    /// this link was discovered. Will be "firefox"
    /// or "chrome," or similar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    pub timestamp: DateTime<chrono::Utc>,

    /// The relevancy score for this link in fulltext
    /// search results. This value isn't persisted in
    /// the database, and it will be None for Link
    /// structs being inserted to the database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

impl Link {
    pub fn new(guid: String, url: String, title: String) -> Link {
        let timestamp = chrono::Utc::now();
        Link {
            guid,
            url,
            title,
            timestamp,
            ..Default::default()
        }
    }

    pub fn with_subtitle(mut self, subtitle: String) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    pub fn with_timestamp_seconds(mut self, timestamp_seconds: i64) -> Self {
        let timestamp = DateTime::from_timestamp(timestamp_seconds, 0);
        self.timestamp = timestamp.expect("Failed to create timestamp");
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_new() {
        let link = Link::new(
            "test1".to_string(),
            "https://example.com".to_string(),
            "Example".to_string(),
        );
        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, "Example");
    }

    #[test]
    fn test_link_with_subtitle() {
        let link = Link::new(
            "test2".to_string(),
            "https://www.subtitle.com".to_string(),
            "Example with Subtitle".to_string(),
        )
        .with_subtitle("Subtitle".to_string());
        assert_eq!(link.url, "https://www.subtitle.com");
        assert_eq!(link.title, "Example with Subtitle");
        assert_eq!(link.subtitle, Some("Subtitle".to_string()));
    }

    #[test]
    fn test_link_with_timestamp_seconds() {
        // Get current timestamp
        let timestamp = chrono::Utc::now().timestamp();

        let link = Link::new(
            "test3".to_string(),
            "https://a.com".to_string(),
            "Example with Timestamp".to_string(),
        )
        .with_timestamp_seconds(timestamp);
        assert_eq!(link.url, "https://a.com");
        assert_eq!(link.title, "Example with Timestamp");
        assert_eq!(link.timestamp.timestamp(), timestamp);
    }
}
