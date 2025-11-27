use crate::edit_modes::{EditMode, SnapSettings};
use crate::keyboard::PlaybackSpeed;
use crate::timeline_ui_helpers::edit_mode_button;
/// Timeline toolbar UI - Edit modes, snap settings, playback controls
/// Phase 1: Timeline Polish & UX
use eframe::egui;

/// Timeline toolbar state
pub struct TimelineToolbar {
    pub show_keyboard_help: bool,
}

impl Default for TimelineToolbar {
    fn default() -> Self {
        Self {
            show_keyboard_help: false,
        }
    }
}

impl TimelineToolbar {
    /// Draw the toolbar
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        edit_mode: &mut EditMode,
        snap_settings: &mut SnapSettings,
        playback_speed: &PlaybackSpeed,
    ) {
        ui.horizontal(|ui| {
            // Edit mode buttons
            ui.label("Edit Mode:");

            for mode in EditMode::all() {
                if edit_mode_button(ui, *edit_mode, mode) {
                    *edit_mode = mode;
                }
            }

            ui.separator();

            // Snap toggle
            let snap_text = if snap_settings.enabled {
                "🧲 Snap: ON"
            } else {
                "Snap: OFF"
            };
            if ui.button(snap_text).clicked() {
                snap_settings.toggle();
            }

            if snap_settings.enabled {
                ui.label(format!("({}px)", snap_settings.snap_tolerance as i32));
            }

            ui.separator();

            // Playback speed indicator (J/K/L)
            if !playback_speed.is_paused() {
                let speed_text = if playback_speed.reverse {
                    format!("◀◀ {}x", playback_speed.speed)
                } else {
                    format!("▶▶ {}x", playback_speed.speed)
                };
                ui.label(
                    egui::RichText::new(speed_text)
                        .color(egui::Color32::from_rgb(100, 255, 100))
                        .strong(),
                );
            }

            ui.separator();

            // Keyboard help button
            if ui.button("⌨️ Shortcuts").clicked() {
                self.show_keyboard_help = !self.show_keyboard_help;
            }
        });
    }

    /// Draw snap settings panel (if opened)
    pub fn snap_settings_panel(
        &self,
        ctx: &egui::Context,
        snap_settings: &mut SnapSettings,
        open: &mut bool,
    ) {
        egui::Window::new("⚙️ Snap Settings")
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.checkbox(&mut snap_settings.enabled, "Enable Snapping");

                ui.separator();

                ui.checkbox(&mut snap_settings.snap_to_playhead, "Snap to Playhead");
                ui.checkbox(&mut snap_settings.snap_to_clips, "Snap to Clip Edges");
                ui.checkbox(&mut snap_settings.snap_to_markers, "Snap to Markers");
                ui.checkbox(
                    &mut snap_settings.snap_to_seconds,
                    "Snap to Second Boundaries",
                );

                ui.separator();

                ui.label("Snap Tolerance:");
                ui.add(
                    egui::Slider::new(&mut snap_settings.snap_tolerance, 1.0..=20.0).suffix(" px"),
                );
            });
    }
}

/// Playback controls toolbar (separate from edit toolbar)
pub struct PlaybackToolbar;

impl PlaybackToolbar {
    pub fn ui(ui: &mut egui::Ui, playing: &mut bool, playhead: i64, fps: timeline::Fps) {
        ui.horizontal(|ui| {
            // Play/Pause button
            let play_text = if *playing { "⏸" } else { "▶" };
            if ui
                .button(egui::RichText::new(play_text).size(20.0))
                .clicked()
            {
                *playing = !*playing;
            }

            ui.separator();

            // Timecode display with DF/NDF indicator (Phase 1)
            let timecode = frame_to_timecode(playhead, fps);

            // Determine if drop-frame based on frame rate
            // Drop-frame is used for 29.97 fps and 59.94 fps
            let is_drop_frame = {
                let fps_value = fps.num as f64 / fps.den.max(1) as f64;
                (fps_value - 29.97).abs() < 0.01 || (fps_value - 59.94).abs() < 0.01
            };

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(timecode).monospace().size(14.0));
                ui.label(
                    egui::RichText::new(if is_drop_frame { "DF" } else { "NDF" })
                        .small()
                        .weak(),
                );
            });

            ui.separator();

            // Frame number
            ui.label(egui::RichText::new(format!("Frame: {}", playhead)).weak());
        });
    }
}

/// Convert frame number to timecode (HH:MM:SS:FF)
fn frame_to_timecode(frame: i64, fps: timeline::Fps) -> String {
    if frame < 0 {
        return "-00:00:00:00".to_string();
    }

    let fps_f = fps.num as f64 / fps.den.max(1) as f64;
    let total_seconds = frame as f64 / fps_f;

    let hours = (total_seconds / 3600.0) as i64;
    let minutes = ((total_seconds % 3600.0) / 60.0) as i64;
    let seconds = (total_seconds % 60.0) as i64;
    let frames = (frame % fps.num as i64).max(0);

    format!("{:02}:{:02}:{:02}:{:02}", hours, minutes, seconds, frames)
}

/// Status bar at bottom of timeline
pub struct StatusBar;

impl StatusBar {
    pub fn ui(ui: &mut egui::Ui, selection_count: usize, edit_mode: EditMode, snap_enabled: bool) {
        ui.horizontal(|ui| {
            // Selection info
            let selection_text = match selection_count {
                0 => "No selection".to_string(),
                1 => "1 clip selected".to_string(),
                n => format!("{} clips selected", n),
            };
            ui.label(selection_text);

            ui.separator();

            // Edit mode
            ui.label(format!("Mode: {}", edit_mode.name()));

            ui.separator();

            // Snap status
            let snap_icon = if snap_enabled { "🧲" } else { "" };
            ui.label(format!(
                "{}Snap: {}",
                snap_icon,
                if snap_enabled { "ON" } else { "OFF" }
            ));

            // Spacer
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Phase 1: Timeline UX").weak().italics());
            });
        });
    }
}
