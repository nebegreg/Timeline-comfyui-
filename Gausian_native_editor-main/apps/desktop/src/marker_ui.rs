/// Marker management UI
/// Phase 1: Timeline Polish & UX
use eframe::egui;
use timeline::{Marker, MarkerCollection, MarkerId, MarkerType, Region};

/// Marker panel UI
pub struct MarkerPanel {
    pub selected_marker: Option<MarkerId>,
    pub new_marker_label: String,
    pub new_marker_type: MarkerType,
}

impl Default for MarkerPanel {
    fn default() -> Self {
        Self {
            selected_marker: None,
            new_marker_label: String::new(),
            new_marker_type: MarkerType::Standard,
        }
    }
}

impl MarkerPanel {
    /// Draw marker list panel
    pub fn ui(&mut self, ui: &mut egui::Ui, markers: &mut MarkerCollection, current_frame: i64) {
        ui.heading("Markers");
        ui.separator();

        // Quick add marker at playhead
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_marker_label);
            if ui.button("➕ Add at Playhead").clicked() {
                let label = if self.new_marker_label.is_empty() {
                    format!("Marker {}", markers.markers().count() + 1)
                } else {
                    self.new_marker_label.clone()
                };

                let marker = Marker::new(current_frame, label).with_type(self.new_marker_type);
                markers.add_marker(marker);
                self.new_marker_label.clear();
            }
        });

        // Marker type selector
        ui.horizontal(|ui| {
            ui.label("Type:");
            egui::ComboBox::from_id_salt("marker_type")
                .selected_text(format!("{:?}", self.new_marker_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.new_marker_type,
                        MarkerType::Standard,
                        "Standard",
                    );
                    ui.selectable_value(&mut self.new_marker_type, MarkerType::Chapter, "Chapter");
                    ui.selectable_value(&mut self.new_marker_type, MarkerType::Comment, "Comment");
                    ui.selectable_value(&mut self.new_marker_type, MarkerType::Todo, "TODO");
                });
        });

        ui.separator();

        // In/Out controls
        ui.horizontal(|ui| {
            if ui.button("Set In (I)").clicked() {
                markers.set_in_point(current_frame);
            }
            if ui.button("Set Out (O)").clicked() {
                markers.set_out_point(current_frame);
            }
            if ui.button("Clear In/Out").clicked() {
                markers.clear_in_out();
            }
        });

        if let Some((in_frame, out_frame)) = markers.get_in_out_range() {
            ui.label(format!(
                "📍 In: {} | Out: {} (Duration: {})",
                in_frame,
                out_frame,
                out_frame - in_frame
            ));
        }

        ui.separator();

        // Marker list
        ui.heading("All Markers");
        egui::ScrollArea::vertical().show(ui, |ui| {
            let sorted_markers = markers.markers_sorted();

            for marker in sorted_markers {
                let is_selected = self.selected_marker == Some(marker.id);

                ui.horizontal(|ui| {
                    let response = ui.selectable_label(
                        is_selected,
                        format!(
                            "{}  {} @ {}",
                            marker_type_icon(marker.marker_type),
                            marker.label,
                            marker.frame
                        ),
                    );

                    if response.clicked() {
                        self.selected_marker = Some(marker.id);
                    }

                    // Delete button
                    if ui.small_button("🗑").clicked() {
                        markers.remove_marker(&marker.id);
                        if self.selected_marker == Some(marker.id) {
                            self.selected_marker = None;
                        }
                    }
                });

                // Show note if present
                if !marker.note.is_empty() {
                    ui.indent(marker.id, |ui| {
                        ui.label(egui::RichText::new(&marker.note).italics().weak());
                    });
                }
            }

            if sorted_markers.is_empty() {
                ui.label(egui::RichText::new("No markers yet").weak().italics());
            }
        });
    }

    /// Draw marker editor (for selected marker)
    pub fn marker_editor(&mut self, ui: &mut egui::Ui, markers: &mut MarkerCollection) {
        if let Some(marker_id) = self.selected_marker {
            if let Some(marker) = markers.get_marker_mut(&marker_id) {
                ui.heading("Edit Marker");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Label:");
                    ui.text_edit_singleline(&mut marker.label);
                });

                ui.horizontal(|ui| {
                    ui.label("Frame:");
                    ui.add(egui::DragValue::new(&mut marker.frame));
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("edit_marker_type")
                        .selected_text(format!("{:?}", marker.marker_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut marker.marker_type,
                                MarkerType::Standard,
                                "Standard",
                            );
                            ui.selectable_value(
                                &mut marker.marker_type,
                                MarkerType::In,
                                "In Point",
                            );
                            ui.selectable_value(
                                &mut marker.marker_type,
                                MarkerType::Out,
                                "Out Point",
                            );
                            ui.selectable_value(
                                &mut marker.marker_type,
                                MarkerType::Chapter,
                                "Chapter",
                            );
                            ui.selectable_value(
                                &mut marker.marker_type,
                                MarkerType::Comment,
                                "Comment",
                            );
                            ui.selectable_value(&mut marker.marker_type, MarkerType::Todo, "TODO");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.text_edit_singleline(&mut marker.color);
                    // Color picker would go here
                });

                ui.label("Note:");
                ui.text_edit_multiline(&mut marker.note);
            }
        } else {
            ui.label(egui::RichText::new("No marker selected").weak().italics());
        }
    }
}

/// Get icon for marker type
fn marker_type_icon(marker_type: MarkerType) -> &'static str {
    match marker_type {
        MarkerType::In => "▶",
        MarkerType::Out => "⏹",
        MarkerType::Chapter => "📖",
        MarkerType::Comment => "💬",
        MarkerType::Todo => "✓",
        MarkerType::Standard => "📍",
    }
}

/// Region panel UI
pub struct RegionPanel {
    pub new_region_label: String,
}

impl Default for RegionPanel {
    fn default() -> Self {
        Self {
            new_region_label: String::new(),
        }
    }
}

impl RegionPanel {
    /// Draw region list
    pub fn ui(&mut self, ui: &mut egui::Ui, markers: &mut MarkerCollection) {
        ui.heading("Regions");
        ui.separator();

        // Add region from In/Out
        if let Some((in_frame, out_frame)) = markers.get_in_out_range() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_region_label);
                if ui.button("Create Region from In/Out").clicked() {
                    let label = if self.new_region_label.is_empty() {
                        format!("Region {}", markers.regions().count() + 1)
                    } else {
                        self.new_region_label.clone()
                    };

                    let region = Region::new(in_frame, out_frame, label);
                    markers.add_region(region);
                    self.new_region_label.clear();
                }
            });
        } else {
            ui.label(
                egui::RichText::new("Set In/Out points to create a region")
                    .weak()
                    .italics(),
            );
        }

        ui.separator();

        // Region list
        egui::ScrollArea::vertical().show(ui, |ui| {
            let regions: Vec<_> = markers.regions().collect();

            for region in regions {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "📏 {} ({} → {} = {} frames)",
                        region.label,
                        region.start,
                        region.end,
                        region.duration()
                    ));

                    if ui.small_button("🗑").clicked() {
                        markers.remove_region(&region.id);
                    }
                });

                if !region.note.is_empty() {
                    ui.indent(region.id, |ui| {
                        ui.label(egui::RichText::new(&region.note).italics().weak());
                    });
                }
            }

            if regions.is_empty() {
                ui.label(egui::RichText::new("No regions yet").weak().italics());
            }
        });
    }
}
