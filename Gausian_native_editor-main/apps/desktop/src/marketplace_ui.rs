/// Phase 5: Plugin Marketplace UI
/// Browse, search, install, and manage plugins from the marketplace

use egui::{Color32, Context, Response, RichText, ScrollArea, TextureHandle, Ui, Vec2, Window};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::marketplace_manager::{MarketplaceCommand, SearchQuery};

/// Plugin info from marketplace API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub long_description: Option<String>,
    pub version: String,
    pub author: String,
    pub plugin_type: String,
    pub category: String,
    pub tags: String,
    pub downloads: i64,
    pub rating: f32,
    pub rating_count: i32,
    pub verified: bool,
    pub file_size: i64,
    pub license: String,
}

/// Search results from API
#[derive(Debug, Deserialize)]
pub struct SearchResults {
    pub plugins: Vec<MarketplacePlugin>,
    pub total: i64,
    pub page: i64,
    pub pages: i64,
}

/// Marketplace UI state
pub struct MarketplaceUI {
    /// Is marketplace panel open?
    pub show_panel: bool,

    /// Current search query
    pub search_query: String,

    /// Selected category filter
    pub selected_category: Option<String>,

    /// Selected plugin type filter
    pub selected_type: Option<String>,

    /// Sort mode
    pub sort_by: SortMode,

    /// Current page
    pub current_page: i64,

    /// Search results
    pub results: Option<SearchResults>,

    /// Selected plugin for details view
    pub selected_plugin: Option<MarketplacePlugin>,

    /// Installed plugins
    pub installed_plugins: HashMap<String, String>, // id -> version

    /// Plugin being installed
    pub installing: Option<String>,

    /// Connection status
    pub connection_status: ConnectionStatus,

    /// Marketplace server URL
    pub server_url: String,

    /// Command sender to marketplace manager
    command_sender: Option<Sender<MarketplaceCommand>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortMode {
    Downloads,
    Rating,
    Recent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

impl MarketplaceUI {
    pub fn new() -> Self {
        Self {
            show_panel: false,
            search_query: String::new(),
            selected_category: None,
            selected_type: None,
            sort_by: SortMode::Downloads,
            current_page: 1,
            results: None,
            selected_plugin: None,
            installed_plugins: HashMap::new(),
            installing: None,
            connection_status: ConnectionStatus::Disconnected,
            server_url: "http://127.0.0.1:3000".to_string(),
            command_sender: None,
        }
    }

    /// Set the command sender for marketplace operations
    pub fn set_command_sender(&mut self, sender: Sender<MarketplaceCommand>) {
        self.command_sender = Some(sender);
    }

    /// Toggle marketplace panel
    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;

        // Load plugins when opening
        if self.show_panel && self.results.is_none() {
            self.refresh_plugins();
        }
    }

    /// Render marketplace panel
    pub fn render(&mut self, ctx: &Context) {
        if !self.show_panel {
            return;
        }

        Window::new("🛍️ Plugin Marketplace")
            .default_width(800.0)
            .default_height(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
    }

    fn render_content(&mut self, ui: &mut Ui) {
        // Top toolbar
        ui.horizontal(|ui| {
            ui.heading("Plugin Marketplace");
            ui.add_space(10.0);

            // Connection status
            self.render_connection_status(ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_plugins();
                }

                if ui.button("📦 Installed").clicked() {
                    self.show_installed_plugins();
                }
            });
        });

        ui.separator();

        // Search and filters
        ui.horizontal(|ui| {
            ui.label("🔍");
            let response = ui.text_edit_singleline(&mut self.search_query);
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.perform_search();
            }

            if ui.button("Search").clicked() {
                self.perform_search();
            }

            ui.separator();

            // Category filter
            ui.label("Category:");
            egui::ComboBox::from_id_source("category_filter")
                .selected_text(self.selected_category.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_category, None, "All");
                    ui.selectable_value(&mut self.selected_category, Some("effect".to_string()), "Effects");
                    ui.selectable_value(&mut self.selected_category, Some("transition".to_string()), "Transitions");
                    ui.selectable_value(&mut self.selected_category, Some("audio".to_string()), "Audio");
                    ui.selectable_value(&mut self.selected_category, Some("utility".to_string()), "Utilities");
                });

