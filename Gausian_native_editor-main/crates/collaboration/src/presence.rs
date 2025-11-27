/// User presence tracking for collaborative editing
/// Shows where users are working and their current selection
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use timeline::{Frame, NodeId};

use crate::{SessionId, UserId};

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub color: UserColor,
    pub avatar_url: Option<String>,
}

impl User {
    pub fn new(id: UserId, name: String) -> Self {
        Self {
            id,
            name,
            color: UserColor::from_user_id(id),
            avatar_url: None,
        }
    }
}

/// Color assigned to a user for cursor/selection highlighting
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UserColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl UserColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Generate a color based on user ID (deterministic)
    pub fn from_user_id(user_id: UserId) -> Self {
        let bytes = user_id.0.as_bytes();
        Self {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
        }
    }

    /// Convert to hex color string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// User's current state in the editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user: User,
    pub session_id: SessionId,
    pub cursor_position: Option<CursorPosition>,
    pub selection: Option<Selection>,
    pub viewport: Option<Viewport>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

impl UserPresence {
    pub fn new(user: User, session_id: SessionId) -> Self {
        Self {
            user,
            session_id,
            cursor_position: None,
            selection: None,
            viewport: None,
            last_activity: chrono::Utc::now(),
            is_active: true,
        }
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now();
    }

    /// Check if user is considered idle (no activity for 60 seconds)
    pub fn is_idle(&self) -> bool {
        let now = chrono::Utc::now();
        let duration = now - self.last_activity;
        duration.num_seconds() > 60
    }
}

/// Cursor position in the timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Frame position
    pub frame: Frame,

    /// Track index (if hovering over a track)
    pub track_index: Option<usize>,
}

/// Selection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    /// Selected nodes
    pub node_ids: Vec<NodeId>,

    /// Frame range (for range selection)
    pub frame_range: Option<(Frame, Frame)>,
}

/// Viewport state (what the user is looking at)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// Visible frame range
    pub visible_start: Frame,
    pub visible_end: Frame,

    /// Zoom level
    pub zoom: f32,
}

/// Presence manager tracks all users in a session
#[derive(Debug, Clone, Default)]
pub struct PresenceManager {
    users: HashMap<UserId, UserPresence>,
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Add or update user presence
    pub fn update_user(&mut self, presence: UserPresence) {
        self.users.insert(presence.user.id, presence);
    }

    /// Remove user
    pub fn remove_user(&mut self, user_id: &UserId) {
        self.users.remove(user_id);
    }

    /// Get user presence
    pub fn get_user(&self, user_id: &UserId) -> Option<&UserPresence> {
        self.users.get(user_id)
    }

    /// Get user presence (mutable)
    pub fn get_user_mut(&mut self, user_id: &UserId) -> Option<&mut UserPresence> {
        self.users.get_mut(user_id)
    }

    /// Get all active users
    pub fn get_active_users(&self) -> Vec<&UserPresence> {
        self.users
            .values()
            .filter(|p| p.is_active && !p.is_idle())
            .collect()
    }

    /// Get all users
    pub fn get_all_users(&self) -> Vec<&UserPresence> {
        self.users.values().collect()
    }

    /// Update cursor position for a user
    pub fn update_cursor(&mut self, user_id: UserId, cursor: CursorPosition) {
        if let Some(presence) = self.users.get_mut(&user_id) {
            presence.cursor_position = Some(cursor);
            presence.touch();
        }
    }

    /// Update selection for a user
    pub fn update_selection(&mut self, user_id: UserId, selection: Selection) {
        if let Some(presence) = self.users.get_mut(&user_id) {
            presence.selection = Some(selection);
            presence.touch();
        }
    }

    /// Update viewport for a user
    pub fn update_viewport(&mut self, user_id: UserId, viewport: Viewport) {
        if let Some(presence) = self.users.get_mut(&user_id) {
            presence.viewport = Some(viewport);
            presence.touch();
        }
    }

    /// Get users currently viewing a specific node
    pub fn get_users_viewing_node(&self, node_id: &NodeId) -> Vec<&UserPresence> {
        self.users
            .values()
            .filter(|p| {
                p.selection
                    .as_ref()
                    .map(|s| s.node_ids.contains(node_id))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Clean up idle users (inactive for > 5 minutes)
    pub fn cleanup_idle_users(&mut self) {
        let now = chrono::Utc::now();
        self.users.retain(|_, presence| {
            let duration = now - presence.last_activity;
            duration.num_seconds() <= 300 // 5 minutes
        });
    }
}

/// Presence update message (sent via WebSocket)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PresenceUpdate {
    #[serde(rename = "user_joined")]
    UserJoined { user: User },

    #[serde(rename = "user_left")]
    UserLeft { user_id: UserId },

    #[serde(rename = "cursor_moved")]
    CursorMoved {
        user_id: UserId,
        position: CursorPosition,
    },

    #[serde(rename = "selection_changed")]
    SelectionChanged {
        user_id: UserId,
        selection: Selection,
    },

    #[serde(rename = "viewport_changed")]
    ViewportChanged { user_id: UserId, viewport: Viewport },

    #[serde(rename = "user_idle")]
    UserIdle { user_id: UserId },

    #[serde(rename = "user_active")]
    UserActive { user_id: UserId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_color_from_id() {
        let user_id = UserId::new();
        let color = UserColor::from_user_id(user_id);

        // Just check it doesn't panic
        let hex = color.to_hex();
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
    }

    #[test]
    fn test_presence_manager() {
        let mut manager = PresenceManager::new();

        let user = User {
            id: UserId::new(),
            name: "Alice".to_string(),
            color: UserColor::new(255, 0, 0),
            avatar_url: None,
        };

        let presence = UserPresence::new(user.clone(), SessionId::new());
        let user_id = presence.user.id;

        manager.update_user(presence);

        // Check user was added
        assert!(manager.get_user(&user_id).is_some());

        // Update cursor
        manager.update_cursor(
            user_id,
            CursorPosition {
                frame: 100,
                track_index: Some(0),
            },
        );

        // Check cursor was updated
        let presence = manager.get_user(&user_id).unwrap();
        assert_eq!(presence.cursor_position.as_ref().unwrap().frame, 100);
    }

    #[test]
    fn test_idle_detection() {
        let user = User {
            id: UserId::new(),
            name: "Bob".to_string(),
            color: UserColor::new(0, 255, 0),
            avatar_url: None,
        };

        let mut presence = UserPresence::new(user, SessionId::new());

        // Initially not idle
        assert!(!presence.is_idle());

        // Set activity to 2 minutes ago
        presence.last_activity = chrono::Utc::now() - chrono::Duration::seconds(120);

        // Should be considered idle
        assert!(presence.is_idle());
    }
}
