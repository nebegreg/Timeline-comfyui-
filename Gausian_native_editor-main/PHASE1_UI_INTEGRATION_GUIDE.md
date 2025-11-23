# Phase 1 UI Integration Guide

This guide shows how to integrate the new Phase 1 UI components into your timeline.

## 🎨 New UI Modules

### 1. `timeline_ui_helpers.rs`
Visual helpers for selection and markers.

**Functions:**
- `draw_selection_outline()` - Draw selection outline around clips
- `draw_rect_selection()` - Draw rectangle drag selection
- `draw_marker()` - Draw timeline marker with label
- `draw_region()` - Draw in/out region
- `draw_snap_indicator()` - Visual snap feedback
- `keyboard_help_panel()` - Keyboard shortcuts help
- `edit_mode_button()` - Edit mode button with tooltip

### 2. `timeline_toolbar.rs`
Toolbar for edit modes and playback controls.

**Components:**
- `TimelineToolbar` - Edit mode selector, snap toggle, playback speed
- `PlaybackToolbar` - Play/pause, timecode display
- `StatusBar` - Selection info, edit mode, snap status

### 3. `marker_ui.rs`
Marker and region management panels.

**Components:**
- `MarkerPanel` - Add/edit/delete markers
- `RegionPanel` - Create regions from in/out points

---

## 🔧 Integration Example

### In your timeline UI code:

```rust
use crate::timeline_ui_helpers;
use crate::timeline_toolbar::{TimelineToolbar, StatusBar};
use crate::marker_ui::MarkerPanel;

pub struct YourApp {
    // ... existing fields ...

    // Phase 1 additions
    selection: SelectionState,
    edit_mode: EditMode,
    snap_settings: SnapSettings,
    markers: timeline::MarkerCollection,
    playback_speed: PlaybackSpeed,
    rect_selection: Option<RectSelection>,

    // UI state
    timeline_toolbar: TimelineToolbar,
    marker_panel: MarkerPanel,
}

impl YourApp {
    fn timeline_ui(&mut self, ui: &mut egui::Ui) {
        // 1. Draw toolbar at top
        self.timeline_toolbar.ui(
            ui,
            &mut self.edit_mode,
            &mut self.snap_settings,
            &self.playback_speed,
        );

        ui.separator();

        // 2. Main timeline area
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let painter = ui.painter();
            let timeline_rect = ui.available_rect_before_wrap();

            // Draw your clips as usual
            for (track_idx, track) in self.seq.graph.tracks.iter().enumerate() {
                for (clip_idx, node_id) in track.node_ids.iter().enumerate() {
                    let clip_rect = calculate_clip_rect(...); // Your existing logic

                    // Draw clip background
                    painter.rect_filled(clip_rect, 2.0, clip_color);

                    // ✨ NEW: Draw selection outline if selected
                    if self.selection.is_selected(node_id) {
                        let is_primary = self.selection.is_primary(node_id);
                        timeline_ui_helpers::draw_selection_outline(
                            &painter,
                            clip_rect,
                            is_primary,
                        );
                    }
                }
            }

            // ✨ NEW: Draw rectangle selection (drag-to-select)
            if let Some(rect_sel) = &self.rect_selection {
                timeline_ui_helpers::draw_rect_selection(
                    &painter,
                    rect_sel.rect(),
                );
            }

            // ✨ NEW: Draw markers
            for marker in self.markers.markers() {
                let x_pos = frame_to_x(marker.frame, zoom);
                timeline_ui_helpers::draw_marker(
                    &painter,
                    marker,
                    x_pos,
                    timeline_rect.top(),
                    timeline_rect.bottom(),
                    true, // show label
                );
            }

            // ✨ NEW: Draw in/out region
            if let Some((in_frame, out_frame)) = self.markers.get_in_out_range() {
                let start_x = frame_to_x(in_frame, zoom);
                let end_x = frame_to_x(out_frame, zoom);
                timeline_ui_helpers::draw_region(
                    &painter,
                    start_x,
                    end_x,
                    timeline_rect.top(),
                    timeline_rect.bottom(),
                    "#4A9EFF40",
                );
            }
        });

        ui.separator();

        // 3. Status bar at bottom
        StatusBar::ui(
            ui,
            self.selection.count(),
            self.edit_mode,
            self.snap_settings.enabled,
        );
    }
}
```

---

## 🖱️ Mouse Interaction Example

### Selection with rectangle drag:

```rust
fn handle_timeline_input(&mut self, response: &egui::Response) {
    // Start rectangle selection
    if response.drag_started() && !response.ctx.input(|i| i.modifiers.shift) {
        self.rect_selection = Some(RectSelection::new(response.interact_pointer_pos().unwrap()));
        self.selection.clear(); // Clear existing unless Shift held
    }

    // Update rectangle selection
    if let Some(rect_sel) = &mut self.rect_selection {
        if response.dragged() {
            rect_sel.update(response.interact_pointer_pos().unwrap());

            // Select clips within rectangle
            let node_rects: Vec<_> = /* collect (NodeId, Rect) */;
            self.selection.select_in_rect(rect_sel.rect(), &node_rects);
        }

        if response.drag_stopped() {
            self.rect_selection = None;
        }
    }

    // Click to select single clip
    if response.clicked() {
        if let Some(node_id) = node_at_pos(response.interact_pointer_pos().unwrap()) {
            if response.ctx.input(|i| i.modifiers.shift) {
                self.selection.toggle_selection(node_id);
            } else {
                self.selection.select_single(node_id);
            }
        }
    }
}
```

---

## ⌨️ Keyboard Shortcut Integration