            // Type filter
            ui.label("Type:");
            egui::ComboBox::from_id_source("type_filter")
                .selected_text(self.selected_type.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_type, None, "All");
                    ui.selectable_value(&mut self.selected_type, Some("wasm".to_string()), "WASM");
                    ui.selectable_value(&mut self.selected_type, Some("python".to_string()), "Python");
                });

            // Sort
            ui.label("Sort:");
            egui::ComboBox::from_id_source("sort_mode")
                .selected_text(format!("{:?}", self.sort_by))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_by, SortMode::Downloads, "Downloads");
                    ui.selectable_value(&mut self.sort_by, SortMode::Rating, "Rating");
                    ui.selectable_value(&mut self.sort_by, SortMode::Recent, "Recent");
                });
        });

        ui.separator();

        // Plugin grid
        if let Some(results) = &self.results {
            ScrollArea::vertical().show(ui, |ui| {
                self.render_plugin_grid(ui, &results.plugins);
            });

            // Pagination
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Page {} of {} ({} plugins)", results.page, results.pages, results.total));

                ui.add_space(20.0);

                if results.page > 1 && ui.button("◀ Previous").clicked() {
                    self.current_page -= 1;
                    self.refresh_plugins();
                }

                if results.page < results.pages && ui.button("Next ▶").clicked() {
                    self.current_page += 1;
                    self.refresh_plugins();
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Loading plugins...");
            });
        }
    }

    fn render_connection_status(&self, ui: &mut Ui) {
        let (icon, color, tooltip) = match &self.connection_status {
            ConnectionStatus::Connected => ("●", Color32::GREEN, "Connected to marketplace"),
            ConnectionStatus::Connecting => ("⟳", Color32::YELLOW, "Connecting..."),
            ConnectionStatus::Disconnected => ("●", Color32::GRAY, "Disconnected"),
            ConnectionStatus::Error(err) => ("✗", Color32::RED, err.as_str()),
        };

        ui.colored_label(color, icon).on_hover_text(tooltip);
    }

    fn render_plugin_grid(&mut self, ui: &mut Ui, plugins: &[MarketplacePlugin]) {
        let available_width = ui.available_width();
        let card_width = 350.0;
        let columns = (available_width / card_width).floor().max(1.0) as usize;

        egui::Grid::new("plugin_grid")
            .num_columns(columns)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                for (idx, plugin) in plugins.iter().enumerate() {
                    if idx > 0 && idx % columns == 0 {
                        ui.end_row();
                    }
                    self.render_plugin_card(ui, plugin);
                }
            });
    }

    fn render_plugin_card(&mut self, ui: &mut Ui, plugin: &MarketplacePlugin) {
        let is_installed = self.installed_plugins.contains_key(&plugin.id);
        let is_installing = self.installing.as_ref() == Some(&plugin.id);

        egui::Frame::group(ui.style())
            .fill(ui.visuals().window_fill())
            .stroke(ui.visuals().window_stroke)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.set_width(330.0);
                ui.set_height(200.0);

                // Header with name and verified badge
                ui.horizontal(|ui| {
                    ui.heading(&plugin.name);
                    if plugin.verified {
                        ui.label(RichText::new("✓").color(Color32::from_rgb(0, 150, 255)))
                            .on_hover_text("Verified plugin");
                    }
                });

                // Category and type badges
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&plugin.category).small().background_color(Color32::from_gray(60)));
                    ui.label(RichText::new(&plugin.plugin_type).small().background_color(Color32::from_gray(80)));
                });

                ui.add_space(5.0);

                // Description
                ui.label(&plugin.description);

                ui.add_space(5.0);

                // Stats
                ui.horizontal(|ui| {
                    // Rating
                    ui.label("⭐");
                    ui.label(format!("{:.1} ({} reviews)", plugin.rating, plugin.rating_count));

                    ui.add_space(10.0);

                    // Downloads
                    ui.label("📥");
                    ui.label(format_downloads(plugin.downloads));
                });

                ui.add_space(5.0);

                // Author and version
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("by {}", plugin.author)).small().italics());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("v{}", plugin.version)).small());
                    });
                });

                ui.add_space(10.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if is_installed {
                        ui.colored_label(Color32::GREEN, "✓ Installed");
                        if ui.button("Update").clicked() {
                            self.update_plugin(plugin.id.clone());
                        }
                    } else if is_installing {
                        ui.spinner();
                        ui.label("Installing...");
                    } else {
                        if ui.button("📥 Install").clicked() {
                            self.install_plugin(plugin.id.clone());
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Details").clicked() {
                            self.selected_plugin = Some(plugin.clone());
                        }
                    });
                });
            });
    }

    /// Refresh plugin list from marketplace
    fn refresh_plugins(&mut self) {
        self.connection_status = ConnectionStatus::Connecting;

        // Build search query
        let query = SearchQuery {
            query: if self.search_query.is_empty() {
                None
            } else {
                Some(self.search_query.clone())
            },
            category: self.selected_category.clone(),
            plugin_type: self.selected_type.clone(),
            sort_by: match self.sort_by {
                SortMode::Downloads => plugin_host::marketplace::SortBy::Downloads,
                SortMode::Rating => plugin_host::marketplace::SortBy::Rating,
                SortMode::Recent => plugin_host::marketplace::SortBy::Updated,
            },
            page: self.current_page as u32,
            per_page: 20,
            free_only: false,
            tags: vec![],
        };

        // Send command to manager
        if let Some(sender) = &self.command_sender {
            match sender.send(MarketplaceCommand::Search(query)) {
                Ok(_) => {
                    tracing::info!("Marketplace search command sent");
                }
                Err(e) => {
                    tracing::error!("Failed to send marketplace command: {}", e);
                    self.connection_status = ConnectionStatus::Error(format!("Failed to send command: {}", e));
                }
            }
        } else {
            tracing::warn!("Marketplace command sender not initialized");
            self.connection_status = ConnectionStatus::Error("Marketplace not initialized".to_string());
        }
    }

    fn perform_search(&mut self) {
        self.current_page = 1;
        self.refresh_plugins();
    }

    fn install_plugin(&mut self, plugin_id: String) {
        self.installing = Some(plugin_id.clone());
        tracing::info!("Installing plugin: {}", plugin_id);

        // Send install command to manager
        if let Some(sender) = &self.command_sender {
            match sender.send(MarketplaceCommand::InstallPlugin {
                plugin_id: plugin_id.clone(),
            }) {
                Ok(_) => {
                    tracing::info!("Plugin install command sent for: {}", plugin_id);
                }
                Err(e) => {
                    tracing::error!("Failed to send install command: {}", e);
                    self.installing = None;
                    self.connection_status = ConnectionStatus::Error(format!("Failed to install: {}", e));
                }
            }
        } else {
            tracing::warn!("Marketplace command sender not initialized");
            self.installing = None;
            self.connection_status = ConnectionStatus::Error("Marketplace not initialized".to_string());
        }
    }

    fn update_plugin(&mut self, plugin_id: String) {
        tracing::info!("Updating plugin: {}", plugin_id);
        // TODO: Update plugin
    }

    fn show_installed_plugins(&mut self) {
        // TODO: Show installed plugins panel
        tracing::info!("Showing installed plugins");
    }
}

impl Default for MarketplaceUI {
    fn default() -> Self {
        Self::new()
    }
}

/// Format download count (1234 -> "1.2K")
fn format_downloads(count: i64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}
