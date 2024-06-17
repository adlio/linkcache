use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Link {
    // Required fields
    pub url: String,
    pub title: String,
    /// Timestamp as a number of seconds since the epoch
    /// assumed to be UTC, but we assume consumers will
    /// handle time zone conversion.
    pub timestamp: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// Customization of the short and long title options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_title: Option<String>,
}

impl Link {
    pub fn new(url: String, title: String) -> Link {
        let timestamp = chrono::Utc::now().timestamp();
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

    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }
}
