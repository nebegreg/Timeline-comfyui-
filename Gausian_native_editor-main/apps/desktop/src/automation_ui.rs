/// Automation Lanes UI - Phase 8: Animation & Keyframing
///
/// This module provides UI components for visualizing and editing automation lanes
/// in the timeline, including keyframe manipulation and curve previews.

use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use timeline::{
    AutomationEngine, AutomationInterpolation, AutomationKeyframe, AutomationLane,
    KeyframeEasing, Frame,
};

/// Visual constants for automation lane rendering
const LANE_HEIGHT: f32 = 60.0;
const KEYFRAME_RADIUS: f32 = 4.0;
const KEYFRAME_HOVER_RADIUS: f32 = 6.0;
const CURVE_COLOR: Color32 = Color32::from_rgb(100, 180, 255);
const KEYFRAME_COLOR: Color32 = Color32::from_rgb(255, 200, 50);
const KEYFRAME_SELECTED_COLOR: Color32 = Color32::from_rgb(255, 100, 50);
const GRID_COLOR: Color32 = Color32::from_rgba_unmultiplied(255, 255, 255, 20);

/// State for automation lane editing
#[derive(Default)]
pub struct AutomationLaneState {
    /// Currently selected keyframe (by frame number)
    pub selected_keyframe: Option<Frame>,

    /// Keyframe being dragged
    pub dragging_keyframe: Option<Frame>,

    /// Whether the lane is expanded (showing full curve)
    pub expanded: bool,

    /// Value range for display (min, max)
    pub value_range: (f64, f64),
}

impl AutomationLaneState {
    pub fn new(value_range: (f64, f64)) -> Self {
        Self {
            selected_keyframe: None,
            dragging_keyframe: None,
            expanded: true,
            value_range,
        }
    }
}

/// Render an automation lane in the timeline
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `lane` - The automation lane to render
/// * `state` - Mutable state for this lane
/// * `rect` - The rectangle to render into
/// * `pixels_per_frame` - Zoom level (pixels per frame)
/// * `scroll_offset` - Timeline scroll position in frames
/// * `current_frame` - Current playhead position
///
/// # Returns
/// Response with interaction events
pub fn render_automation_lane(
    ui: &mut Ui,
    lane: &mut AutomationLane,
    state: &mut AutomationLaneState,
    rect: Rect,
    pixels_per_frame: f32,
    scroll_offset: Frame,
    current_frame: Frame,
) -> Response {
    let response = ui.allocate_rect(rect, Sense::click_and_drag());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Draw background
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(30, 30, 30, 200),
        );

        // Draw grid lines (horizontal value indicators)
        draw_value_grid(painter, rect, state.value_range);

        // Draw the automation curve
        if !lane.keyframes.is_empty() {
            draw_automation_curve(
                painter,
                lane,
                rect,
                pixels_per_frame,
                scroll_offset,
                state.value_range,
            );
        }

        // Draw keyframes
        for keyframe in &lane.keyframes {
            let is_selected = state.selected_keyframe == Some(keyframe.frame);
            let is_dragging = state.dragging_keyframe == Some(keyframe.frame);

            draw_keyframe(
                painter,
                keyframe,
                rect,
                pixels_per_frame,
                scroll_offset,
                state.value_range,
                is_selected || is_dragging,
            );
        }

        // Draw playhead position indicator
        if current_frame >= scroll_offset {
            let playhead_x = rect.left() + ((current_frame - scroll_offset) as f32 * pixels_per_frame);
            if playhead_x >= rect.left() && playhead_x <= rect.right() {
                painter.vline(
                    playhead_x,
                    rect.top()..=rect.bottom(),
                    Stroke::new(2.0, Color32::from_rgb(255, 50, 50)),
                );
            }
        }
    }

    // Handle interactions
    if response.clicked() {
        // Check if click is on a keyframe
        if let Some(frame) = find_keyframe_at_pos(
            response.hover_pos().unwrap(),
            lane,
            rect,
            pixels_per_frame,
            scroll_offset,
            state.value_range,
        ) {
            state.selected_keyframe = Some(frame);
        } else if response.ctx.input(|i| i.modifiers.command) {
            // Cmd+Click to add keyframe
            if let Some(pos) = response.hover_pos() {
                let (frame, value) = screen_to_lane_coords(
                    pos,
                    rect,
                    pixels_per_frame,
                    scroll_offset,
                    state.value_range,
                );

                lane.add_keyframe(AutomationKeyframe {
                    frame,
                    value,
                    easing: KeyframeEasing::Linear,
                });
                state.selected_keyframe = Some(frame);
            }
        } else {
            // Click on empty area - deselect
            state.selected_keyframe = None;
        }
    }

    // Handle dragging
    if response.dragged() {
        if let Some(selected_frame) = state.selected_keyframe {
            if let Some(pos) = response.hover_pos() {
                state.dragging_keyframe = Some(selected_frame);

                // Update keyframe position and value
                let (new_frame, new_value) = screen_to_lane_coords(
                    pos,
                    rect,
                    pixels_per_frame,
                    scroll_offset,
                    state.value_range,
                );

                // Remove old keyframe and add at new position
                if let Some(mut kf) = lane.remove_keyframe(selected_frame) {
                    kf.frame = new_frame;
                    kf.value = new_value.clamp(state.value_range.0, state.value_range.1);
                    lane.add_keyframe(kf);
                    state.selected_keyframe = Some(new_frame);
                }
            }
        }
    } else {
        state.dragging_keyframe = None;
    }

    // Handle delete key
    if response.has_focus() {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                if let Some(selected_frame) = state.selected_keyframe {
                    lane.remove_keyframe(selected_frame);
                    state.selected_keyframe = None;
                }
            }
        });
    }

    response
}

