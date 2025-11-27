/// Plugin Details Panel
/// Shows detailed information about a selected plugin
use crate::marketplace_ui::MarketplacePlugin;
use egui::{Color32, Context, RichText, ScrollArea, Ui, Window};

/// Plugin details panel state
pub struct PluginDetailsPanel {
    pub show: bool,
    pub plugin: Option<MarketplacePlugin>,
}

impl PluginDetailsPanel {
    pub fn new() -> Self {
        Self {
            show: false,
            plugin: None,
        }
    }

    pub fn show_plugin(&mut self, plugin: MarketplacePlugin) {
        self.plugin = Some(plugin);
        self.show = true;
    }

    pub fn render(&mut self, ctx: &Context) -> Option<PluginAction> {
        if !self.show {
            return None;
        }

        let mut action = None;

        Window::new("Plugin Details")
            .default_width(600.0)
            .default_height(700.0)
            .resizable(true)
            .open(&mut self.show)
            .show(ctx, |ui| {
                if let Some(plugin) = &self.plugin {
                    action = self.render_plugin_details(ui, plugin);
                }
            });

        action
    }

    fn render_plugin_details(
        &mut self,
        ui: &mut Ui,
        plugin: &MarketplacePlugin,
    ) -> Option<PluginAction> {
        let mut action = None;

        // Header
        ui.horizontal(|ui| {
            ui.heading(&plugin.name);
            ui.add_space(10.0);
            if plugin.verified {
                ui.label(RichText::new("✓ Verified").color(Color32::from_rgb(0, 150, 255)));
            }
        });

        ui.label(RichText::new(format!("Version {}", plugin.version)).italics());

        ui.add_space(10.0);

        // Author and license
        ui.horizontal(|ui| {
            ui.label(format!("👤 By {}", plugin.author));
            ui.add_space(20.0);
            ui.label(format!("📄 {}", plugin.license));
        });

        ui.separator();

        // Short description
        ui.label(&plugin.description);

        ui.add_space(10.0);

        // Stats row
        ui.horizontal(|ui| {
            // Rating
            egui::Frame::group(ui.style())
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(format!("{:.1}", plugin.rating));
                        ui.label(RichText::new("⭐⭐⭐⭐⭐").small());
                        ui.label(RichText::new(format!("{} ratings", plugin.rating_count)).small());
                    });
                });

            ui.add_space(10.0);

            // Downloads
            egui::Frame::group(ui.style())
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(format_number(plugin.downloads));
                        ui.label(RichText::new("📥").small());
                        ui.label(RichText::new("downloads").small());
                    });
                });

            ui.add_space(10.0);

            // File size
            egui::Frame::group(ui.style())
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(format_bytes(plugin.file_size));
                        ui.label(RichText::new("💾").small());
                        ui.label(RichText::new("size").small());
                    });
                });
        });

        ui.separator();

        // Badges
        ui.horizontal(|ui| {
            ui.label("Category:");
            ui.label(RichText::new(&plugin.category).background_color(Color32::from_gray(60)));

            ui.add_space(10.0);

            ui.label("Type:");
            ui.label(RichText::new(&plugin.plugin_type).background_color(Color32::from_gray(80)));
        });

        ui.add_space(5.0);

        // Tags
        if !plugin.tags.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label("Tags:");
                for tag in plugin.tags.split(',') {
                    ui.label(
                        RichText::new(format!("#{}", tag.trim()))
                            .small()
                            .color(Color32::from_rgb(100, 150, 255)),
                    );
                }
            });
        }

        ui.separator();

        // Long description
        if let Some(long_desc) = &plugin.long_description {
            ui.heading("Description");
            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                ui.label(long_desc);
            });

            ui.separator();
        }

        // Action buttons
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("📥 Install Plugin").size(16.0))
                .clicked()
            {
                action = Some(PluginAction::Install(plugin.id.clone()));
            }

            ui.add_space(10.0);

            if ui.button("View Source").clicked() {
                // Open repository URL in browser if available
                if let Some(ref homepage) = plugin.long_description {
                    // Try to extract URL from long_description or use a default
                    // For now, we'll try to open using the system's default browser
                    if let Err(e) = open::that(format!("https://github.com/{}", plugin.author)) {
                        tracing::error!("Failed to open repository URL: {}", e);
                    }
                } else {
                    tracing::warn!("No repository URL available for plugin");
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    self.show = false;
                }
            });
        });

        action
    }
}

impl Default for PluginDetailsPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered from plugin details
pub enum PluginAction {
    Install(String),
    Update(String),
    Uninstall(String),
}

fn format_number(num: i64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

fn format_bytes(bytes: i64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} B", bytes)
    }
}
