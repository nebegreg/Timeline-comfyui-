/// Graph Editor - Phase 8: Animation & Keyframing
///
/// Advanced curve editor with interactive Bézier handles for precise
/// keyframe control. Uses egui_plot for professional graph visualization.
use eframe::egui::{self, Color32, Ui};
use egui_plot::{Line, Plot, PlotBounds, PlotPoints, Points};
use timeline::{
    AutomationEngine, AutomationInterpolation, AutomationKeyframe, AutomationLane, Frame,
    KeyframeEasing,
};

/// Graph editor state
pub struct GraphEditorState {
    /// Currently selected keyframe for editing
    pub selected_keyframe: Option<Frame>,

    /// Zoom level
    pub zoom: f32,

    /// Whether to show grid
    pub show_grid: bool,

    /// Whether to show values
    pub show_values: bool,
}

impl Default for GraphEditorState {
    fn default() -> Self {
        Self {
            selected_keyframe: None,
            zoom: 1.0,
            show_grid: true,
            show_values: true,
        }
    }
}

/// Render the graph editor window
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `lane` - The automation lane to edit
/// * `state` - Mutable graph editor state
/// * `current_frame` - Current playhead position
///
/// # Returns
/// True if the lane was modified
pub fn render_graph_editor(
    ui: &mut Ui,
    lane: &mut AutomationLane,
    state: &mut GraphEditorState,
    current_frame: Frame,
) -> bool {
    let mut modified = false;

    // Header with controls
    ui.horizontal(|ui| {
        ui.heading("Curve Editor");
        ui.separator();

        ui.label(format!("Parameter: {}", lane.target.parameter));
        ui.separator();

        // Interpolation mode selector
        ui.label("Interpolation:");
        egui::ComboBox::from_id_salt("graph_interpolation")
            .selected_text(interpolation_name(&lane.interpolation))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        &mut lane.interpolation,
                        AutomationInterpolation::Linear,
                        "Linear",
                    )
                    .clicked()
                {
                    modified = true;
                }
                if ui
                    .selectable_value(
                        &mut lane.interpolation,
                        AutomationInterpolation::Step,
                        "Step",
                    )
                    .clicked()
                {
                    modified = true;
                }
                if ui
                    .selectable_value(
                        &mut lane.interpolation,
                        AutomationInterpolation::Bezier,
                        "Bézier",
                    )
                    .clicked()
                {
                    modified = true;
                }
            });

        ui.separator();

        // Grid toggle
        if ui.checkbox(&mut state.show_grid, "Grid").changed() {
            // State changed
        }

        // Values toggle
        if ui.checkbox(&mut state.show_values, "Values").changed() {
            // State changed
        }
    });

    ui.separator();

    // Main plot area
    let plot_response = render_plot(ui, lane, state, current_frame, &mut modified);

    // Keyframe properties panel (below plot)
    ui.separator();
    ui.collapsing("Keyframe Properties", |ui| {
        render_keyframe_properties(ui, lane, state, &mut modified);
    });

    modified || plot_response
}

