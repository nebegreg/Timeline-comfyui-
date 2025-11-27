///! Plugin Marketplace Server
///! REST API server for plugin catalog, ratings, and downloads

mod api;
mod models;
mod storage;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use storage::MarketplaceStorage;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::models::Plugin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("marketplace_server=debug,axum=debug")
        .init();

    info!("Starting Plugin Marketplace Server...");

    // Create storage
    let storage_dir = "marketplace_data";
    let storage = Arc::new(MarketplaceStorage::new(storage_dir)?);
    info!("Storage initialized at: {}", storage_dir);

    // Seed with example plugins if empty
    if storage.list_plugins(&models::SearchQuery {
        q: None,
        category: None,
        plugin_type: None,
        tag: None,
        sort: None,
        page: None,
        limit: Some(1),
    }).total == 0 {
        info!("Seeding database with example plugins...");
        seed_storage(&storage)?;
    }

    // Build API router
    let app = Router::new()
        // Plugin endpoints
        .route("/api/plugins", get(api::list_plugins).post(api::create_plugin))
        .route("/api/plugins/:id", get(api::get_plugin))
        .route("/api/plugins/:id/ratings", get(api::get_ratings).post(api::create_rating))
        .route("/api/plugins/:id/download", post(api::record_download))
        // Stats endpoint
        .route("/api/stats", get(api::get_stats))
        // CORS for local development
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(storage);

    // Start server
    let addr = "127.0.0.1:3000";
    info!("Marketplace server listening on http://{}", addr);
    info!("API endpoints:");
    info!("  GET  /api/plugins          - List plugins");
    info!("  POST /api/plugins          - Upload plugin");
    info!("  GET  /api/plugins/:id      - Get plugin details");
    info!("  GET  /api/plugins/:id/ratings  - Get ratings");
    info!("  POST /api/plugins/:id/ratings - Add rating");
    info!("  POST /api/plugins/:id/download - Record download");
    info!("  GET  /api/stats            - Marketplace stats");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Seed storage with example plugins for testing
fn seed_storage(storage: &MarketplaceStorage) -> anyhow::Result<()> {
    use chrono::Utc;

    let plugins = vec![
        (
            "vintage-film-effect",
            "Vintage Film Effect",
            "Add nostalgic film grain and color tones to your footage",
            "python",
            "effect",
            "vintage,film,grain,retro",
            "MIT",
            true,
        ),
        (
            "smooth-zoom-transition",
            "Smooth Zoom Transition",
            "Professional zoom transition with motion blur",
            "wasm",
            "transition",
            "zoom,transition,smooth",
            "MIT",
            true,
        ),
        (
            "audio-normalizer",
            "Audio Normalizer",
            "Automatically normalize audio levels across clips",
            "python",
            "audio",
            "audio,normalize,volume",
            "Apache-2.0",
            false,
        ),
        (
            "color-pop-effect",
            "Color Pop Effect",
            "Isolate a specific color while desaturating the rest",
            "wasm",
            "effect",
            "color,pop,creative",
            "MIT",
            true,
        ),
        (
            "auto-subtitle",
            "Auto Subtitle Generator",
            "Generate subtitles using AI speech recognition",
            "python",
            "utility",
            "subtitle,ai,transcription",
            "GPL-3.0",
            false,
        ),
    ];

    let plugins_count = plugins.len();

    for (id, name, desc, ptype, category, tags, license, verified) in plugins {
        let plugin = Plugin {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            long_description: Some(format!(
                "## {}\n\n{}\n\n### Features\n- Professional quality\n- Easy to use\n- GPU accelerated",
                name, desc
            )),
            version: "1.0.0".to_string(),
            author: "Gausian Team".to_string(),
            author_email: Some("plugins@gausian.dev".to_string()),
            plugin_type: ptype.to_string(),
            category: category.to_string(),
            tags: tags.to_string(),
            download_url: format!("/api/plugins/{}/download", id),
            file_size: 1024 * 512,
            file_hash: "abc123def456".to_string(),
            license: license.to_string(),
            homepage: Some(format!("https://github.com/gausian/{}", id)),
            screenshots: None,
            downloads: 500 + (id.len() * 100) as i64,
            rating: 4.0 + (id.len() % 10) as f32 / 10.0,
            rating_count: 25 + (id.len() % 75) as i32,
            verified,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        storage.add_plugin(plugin)?;
    }

    info!("Seeded {} example plugins", plugins_count);
    Ok(())
}
