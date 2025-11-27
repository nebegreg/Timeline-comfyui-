/// Plugin Marketplace Data Models
/// Structures for plugins, ratings, and metadata

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// Unique plugin ID
    pub id: String,

    /// Plugin name
    pub name: String,

    /// Short description (< 200 chars)
    pub description: String,

    /// Long description with markdown
    pub long_description: Option<String>,

    /// Current version (semver)
    pub version: String,

    /// Author name or organization
    pub author: String,

    /// Author email
    pub author_email: Option<String>,

    /// Plugin type: "wasm" or "python"
    pub plugin_type: String,

    /// Category: "effect", "transition", "audio", "utility", etc.
    pub category: String,

    /// Tags for search (comma-separated)
    pub tags: String,

    /// Download URL for plugin file
    pub download_url: String,

    /// File size in bytes
    pub file_size: i64,

    /// SHA256 hash for integrity
    pub file_hash: String,

    /// License (MIT, GPL, etc.)
    pub license: String,

    /// Homepage/repository URL
    pub homepage: Option<String>,

    /// Screenshot URLs (JSON array)
    pub screenshots: Option<String>,

    /// Total downloads
    pub downloads: i64,

    /// Average rating (0.0 - 5.0)
    pub rating: f32,

    /// Number of ratings
    pub rating_count: i32,

    /// Is this plugin verified/official?
    pub verified: bool,

    /// Upload timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Plugin creation request
#[derive(Debug, Deserialize)]
pub struct CreatePluginRequest {
    pub name: String,
    pub description: String,
    pub long_description: Option<String>,
    pub version: String,
    pub author: String,
    pub author_email: Option<String>,
    pub plugin_type: String,
    pub category: String,
    pub tags: Vec<String>,
    pub license: String,
    pub homepage: Option<String>,
    pub screenshots: Option<Vec<String>>,
    /// Base64-encoded plugin file
    pub file_data: String,
}

/// Plugin rating/review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub id: i64,
    pub plugin_id: String,
    pub user_id: String,
    pub rating: i32,  // 1-5 stars
    pub review: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Rating creation request
#[derive(Debug, Deserialize)]
pub struct CreateRatingRequest {
    pub user_id: String,
    pub rating: i32,
    pub review: Option<String>,
}

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,           // Search term
    pub category: Option<String>,    // Filter by category
    pub plugin_type: Option<String>, // Filter by type
    pub tag: Option<String>,         // Filter by tag
    pub sort: Option<String>,        // Sort: "downloads", "rating", "recent"
    pub page: Option<i64>,           // Page number (1-based)
    pub limit: Option<i64>,          // Results per page
}

/// Search results with pagination
#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub plugins: Vec<Plugin>,
    pub total: i64,
    pub page: i64,
    pub pages: i64,
}

/// Plugin statistics
#[derive(Debug, Serialize)]
pub struct PluginStats {
    pub total_plugins: i64,
    pub total_downloads: i64,
    pub verified_plugins: i64,
}
