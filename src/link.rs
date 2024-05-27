use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Link {
    // Required fields
    pub url: String,
    pub title: String,

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

    // Chrome History features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visit_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typed_count: Option<usize>,
}

impl Link {
    pub fn new(url: String, title: String) -> Link {
        Link {
            url,
            title,
            ..Default::default()
        }
    }
}