/// Draw the value grid (horizontal lines)
fn draw_value_grid(painter: &egui::Painter, rect: Rect, value_range: (f64, f64)) {
    let num_lines = 5;
    for i in 0..=num_lines {
        let t = i as f32 / num_lines as f32;
        let y = rect.top() + (rect.height() * (1.0 - t)); // Invert Y axis

        painter.hline(
            rect.left()..=rect.right(),
            y,
            Stroke::new(1.0, GRID_COLOR),
        );

        // Draw value label
        let value = value_range.0 + (value_range.1 - value_range.0) * t as f64;
        painter.text(
            Pos2::new(rect.left() + 5.0, y - 8.0),
            egui::Align2::LEFT_TOP,
            format!("{:.2}", value),
            egui::FontId::proportional(10.0),
            Color32::from_rgba_unmultiplied(200, 200, 200, 150),
        );
    }
}

/// Draw the automation curve between keyframes
fn draw_automation_curve(
    painter: &egui::Painter,
    lane: &AutomationLane,
    rect: Rect,
    pixels_per_frame: f32,
    scroll_offset: Frame,
    value_range: (f64, f64),
) {
    if lane.keyframes.len() < 2 {
        return;
    }

    // Get frame range
    let (min_frame, max_frame) = lane.keyframe_range().unwrap();
    let visible_start = scroll_offset.max(min_frame);
    let visible_end = (scroll_offset + (rect.width() / pixels_per_frame) as i64).min(max_frame);

    // Sample points along the curve
    let mut points = Vec::new();
    let step = (1.0 / pixels_per_frame).max(1.0) as i64; // At least 1 frame per step

    for frame in (visible_start..=visible_end).step_by(step.max(1) as usize) {
        if let Ok(value) = AutomationEngine::evaluate(lane, frame) {
            let screen_pos = lane_coords_to_screen(
                frame,
                value,
                rect,
                pixels_per_frame,
                scroll_offset,
                value_range,
            );
            points.push(screen_pos);
        }
    }

    // Draw curve as polyline
    if points.len() >= 2 {
        painter.add(egui::Shape::line(
            points,
            Stroke::new(2.0, CURVE_COLOR),
        ));
    }
}

