/// REST API endpoints for plugin marketplace
/// Handles plugin CRUD, search, ratings, and downloads

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sha2::Digest;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::storage::MarketplaceStorage;

/// API error type
pub enum ApiError {
    StorageError(String),
    NotFound,
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::StorageError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", e))
            }
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// GET /api/plugins - List all plugins with optional pagination
pub async fn list_plugins(
    State(storage): State<Arc<MarketplaceStorage>>,
    Query(query): Query<SearchQuery>,
) -> Json<SearchResults> {
    Json(storage.list_plugins(&query))
}

/// GET /api/plugins/:id - Get plugin details
pub async fn get_plugin(
    State(storage): State<Arc<MarketplaceStorage>>,
    Path(id): Path<String>,
) -> Result<Json<Plugin>, ApiError> {
    storage
        .get_plugin(&id)
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// POST /api/plugins - Create a new plugin
pub async fn create_plugin(
    State(storage): State<Arc<MarketplaceStorage>>,
    Json(req): Json<CreatePluginRequest>,
) -> Result<Json<Plugin>, ApiError> {
    // Validate
    if req.name.is_empty() {
        return Err(ApiError::BadRequest("Name is required".to_string()));
    }

    if !["wasm", "python"].contains(&req.plugin_type.as_str()) {
        return Err(ApiError::BadRequest(
            "Plugin type must be 'wasm' or 'python'".to_string(),
        ));
    }

    // Decode plugin file
    use base64::Engine;
    let file_data = base64::engine::general_purpose::STANDARD
        .decode(&req.file_data)
        .map_err(|_| ApiError::BadRequest("Invalid base64 file data".to_string()))?;

    let file_hash = format!("{:x}", sha2::Sha256::digest(&file_data));
    let file_size = file_data.len() as i64;

    // Generate plugin ID
    let id = Uuid::new_v4().to_string();

    // TODO: Save file to storage
    let download_url = format!("/api/plugins/{}/download", id);

    let tags = req.tags.join(",");
    let screenshots = req.screenshots.map(|s| serde_json::to_string(&s).unwrap());

    let plugin = Plugin {
        id: id.clone(),
        name: req.name,
        description: req.description,
        long_description: req.long_description,
        version: req.version,
        author: req.author,
        author_email: req.author_email,
        plugin_type: req.plugin_type,
        category: req.category,
        tags,
        download_url,
        file_size,
        file_hash,
        license: req.license,
        homepage: req.homepage,
        screenshots,
        downloads: 0,
        rating: 0.0,
        rating_count: 0,
        verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    storage
        .add_plugin(plugin)
        .map(Json)
        .map_err(|e| ApiError::StorageError(e.to_string()))
}

/// POST /api/plugins/:id/ratings - Add a rating/review
pub async fn create_rating(
    State(storage): State<Arc<MarketplaceStorage>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<CreateRatingRequest>,
) -> Result<Json<Rating>, ApiError> {
    if req.rating < 1 || req.rating > 5 {
        return Err(ApiError::BadRequest(
            "Rating must be between 1 and 5".to_string(),
        ));
    }

    // Check if plugin exists
    if storage.get_plugin(&plugin_id).is_none() {
        return Err(ApiError::NotFound);
    }

    let rating = Rating {
        id: chrono::Utc::now().timestamp_millis(),
        plugin_id: plugin_id.clone(),
        user_id: req.user_id,
        rating: req.rating,
        review: req.review,
        created_at: chrono::Utc::now(),
    };

    storage
        .add_rating(plugin_id, rating)
        .map(Json)
        .map_err(|e| ApiError::StorageError(e.to_string()))
}

/// GET /api/plugins/:id/ratings - Get all ratings for a plugin
pub async fn get_ratings(
    State(storage): State<Arc<MarketplaceStorage>>,
    Path(plugin_id): Path<String>,
) -> Json<Vec<Rating>> {
    Json(storage.get_ratings(&plugin_id))
}

/// POST /api/plugins/:id/download - Increment download count
pub async fn record_download(
    State(storage): State<Arc<MarketplaceStorage>>,
    Path(plugin_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    storage
        .increment_downloads(&plugin_id)
        .map(|_| StatusCode::OK)
        .map_err(|e| ApiError::StorageError(e.to_string()))
}

/// GET /api/stats - Get marketplace statistics
pub async fn get_stats(
    State(storage): State<Arc<MarketplaceStorage>>,
) -> Json<serde_json::Value> {
    Json(storage.get_stats())
}
