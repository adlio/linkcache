use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Link represents the unique combination of a URL and Title.
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Link {
    pub url: String,

    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    pub timestamp: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

impl Link {
    pub fn new(url: String, title: String) -> Link {
        let timestamp = chrono::Utc::now();
        Link {
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

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_new() {
        let link = Link::new("https://example.com".to_string(), "Example".to_string());
        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, "Example");
    }

    #[test]
    fn test_link_with_subtitle() {
        let link = Link::new(
            "https://www.subtitle.com".to_string(),
            "Example with Subtitle".to_string(),
        )
        .with_subtitle("Subtitle".to_string());
        assert_eq!(link.url, "https://www.subtitle.com");
        assert_eq!(link.title, "Example with Subtitle");
        assert_eq!(link.subtitle, Some("Subtitle".to_string()));
    }

    #[test]
    fn test_link_with_author() {
        let link = Link::new(
            "https://www.author.com".to_string(),
            "Example with Author".to_string(),
        )
        .with_author("Author".to_string());
        assert_eq!(link.url, "https://www.author.com");
        assert_eq!(link.title, "Example with Author");
        assert_eq!(link.author, Some("Author".to_string()));
    }

    #[test]
    fn test_link_with_timestamp_seconds() {
        // Get current timestamp
        let timestamp = chrono::Utc::now().timestamp();

        let link = Link::new(
            "https://a.com".to_string(),
            "Example with Timestamp".to_string(),
        )
        .with_timestamp_seconds(timestamp);
        assert_eq!(link.url, "https://a.com");
        assert_eq!(link.title, "Example with Timestamp");
        assert_eq!(link.timestamp.timestamp(), timestamp);
    }
}