/// Draw a single keyframe
fn draw_keyframe(
    painter: &egui::Painter,
    keyframe: &AutomationKeyframe,
    rect: Rect,
    pixels_per_frame: f32,
    scroll_offset: Frame,
    value_range: (f64, f64),
    selected: bool,
) {
    let pos = lane_coords_to_screen(
        keyframe.frame,
        keyframe.value,
        rect,
        pixels_per_frame,
        scroll_offset,
        value_range,
    );

    // Only draw if visible
    if pos.x < rect.left() || pos.x > rect.right() {
        return;
    }

    let radius = if selected { KEYFRAME_HOVER_RADIUS } else { KEYFRAME_RADIUS };
    let color = if selected { KEYFRAME_SELECTED_COLOR } else { KEYFRAME_COLOR };

    // Draw outer circle (white)
    painter.circle_filled(pos, radius + 1.0, Color32::WHITE);

    // Draw inner circle (colored)
    painter.circle_filled(pos, radius, color);

    // Draw easing indicator (small icon)
    let icon = match keyframe.easing {
        KeyframeEasing::Linear => "",
        KeyframeEasing::EaseIn => "◣",
        KeyframeEasing::EaseOut => "◤",
        KeyframeEasing::EaseInOut => "◆",
        KeyframeEasing::Custom { .. } => "✦",
    };

    if !icon.is_empty() {
        painter.text(
            Pos2::new(pos.x + radius + 3.0, pos.y - 6.0),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
}

/// Convert screen position to lane coordinates (frame, value)
fn screen_to_lane_coords(
    screen_pos: Pos2,
    rect: Rect,
    pixels_per_frame: f32,
    scroll_offset: Frame,
    value_range: (f64, f64),
) -> (Frame, f64) {
    // Calculate frame
    let x_offset = screen_pos.x - rect.left();
    let frame = scroll_offset + (x_offset / pixels_per_frame) as i64;

    // Calculate value (inverted Y axis)
    let y_normalized = 1.0 - ((screen_pos.y - rect.top()) / rect.height());
    let value = value_range.0 + (value_range.1 - value_range.0) * y_normalized as f64;

    (frame, value)
}

/// Convert lane coordinates to screen position
fn lane_coords_to_screen(
    frame: Frame,
    value: f64,
    rect: Rect,
    pixels_per_frame: f32,
    scroll_offset: Frame,
    value_range: (f64, f64),
) -> Pos2 {
    // Calculate X position
    let x = rect.left() + ((frame - scroll_offset) as f32 * pixels_per_frame);

    // Calculate Y position (inverted)
    let value_normalized = ((value - value_range.0) / (value_range.1 - value_range.0)) as f32;
    let y = rect.top() + (rect.height() * (1.0 - value_normalized));

    Pos2::new(x, y)
}

/// Find keyframe at screen position (with tolerance)
fn find_keyframe_at_pos(
    screen_pos: Pos2,
    lane: &AutomationLane,
    rect: Rect,
    pixels_per_frame: f32,
    scroll_offset: Frame,
    value_range: (f64, f64),
) -> Option<Frame> {
    const CLICK_TOLERANCE: f32 = 8.0;

    for keyframe in &lane.keyframes {
        let kf_pos = lane_coords_to_screen(
            keyframe.frame,
            keyframe.value,
            rect,
            pixels_per_frame,
            scroll_offset,
            value_range,
        );

        let distance = screen_pos.distance(kf_pos);
        if distance <= CLICK_TOLERANCE {
            return Some(keyframe.frame);
        }
    }

    None
}

/// Keyframe inspector panel (for editing selected keyframe properties)
pub fn keyframe_inspector(
    ui: &mut Ui,
    lane: &mut AutomationLane,
    selected_keyframe: Option<Frame>,
) {
    ui.heading("Keyframe Properties");

    if let Some(frame) = selected_keyframe {
        if let Some(keyframe) = lane.get_keyframe_mut(frame) {
            ui.separator();

            // Frame position
            ui.horizontal(|ui| {
                ui.label("Frame:");
                ui.label(format!("{}", keyframe.frame));
            });

            // Value slider
            ui.horizontal(|ui| {
                ui.label("Value:");
                ui.add(egui::DragValue::new(&mut keyframe.value)
                    .speed(0.01)
                    .clamp_range(0.0..=1.0));
            });

            // Easing selector
            ui.horizontal(|ui| {
                ui.label("Easing:");
                egui::ComboBox::from_id_salt("easing_selector")
                    .selected_text(easing_name(&keyframe.easing))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut keyframe.easing, KeyframeEasing::Linear, "Linear");
                        ui.selectable_value(&mut keyframe.easing, KeyframeEasing::EaseIn, "Ease In");
                        ui.selectable_value(&mut keyframe.easing, KeyframeEasing::EaseOut, "Ease Out");
                        ui.selectable_value(&mut keyframe.easing, KeyframeEasing::EaseInOut, "Ease In-Out");
                    });
            });

            // Custom tangents (if custom easing)
            if let KeyframeEasing::Custom { in_tangent, out_tangent } = &mut keyframe.easing {
                ui.horizontal(|ui| {
                    ui.label("In Tangent:");
                    ui.add(egui::Slider::new(in_tangent, 0.0..=1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Out Tangent:");
                    ui.add(egui::Slider::new(out_tangent, 0.0..=1.0));
                });
            }

            ui.separator();

            // Interpolation method (for the lane)
            ui.horizontal(|ui| {
                ui.label("Interpolation:");
                egui::ComboBox::from_id_salt("interpolation_selector")
                    .selected_text(interpolation_name(&lane.interpolation))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut lane.interpolation, AutomationInterpolation::Linear, "Linear");
                        ui.selectable_value(&mut lane.interpolation, AutomationInterpolation::Step, "Step");
                        ui.selectable_value(&mut lane.interpolation, AutomationInterpolation::Bezier, "Bézier");
                    });
            });
        } else {
            ui.label("No keyframe selected");
        }
    } else {
        ui.label("No keyframe selected");
        ui.separator();
        ui.label("Cmd+Click on the automation lane to add a keyframe");
        ui.label("Click a keyframe to select it");
        ui.label("Drag a keyframe to move it");
        ui.label("Delete/Backspace to remove selected keyframe");
    }
}

