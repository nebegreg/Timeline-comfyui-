/// Marketplace Manager - Async bridge for plugin marketplace operations
/// Phase 5: Plugin Marketplace - UI Integration
///
/// Handles async marketplace operations in a background thread and communicates
/// with the egui UI via channels.
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use plugin_host::marketplace::{
    MarketplacePlugin, PluginMarketplace, SearchQuery as MpSearchQuery,
    SearchResults as MpSearchResults,
};

// Re-export types for convenience
pub type SearchQuery = MpSearchQuery;
pub type SearchResults = MpSearchResults;

/// Commands sent from UI to marketplace manager
#[derive(Debug, Clone)]
pub enum MarketplaceCommand {
    /// Search for plugins with given query
    Search(SearchQuery),

    /// Get featured plugins
    GetFeatured,

    /// Install a plugin by ID
    InstallPlugin { plugin_id: String },

    /// Get plugin details
    GetPluginDetails { plugin_id: String },

    /// Check for updates
    CheckUpdates,

    /// Shutdown the manager
    Shutdown,
}

/// Responses sent from marketplace manager to UI
#[derive(Debug, Clone)]
pub enum MarketplaceResponse {
    /// Search results ready
    SearchResults(SearchResults),

    /// Featured plugins ready
    FeaturedPlugins(Vec<MarketplacePlugin>),

    /// Plugin installation progress
    InstallProgress { plugin_id: String, progress: f32 },

    /// Plugin installation complete
    InstallComplete {
        plugin_id: String,
        success: bool,
        error: Option<String>,
    },

    /// Plugin details ready
    PluginDetails(MarketplacePlugin),

    /// Error occurred
    Error { message: String },

    /// Updates available
    UpdatesAvailable(Vec<MarketplacePlugin>),
}

/// Marketplace manager state
pub struct MarketplaceManager {
    command_tx: Sender<MarketplaceCommand>,
    response_rx: Receiver<MarketplaceResponse>,
}

impl MarketplaceManager {
    /// Create new marketplace manager and start background thread
    pub fn new(marketplace_url: String) -> Self {
        let (command_tx, command_rx) = channel();
        let (response_tx, response_rx) = channel();

        // Spawn background thread for async operations
        thread::spawn(move || {
            run_marketplace_thread(marketplace_url, command_rx, response_tx);
        });

        Self {
            command_tx,
            response_rx,
        }
    }

    /// Send a command to the marketplace manager
    pub fn send_command(&self, cmd: MarketplaceCommand) -> Result<(), String> {
        self.command_tx
            .send(cmd)
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Poll for responses (call every frame from egui)
    pub fn poll_responses(&self) -> Vec<MarketplaceResponse> {
        let mut responses = Vec::new();
        while let Ok(response) = self.response_rx.try_recv() {
            responses.push(response);
        }
        responses
    }

    /// Check if there are pending responses
    pub fn has_responses(&self) -> bool {
        matches!(self.response_rx.try_recv(), Ok(_))
    }

    /// Get a clone of the command sender for UI integration
    pub fn get_sender(&self) -> Sender<MarketplaceCommand> {
        self.command_tx.clone()
    }
}

/// Background thread that runs async marketplace operations
fn run_marketplace_thread(
    marketplace_url: String,
    command_rx: Receiver<MarketplaceCommand>,
    response_tx: Sender<MarketplaceResponse>,
) {
    // Create tokio runtime for async operations
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            let _ = response_tx.send(MarketplaceResponse::Error {
                message: format!("Failed to create async runtime: {}", e),
            });
            return;
        }
    };

    // Create marketplace client
    let marketplace = match rt.block_on(async {
        let cache_dir = std::path::PathBuf::from("marketplace_cache");
        PluginMarketplace::new(marketplace_url, cache_dir)
    }) {
        Ok(mp) => mp,
        Err(e) => {
            let _ = response_tx.send(MarketplaceResponse::Error {
                message: format!("Failed to create marketplace client: {}", e),
            });
            return;
        }
    };

    // Process commands
    while let Ok(cmd) = command_rx.recv() {
        match cmd {
            MarketplaceCommand::Search(query) => {
                let response_tx = response_tx.clone();
                let marketplace = marketplace.clone();

                rt.spawn(async move {
                    match marketplace.search_plugins(query).await {
                        Ok(results) => {
                            let _ = response_tx.send(MarketplaceResponse::SearchResults(results));
                        }
                        Err(e) => {
                            let _ = response_tx.send(MarketplaceResponse::Error {
                                message: format!("Search failed: {}", e),
                            });
                        }
                    }
                });
            }

            MarketplaceCommand::GetFeatured => {
                let response_tx = response_tx.clone();
                let marketplace = marketplace.clone();

                rt.spawn(async move {
                    match marketplace.get_featured_plugins().await {
                        Ok(plugins) => {
                            let _ = response_tx.send(MarketplaceResponse::FeaturedPlugins(plugins));
                        }
                        Err(e) => {
                            let _ = response_tx.send(MarketplaceResponse::Error {
                                message: format!("Failed to get featured plugins: {}", e),
                            });
                        }
                    }
                });
            }

            MarketplaceCommand::InstallPlugin { plugin_id } => {
                let response_tx = response_tx.clone();
                let marketplace = marketplace.clone();
                let id = plugin_id.clone();

                // Send initial progress
                let _ = response_tx.send(MarketplaceResponse::InstallProgress {
                    plugin_id: id.clone(),
                    progress: 0.0,
                });

                rt.spawn(async move {
                    let install_dir = std::path::PathBuf::from("plugins");

                    match marketplace.install_plugin(&id, &install_dir).await {
                        Ok(_) => {
                            let _ = response_tx.send(MarketplaceResponse::InstallComplete {
                                plugin_id: id,
                                success: true,
                                error: None,
                            });
                        }
                        Err(e) => {
                            let _ = response_tx.send(MarketplaceResponse::InstallComplete {
                                plugin_id: id,
                                success: false,
                                error: Some(format!("{}", e)),
                            });
                        }
                    }
                });
            }

            MarketplaceCommand::GetPluginDetails { plugin_id } => {
                let response_tx = response_tx.clone();
                let marketplace = marketplace.clone();

                rt.spawn(async move {
                    match marketplace.get_plugin_details(&plugin_id).await {
                        Ok(plugin) => {
                            let _ = response_tx.send(MarketplaceResponse::PluginDetails(plugin));
                        }
                        Err(e) => {
                            let _ = response_tx.send(MarketplaceResponse::Error {
                                message: format!("Failed to get plugin details: {}", e),
                            });
                        }
                    }
                });
            }

            MarketplaceCommand::CheckUpdates => {
                let response_tx = response_tx.clone();
                let marketplace = marketplace.clone();

                rt.spawn(async move {
                    // TODO: Get installed plugins from somewhere
                    let installed = std::collections::HashMap::new();

                    match marketplace.check_updates(&installed).await {
                        Ok(updates) => {
                            let _ =
                                response_tx.send(MarketplaceResponse::UpdatesAvailable(updates));
                        }
                        Err(e) => {
                            let _ = response_tx.send(MarketplaceResponse::Error {
                                message: format!("Failed to check updates: {}", e),
                            });
                        }
                    }
                });
            }

            MarketplaceCommand::Shutdown => {
                // Clean shutdown
                break;
            }
        }
    }
}
