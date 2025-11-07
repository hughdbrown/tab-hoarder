/// Data structures for Tab Hoarder
use serde::{Deserialize, Serialize};

/// Information about a browser tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub pinned: bool,
    pub index: i32,
}

impl TabInfo {
    pub fn new(id: i32, url: String, title: String, pinned: bool, index: i32) -> TabInfo {
        TabInfo {
            id,
            url,
            title,
            pinned,
            index,
        }
    }
}

/// A collapsed tab session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollapsedSession {
    pub id: String,
    pub name: String,
    pub timestamp: f64,
    pub tabs: Vec<SavedTab>,
}

/// A saved tab within a collapsed session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedTab {
    pub url: String,
    pub title: String,
    pub domain: String,
    pub pinned: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_info_creation() {
        let tab = TabInfo::new(
            1,
            "https://google.com".to_string(),
            "Google".to_string(),
            false,
            0,
        );

        assert_eq!(tab.id, 1);
        assert_eq!(tab.url, "https://google.com");
        assert_eq!(tab.title, "Google");
        assert_eq!(tab.pinned, false);
        assert_eq!(tab.index, 0);
    }

    #[test]
    fn test_serialization() {
        let session = CollapsedSession {
            id: "test-123".to_string(),
            name: "Test Session 2024-10-28T10:30:00".to_string(),
            timestamp: 1698508200000.0,
            tabs: vec![
                SavedTab {
                    url: "https://google.com".to_string(),
                    title: "Google".to_string(),
                    domain: "google.com".to_string(),
                    pinned: false,
                },
            ],
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: CollapsedSession = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "test-123");
        assert_eq!(deserialized.tabs.len(), 1);
    }
}
