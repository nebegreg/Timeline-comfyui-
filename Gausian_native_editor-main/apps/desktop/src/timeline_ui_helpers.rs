/// Timeline UI helpers for Phase 1: Selection & Markers visualization
/// Companion to timeline/ui.rs

use eframe::egui;
use timeline::{Marker, MarkerType, NodeId};
use crate::selection::SelectionState;
use crate::edit_modes::EditMode;

/// Visual style for selected clips
pub struct SelectionStyle {
    pub outline_color: egui::Color32,
    pub outline_width: f32,
    pub fill_tint: egui::Color32,
}

impl Default for SelectionStyle {
    fn default() -> Self {
        Self {
            outline_color: egui::Color32::from_rgb(100, 150, 255), // Blue
            outline_width: 2.0,
            fill_tint: egui::Color32::from_rgba_premultiplied(100, 150, 255, 30),
        }
    }
}

/// Primary selection style (brighter)
pub struct PrimarySelectionStyle {
    pub outline_color: egui::Color32,
    pub outline_width: f32,
}

impl Default for PrimarySelectionStyle {
    fn default() -> Self {
        Self {
            outline_color: egui::Color32::from_rgb(255, 200, 50), // Gold
            outline_width: 3.0,
        }
    }
}

/// Draw selection outline around a clip rect
pub fn draw_selection_outline(
    painter: &egui::Painter,
    rect: egui::Rect,
    is_primary: bool,
) {
    let style = if is_primary {
        let primary = PrimarySelectionStyle::default();
        (primary.outline_color, primary.outline_width)
    } else {
        let normal = SelectionStyle::default();
        (normal.outline_color, normal.outline_width)
    };

    // Draw outline
    painter.rect_stroke(
        rect,
        2.0, // Corner radius
        egui::Stroke::new(style.1, style.0),
    );

    // Draw subtle fill tint for non-primary
    if !is_primary {
        painter.rect_filled(
            rect,
            2.0,
            SelectionStyle::default().fill_tint,
        );
    }
}

/// Draw rectangle selection box (drag-to-select)
pub fn draw_rect_selection(
    painter: &egui::Painter,
    rect: egui::Rect,
) {
    // Dashed outline
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 150, 255)),
    );

    // Semi-transparent fill
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_premultiplied(100, 150, 255, 20),
    );
}

/// Marker visualization style
#[derive(Clone)]
pub struct MarkerStyle {
    pub color: egui::Color32,
    pub height: f32,
    pub label_bg: egui::Color32,
}

impl MarkerStyle {
    pub fn from_marker_type(marker_type: MarkerType) -> Self {
        match marker_type {
            MarkerType::In => Self {
                color: egui::Color32::from_rgb(0, 255, 0), // Green
                height: 20.0,
                label_bg: egui::Color32::from_rgba_premultiplied(0, 255, 0, 180),
            },
            MarkerType::Out => Self {
                color: egui::Color32::from_rgb(255, 0, 0), // Red
                height: 20.0,
                label_bg: egui::Color32::from_rgba_premultiplied(255, 0, 0, 180),
            },
            MarkerType::Chapter => Self {
                color: egui::Color32::from_rgb(255, 0, 255), // Magenta
                height: 16.0,
                label_bg: egui::Color32::from_rgba_premultiplied(255, 0, 255, 180),
            },
            MarkerType::Comment => Self {
                color: egui::Color32::from_rgb(255, 255, 0), // Yellow
                height: 14.0,
                label_bg: egui::Color32::from_rgba_premultiplied(255, 255, 0, 180),
            },
            MarkerType::Todo => Self {
                color: egui::Color32::from_rgb(255, 165, 0), // Orange
                height: 14.0,
                label_bg: egui::Color32::from_rgba_premultiplied(255, 165, 0, 180),
            },
            MarkerType::Standard => Self {
                color: egui::Color32::from_rgb(74, 158, 255), // Blue
                height: 12.0,
                label_bg: egui::Color32::from_rgba_premultiplied(74, 158, 255, 180),
            },
        }
    }

    pub fn from_hex(hex: &str) -> Self {
        let color = parse_hex_color(hex).unwrap_or(egui::Color32::from_rgb(74, 158, 255));
        Self {
            color,
            height: 12.0,
            label_bg: egui::Color32::from_rgba_premultiplied(
                color.r(),
                color.g(),
                color.b(),
                180,
            ),
        }
    }
}