/// Render the main plot area
fn render_plot(
    ui: &mut Ui,
    lane: &mut AutomationLane,
    state: &GraphEditorState,
    current_frame: Frame,
    modified: &mut bool,
) -> bool {
    let mut plot_modified = false;

    // Determine plot bounds
    let (frame_min, frame_max) = if let Some((min, max)) = lane.keyframe_range() {
        let padding = ((max - min) as f32 * 0.1) as i64;
        (min - padding.max(10), max + padding.max(10))
    } else {
        (0, 100)
    };

    let value_min = 0.0;
    let value_max = 1.0;

    // Create the plot
    let plot = Plot::new("automation_curve_plot")
        .view_aspect(2.0)
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .show_grid(state.show_grid)
        .show_axes([true, true])
        .legend(Default::default());

    let plot_response = plot.show(ui, |plot_ui| {
        // Draw the automation curve
        if !lane.keyframes.is_empty() {
            let curve_points = generate_curve_points(lane, frame_min, frame_max);
            let curve_line = Line::new(PlotPoints::from(curve_points))
                .color(Color32::from_rgb(100, 180, 255))
                .width(2.0)
                .name("Automation Curve");

            plot_ui.line(curve_line);
        }

        // Draw keyframes as points
        let keyframe_points: Vec<[f64; 2]> = lane
            .keyframes
            .iter()
            .map(|kf| [kf.frame as f64, kf.value])
            .collect();

        if !keyframe_points.is_empty() {
            let points = Points::new(PlotPoints::from(keyframe_points))
                .color(Color32::from_rgb(255, 200, 50))
                .radius(6.0)
                .name("Keyframes");

            plot_ui.points(points);
        }

        // Highlight selected keyframe
        if let Some(selected_frame) = state.selected_keyframe {
            if let Some(kf) = lane.get_keyframe(selected_frame) {
                let selected_point =
                    Points::new(PlotPoints::from(vec![[kf.frame as f64, kf.value]]))
                        .color(Color32::from_rgb(255, 100, 50))
                        .radius(8.0)
                        .name("Selected");

                plot_ui.points(selected_point);
            }
        }

        // Draw playhead line
        let playhead_line = Line::new(PlotPoints::from(vec![
            [current_frame as f64, value_min],
            [current_frame as f64, value_max],
        ]))
        .color(Color32::from_rgb(255, 50, 50))
        .width(2.0)
        .name("Playhead");

        plot_ui.line(playhead_line);

        // Draw zero line
        let zero_line = Line::new(PlotPoints::from(vec![
            [frame_min as f64, 0.0],
            [frame_max as f64, 0.0],
        ]))
        .color(Color32::from_rgba_unmultiplied(255, 255, 255, 50))
        .width(1.0);

        plot_ui.line(zero_line);
    });

    // Handle plot interactions
    if let Some(hover_pos) = plot_response.response.hover_pos() {
        if plot_response.response.clicked() {
            // Find clicked keyframe
            if let Some(pointer_pos) = plot_response
                .transform
                .as_ref()
                .and_then(|t| t.value_from_position(hover_pos))
            {
                let clicked_frame = pointer_pos.x.round() as i64;
                let clicked_value = pointer_pos.y;

                // Check if clicked near a keyframe
                if let Some(nearest) = lane.find_nearest_keyframe(clicked_frame) {
                    if (nearest.frame - clicked_frame).abs() <= 5 {
                        // Close enough to select
                        state.selected_keyframe = Some(nearest.frame);
                        return true;
                    }
                }

                // Not near a keyframe - add new one if Cmd/Ctrl held
                if ui.input(|i| i.modifiers.command) {
                    lane.add_keyframe(AutomationKeyframe {
                        frame: clicked_frame,
                        value: clicked_value.clamp(0.0, 1.0),
                        easing: KeyframeEasing::Linear,
                    });
                    state.selected_keyframe = Some(clicked_frame);
                    *modified = true;
                    plot_modified = true;
                }
            }
        }
    }

    plot_modified
}

/// Generate curve points for visualization
fn generate_curve_points(
    lane: &AutomationLane,
    frame_min: Frame,
    frame_max: Frame,
) -> Vec<[f64; 2]> {
    let mut points = Vec::new();

    // Sample every frame for smooth curve
    for frame in frame_min..=frame_max {
        if let Ok(value) = AutomationEngine::evaluate(lane, frame) {
            points.push([frame as f64, value]);
        }
    }

    points
}