fn easing_name(easing: &KeyframeEasing) -> &'static str {
    match easing {
        KeyframeEasing::Linear => "Linear",
        KeyframeEasing::EaseIn => "Ease In",
        KeyframeEasing::EaseOut => "Ease Out",
        KeyframeEasing::EaseInOut => "Ease In-Out",
        KeyframeEasing::Custom { .. } => "Custom",
    }
}

fn interpolation_name(interpolation: &AutomationInterpolation) -> &'static str {
    match interpolation {
        AutomationInterpolation::Linear => "Linear",
        AutomationInterpolation::Step => "Step",
        AutomationInterpolation::Bezier => "Bézier",
    }
}

/// Toolbar for automation lane actions
pub fn automation_toolbar(ui: &mut Ui, lane: &mut AutomationLane, state: &AutomationLaneState) {
    ui.horizontal(|ui| {
        // Expand/collapse button
        let expand_icon = if state.expanded { "▼" } else { "▶" };
        if ui.button(expand_icon).clicked() {
            // Toggle handled by caller
        }

        // Lane label
        ui.label(format!("Automation: {}", lane.target.parameter));

        ui.separator();

        // Clear all keyframes
        if ui.button("Clear All").clicked() {
            lane.clear_keyframes();
        }

        // Keyframe count
        ui.label(format!("{} keyframes", lane.keyframes.len()));
    });
}