```rust
fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
    use crate::keyboard::KeyCommand;

    // Playback
    if KeyCommand::PlayForward.check(ctx) {
        self.playback_speed.faster();
        self.playing = true;
    }
    if KeyCommand::PlayReverse.check(ctx) {
        self.playback_speed.slower();
        self.playing = true;
    }
    if KeyCommand::PlayPause.check(ctx) {
        self.playback_speed.pause();
        self.playing = false;
    }

    // Markers
    if KeyCommand::SetInPoint.check(ctx) {
        self.markers.set_in_point(self.playhead);
    }
    if KeyCommand::SetOutPoint.check(ctx) {
        self.markers.set_out_point(self.playhead);
    }
    if KeyCommand::AddMarker.check(ctx) {
        let marker = timeline::Marker::new(
            self.playhead,
            format!("Marker {}", self.markers.markers().count() + 1),
        );
        self.markers.add_marker(marker);
    }

    // Selection
    if KeyCommand::SelectAll.check(ctx) {
        let all_nodes = self.seq.graph.nodes.keys().copied();
        self.selection.select_all(all_nodes);
    }
    if KeyCommand::DeselectAll.check(ctx) {
        self.selection.clear();
    }

    // Edit modes
    if KeyCommand::SetNormalMode.check(ctx) {
        self.edit_mode = EditMode::Normal;
    }
    if KeyCommand::SetRippleMode.check(ctx) {
        self.edit_mode = EditMode::Ripple;
    }
    // ... other modes
}
```

---

## 🎯 Snapping Example

```rust
fn apply_snapping(&self, target_frame: i64) -> i64 {
    if !self.snap_settings.enabled {
        return target_frame;
    }

    let tolerance = (self.snap_settings.snap_tolerance / self.zoom_px_per_frame) as i64;
    let mut best_snap = target_frame;
    let mut best_distance = i64::MAX;

    // Snap to playhead
    if self.snap_settings.snap_to_playhead {
        let distance = (target_frame - self.playhead).abs();
        if distance < tolerance && distance < best_distance {
            best_snap = self.playhead;
            best_distance = distance;
        }
    }

    // Snap to clip edges
    if self.snap_settings.snap_to_clips {
        for track in &self.seq.graph.tracks {
            for node_id in &track.node_ids {
                if let Some(node) = self.seq.graph.nodes.get(node_id) {
                    if let timeline::TimelineNodeKind::Clip(clip) = &node.kind {
                        let start = clip.timeline_range.start;
                        let end = clip.timeline_range.end();

                        for snap_point in [start, end] {
                            let distance = (target_frame - snap_point).abs();
                            if distance < tolerance && distance < best_distance {
                                best_snap = snap_point;
                                best_distance = distance;
                            }
                        }
                    }
                }
            }
        }
    }

    // Snap to markers
    if self.snap_settings.snap_to_markers {
        for marker in self.markers.markers() {
            let distance = (target_frame - marker.frame).abs();
            if distance < tolerance && distance < best_distance {
                best_snap = marker.frame;
                best_distance = distance;
            }
        }
    }

    // Snap to second boundaries
    if self.snap_settings.snap_to_seconds {
        let fps = self.seq.fps.num as i64;
        let second_boundary = (target_frame / fps) * fps;
        let distance = (target_frame - second_boundary).abs();
        if distance < tolerance && distance < best_distance {
            best_snap = second_boundary;
        }
    }

    best_snap
}
```

---

## 📐 Complete Example App Update Loop

```rust
impl eframe::App for YourApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts first
        self.handle_keyboard_input(ctx);

        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.timeline_toolbar.ui(
                ui,
                &mut self.edit_mode,
                &mut self.snap_settings,
                &self.playback_speed,
            );
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            StatusBar::ui(
                ui,
                self.selection.count(),
                self.edit_mode,
                self.snap_settings.enabled,
            );
        });

        // Side panel for markers
        egui::SidePanel::right("markers").show(ctx, |ui| {
            self.marker_panel.ui(ui, &mut self.markers, self.playhead);
        });

        // Central timeline
        egui::CentralPanel::default().show(ctx, |ui| {
            self.timeline_ui(ui);
        });

        // Keyboard help window
        if self.timeline_toolbar.show_keyboard_help {
            egui::Window::new("⌨️ Keyboard Shortcuts")
                .open(&mut self.timeline_toolbar.show_keyboard_help)
                .show(ctx, |ui| {
                    timeline_ui_helpers::keyboard_help_panel(ui);
                });
        }
    }
}
```

---

## 🎨 Customizing Colors

All colors can be customized by modifying the Style structs:

```rust
// In timeline_ui_helpers.rs

// Custom selection color
let mut style = SelectionStyle::default();
style.outline_color = egui::Color32::from_rgb(255, 100, 255); // Magenta

// Custom marker style
let mut marker_style = MarkerStyle::from_hex("#FF6B6B"); // Custom red
marker_style.height = 18.0;
```

---

## 📊 Performance Tips

1. **Culling**: Only draw markers/clips visible in viewport
2. **LOD**: Reduce marker label detail when zoomed out
3. **Caching**: Cache clip rects per frame
4. **Batching**: Group similar shapes for painter

```rust
// Viewport culling example
let visible_range = calculate_visible_frame_range(scroll_offset, zoom);

for marker in self.markers.markers() {
    if marker.frame < visible_range.start || marker.frame > visible_range.end {
        continue; // Skip offscreen markers
    }

    // Only draw visible markers
    draw_marker(...);
}
```

---

## ✨ Next Steps

Once integrated, you can:
1. Test selection with Shift-click
2. Try J/K/L playback speeds
3. Add markers with M key
4. Set in/out with I/O keys
5. Create regions
6. Switch edit modes with N/R/T/S/Y

Happy coding! 🦀
