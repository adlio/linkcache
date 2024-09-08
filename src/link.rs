use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