/// Render keyframe properties panel
fn render_keyframe_properties(
    ui: &mut Ui,
    lane: &mut AutomationLane,
    state: &mut GraphEditorState,
    modified: &mut bool,
) {
    if let Some(selected_frame) = state.selected_keyframe {
        if let Some(keyframe) = lane.get_keyframe_mut(selected_frame) {
            ui.horizontal(|ui| {
                ui.label("Frame:");
                ui.label(format!("{}", keyframe.frame));
            });

            ui.horizontal(|ui| {
                ui.label("Value:");
                if ui
                    .add(
                        egui::DragValue::new(&mut keyframe.value)
                            .speed(0.01)
                            .clamp_range(0.0..=1.0)
                            .fixed_decimals(3),
                    )
                    .changed()
                {
                    *modified = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Easing:");
                egui::ComboBox::from_id_salt("keyframe_easing")
                    .selected_text(easing_name(&keyframe.easing))
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut keyframe.easing,
                                KeyframeEasing::Linear,
                                "Linear",
                            )
                            .clicked()
                        {
                            *modified = true;
                        }
                        if ui
                            .selectable_value(
                                &mut keyframe.easing,
                                KeyframeEasing::EaseIn,
                                "Ease In",
                            )
                            .clicked()
                        {
                            *modified = true;
                        }
                        if ui
                            .selectable_value(
                                &mut keyframe.easing,
                                KeyframeEasing::EaseOut,
                                "Ease Out",
                            )
                            .clicked()
                        {
                            *modified = true;
                        }
                        if ui
                            .selectable_value(
                                &mut keyframe.easing,
                                KeyframeEasing::EaseInOut,
                                "Ease In-Out",
                            )
                            .clicked()
                        {
                            *modified = true;
                        }
                    });
            });

            // Custom tangent controls
            if let KeyframeEasing::Custom {
                in_tangent,
                out_tangent,
            } = &mut keyframe.easing
            {
                ui.horizontal(|ui| {
                    ui.label("In Tangent:");
                    if ui.add(egui::Slider::new(in_tangent, 0.0..=1.0)).changed() {
                        *modified = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Out Tangent:");
                    if ui.add(egui::Slider::new(out_tangent, 0.0..=1.0)).changed() {
                        *modified = true;
                    }
                });
            }

            ui.separator();

            // Delete button
            if ui.button("🗑 Delete Keyframe").clicked() {
                lane.remove_keyframe(selected_frame);
                state.selected_keyframe = None;
                *modified = true;
            }
        } else {
            ui.label("Selected keyframe not found");
            state.selected_keyframe = None;
        }
    } else {
        ui.label("No keyframe selected");
        ui.label("Click a keyframe to select it");
        ui.label("Cmd+Click on curve to add keyframe");
    }
}

/// Easing preset buttons
pub fn easing_presets_panel(
    ui: &mut Ui,
    lane: &mut AutomationLane,
    state: &GraphEditorState,
) -> bool {
    let mut modified = false;

    ui.heading("Easing Presets");
    ui.separator();

    if let Some(selected_frame) = state.selected_keyframe {
        if let Some(keyframe) = lane.get_keyframe_mut(selected_frame) {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Linear").clicked() {
                    keyframe.easing = KeyframeEasing::Linear;
                    modified = true;
                }

                if ui.button("Ease In").clicked() {
                    keyframe.easing = KeyframeEasing::EaseIn;
                    modified = true;
                }

                if ui.button("Ease Out").clicked() {
                    keyframe.easing = KeyframeEasing::EaseOut;
                    modified = true;
                }

                if ui.button("Ease In-Out").clicked() {
                    keyframe.easing = KeyframeEasing::EaseInOut;
                    modified = true;
                }

                if ui.button("Custom").clicked() {
                    keyframe.easing = KeyframeEasing::Custom {
                        in_tangent: 0.5,
                        out_tangent: 0.5,
                    };
                    modified = true;
                }
            });
        }
    } else {
        ui.label("Select a keyframe to apply presets");
    }

    modified
}

// Helper functions

fn interpolation_name(interpolation: &AutomationInterpolation) -> &'static str {
    match interpolation {
        AutomationInterpolation::Linear => "Linear",
        AutomationInterpolation::Step => "Step",
        AutomationInterpolation::Bezier => "Bézier",
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