/// Draw a marker on timeline
pub fn draw_marker(
    painter: &egui::Painter,
    marker: &Marker,
    x_pos: f32,
    y_top: f32,
    y_bottom: f32,
    show_label: bool,
) {
    let style = if marker.color.starts_with('#') {
        MarkerStyle::from_hex(&marker.color)
    } else {
        MarkerStyle::from_marker_type(marker.marker_type)
    };

    // Vertical line
    painter.line_segment(
        [
            egui::pos2(x_pos, y_top),
            egui::pos2(x_pos, y_bottom),
        ],
        egui::Stroke::new(2.0, style.color),
    );

    // Triangle at top
    let triangle_height = 8.0;
    let triangle_width = 6.0;
    let triangle = vec![
        egui::pos2(x_pos, y_top),
        egui::pos2(x_pos - triangle_width / 2.0, y_top + triangle_height),
        egui::pos2(x_pos + triangle_width / 2.0, y_top + triangle_height),
    ];
    painter.add(egui::Shape::convex_polygon(
        triangle,
        style.color,
        egui::Stroke::NONE,
    ));

    // Label (if enabled and there's space)
    if show_label && !marker.label.is_empty() {
        let label_pos = egui::pos2(x_pos + 5.0, y_top);
        let galley = painter.layout_no_wrap(
            marker.label.clone(),
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );

        // Background
        let label_rect = egui::Rect::from_min_size(
            label_pos,
            galley.size() + egui::vec2(4.0, 2.0),
        );
        painter.rect_filled(label_rect, 2.0, style.label_bg);

        // Text
        painter.galley(label_pos + egui::vec2(2.0, 1.0), galley, egui::Color32::WHITE);
    }
}

/// Draw region (in/out range)
pub fn draw_region(
    painter: &egui::Painter,
    start_x: f32,
    end_x: f32,
    y_top: f32,
    y_bottom: f32,
    color_hex: &str,
) {
    let color = parse_hex_color(color_hex)
        .unwrap_or(egui::Color32::from_rgba_premultiplied(74, 158, 255, 40));

    let rect = egui::Rect::from_x_y_ranges(start_x..=end_x, y_top..=y_bottom);

    // Semi-transparent fill
    painter.rect_filled(rect, 0.0, color);

    // Border
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(
            color.r(),
            color.g(),
            color.b(),
            255,
        )),
    );
}

/// Parse hex color string (#RRGGBB or #RRGGBBAA)
fn parse_hex_color(hex: &str) -> Option<egui::Color32> {
    let hex = hex.trim_start_matches('#');

    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(egui::Color32::from_rgb(r, g, b))
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        Some(egui::Color32::from_rgba_premultiplied(r, g, b, a))
    } else {
        None
    }
}

/// Edit mode toolbar button
pub fn edit_mode_button(
    ui: &mut egui::Ui,
    current_mode: EditMode,
    mode: EditMode,
) -> bool {
    let is_active = current_mode == mode;

    let button = if is_active {
        egui::Button::new(egui::RichText::new(mode.name()).strong())
            .fill(egui::Color32::from_rgb(60, 120, 180))
    } else {
        egui::Button::new(mode.name())
    };

    let response = ui.add(button);

    if response.hovered() {
        response.on_hover_text(format!(
            "{}\nShortcut: {}\n\n{}",
            mode.name(),
            mode.shortcut(),
            mode.description()
        ));
    }

    response.clicked()
}

/// Selection info text (for status bar)
pub fn selection_info_text(selection: &SelectionState) -> String {
    match selection.count() {
        0 => "No selection".to_string(),
        1 => "1 clip selected".to_string(),
        n => format!("{} clips selected", n),
    }
}

/// Snap indicator (visual feedback when snapping occurs)
pub fn draw_snap_indicator(
    painter: &egui::Painter,
    x_pos: f32,
    y_range: std::ops::RangeInclusive<f32>,
) {
    // Dashed vertical line
    let dash_length = 4.0;
    let gap_length = 3.0;
    let mut y = *y_range.start();

    while y < *y_range.end() {
        let next_y = (y + dash_length).min(*y_range.end());
        painter.line_segment(
            [egui::pos2(x_pos, y), egui::pos2(x_pos, next_y)],
            egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 255, 100)),
        );
        y = next_y + gap_length;
    }
}

/// Keyboard shortcuts help panel
pub fn keyboard_help_panel(ui: &mut egui::Ui) {
    ui.heading("⌨️ Keyboard Shortcuts");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (category, commands) in crate::keyboard::KeyCommand::all_commands() {
            ui.label(egui::RichText::new(category).strong());
            ui.indent(category, |ui| {
                for cmd in commands {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(cmd.shortcut_text())
                            .monospace()
                            .color(egui::Color32::from_rgb(100, 200, 255)));
                        ui.label("—");
                        ui.label(cmd.description());
                    });
                }
            });
            ui.add_space(8.0);
        }
    });
}
