/// Storage serialization utilities for chrome.storage.local

use crate::tab_data::CollapsedSession;
use serde::{Deserialize, Serialize};

/// Root storage structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageData {
    pub sessions: Vec<CollapsedSession>,
}

impl StorageData {
    pub fn new() -> Self {
        StorageData {
            sessions: Vec::new(),
        }
    }

    pub fn add_session(&mut self, session: CollapsedSession) {
        self.sessions.push(session);
    }

    pub fn remove_session(&mut self, session_id: &str) -> bool {
        let original_len = self.sessions.len();
        self.sessions.retain(|s| s.id != session_id);
        self.sessions.len() < original_len
    }

    pub fn get_session(&self, session_id: &str) -> Option<&CollapsedSession> {
        self.sessions.iter().find(|s| s.id == session_id)
    }

    pub fn update_session_name(&mut self, session_id: &str, new_name: String) -> bool {
        self.sessions
            .iter_mut()
            .find(|s| s.id == session_id)
            .map(|session| {
                session.name = new_name;
            })
            .is_some()
    }
}

impl Default for StorageData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab_data::SavedTab;

    fn create_test_session(id: &str, name: &str) -> CollapsedSession {
        CollapsedSession {
            id: id.to_string(),
            name: name.to_string(),
            timestamp: 1698508200000.0,
            tabs: vec![SavedTab {
                url: "https://google.com".to_string(),
                title: "Google".to_string(),
                domain: "google.com".to_string(),
                pinned: false,
            }],
        }
    }

    #[test]
    fn test_storage_data_new() {
        let storage = StorageData::new();
        assert_eq!(storage.sessions.len(), 0);
    }

    #[test]
    fn test_add_session() {
        let mut storage = StorageData::new();
        let session = create_test_session("session-1", "Test Session");

        storage.add_session(session);

        assert_eq!(storage.sessions.len(), 1);
        assert_eq!(storage.sessions[0].id, "session-1");
    }

    #[test]
    fn test_remove_session() {
        let mut storage = StorageData::new();
        storage.add_session(create_test_session("session-1", "Session 1"));
        storage.add_session(create_test_session("session-2", "Session 2"));

        let removed = storage.remove_session("session-1");

        assert!(removed);
        assert_eq!(storage.sessions.len(), 1);
        assert_eq!(storage.sessions[0].id, "session-2");
    }

    #[test]
    fn test_remove_nonexistent_session() {
        let mut storage = StorageData::new();
        storage.add_session(create_test_session("session-1", "Session 1"));

        let removed = storage.remove_session("nonexistent");

        assert!(!removed);
        assert_eq!(storage.sessions.len(), 1);
    }

    #[test]
    fn test_get_session() {
        let mut storage = StorageData::new();
        storage.add_session(create_test_session("session-1", "Test Session"));

        let session = storage.get_session("session-1");

        assert!(session.is_some());
        assert_eq!(session.unwrap().name, "Test Session");
    }

    #[test]
    fn test_update_session_name() {
        let mut storage = StorageData::new();
        storage.add_session(create_test_session("session-1", "Old Name"));

        let updated = storage.update_session_name("session-1", "New Name".to_string());

        assert!(updated);
        assert_eq!(storage.sessions[0].name, "New Name");
    }

    #[test]
    fn test_serialization() {
        let mut storage = StorageData::new();
        storage.add_session(create_test_session("session-1", "Test"));

        let json = serde_json::to_string(&storage).unwrap();
        let deserialized: StorageData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sessions.len(), 1);
        assert_eq!(deserialized.sessions[0].id, "session-1");
    }
}
