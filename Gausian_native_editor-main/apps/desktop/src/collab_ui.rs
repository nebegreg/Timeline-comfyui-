/// Phase 7: Collaborative Editing UI Components
/// Real-time presence indicators, cursors, selections, and user management
use collaboration::{
    CursorPosition, PresenceUpdate, Selection, SessionId, SyncMessage, User, UserId, UserPresence,
    Viewport,
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use std::collections::HashMap;

/// Collaboration UI state
pub struct CollabUI {
    /// Current session (if connected)
    pub session_id: Option<SessionId>,

    /// Current user
    pub user_id: Option<UserId>,

    /// Other users in the session
    pub remote_users: HashMap<UserId, RemoteUserState>,

    /// Show user list panel
    pub show_user_list: bool,

    /// Connection status
    pub connection_status: ConnectionStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct RemoteUserState {
    pub user: User,
    pub presence: UserPresence,
    pub last_update: std::time::Instant,
}

impl CollabUI {
    pub fn new() -> Self {
        Self {
            session_id: None,
            user_id: None,
            remote_users: HashMap::new(),
            show_user_list: false,
            connection_status: ConnectionStatus::Disconnected,
        }
    }

    /// Update remote user presence
    pub fn update_presence(&mut self, user_id: UserId, presence: UserPresence) {
        self.remote_users.insert(
            user_id,
            RemoteUserState {
                user: presence.user.clone(),
                presence,
                last_update: std::time::Instant::now(),
            },
        );
    }

    /// Remove a remote user
    pub fn remove_user(&mut self, user_id: &UserId) {
        self.remote_users.remove(user_id);
    }

    /// Render remote cursors on the timeline
    pub fn render_remote_cursors(
        &self,
        ui: &mut Ui,
        timeline_rect: Rect,
        frame_to_screen: impl Fn(i32) -> f32,
    ) {
        for (user_id, state) in &self.remote_users {
            if let Some(cursor) = &state.presence.cursor_position {
                // Skip if user is self
                if Some(*user_id) == self.user_id {
                    continue;
                }

                // Convert frame to screen position
                let x = frame_to_screen(cursor.frame);
                let cursor_pos = Pos2::new(x, timeline_rect.top());

                // Draw cursor line
                let color = user_color_to_egui(&state.user.color);
                let cursor_height = timeline_rect.height();

                ui.painter().line_segment(
                    [cursor_pos, cursor_pos + Vec2::new(0.0, cursor_height)],
                    Stroke::new(2.0, color),
                );

                // Draw user name tag
                let name_pos = cursor_pos + Vec2::new(4.0, 2.0);
                let name_bg = ui.painter().add(egui::Shape::rect_filled(
                    Rect::from_min_size(name_pos, Vec2::new(100.0, 20.0)),
                    2.0,
                    color.linear_multiply(0.9),
                ));

                ui.painter().text(
                    name_pos + Vec2::new(4.0, 10.0),
                    Align2::LEFT_CENTER,
                    &state.user.name,
                    FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
        }
    }

    /// Render remote selections on timeline clips
    pub fn render_remote_selections(
        &self,
        ui: &mut Ui,
        node_rects: &HashMap<timeline::NodeId, Rect>,
    ) {
        for (user_id, state) in &self.remote_users {
            if let Some(selection) = &state.presence.selection {
                // Skip if user is self
                if Some(*user_id) == self.user_id {
                    continue;
                }

                let color = user_color_to_egui(&state.user.color).linear_multiply(0.6);

                for node_id in &selection.node_ids {
                    if let Some(rect) = node_rects.get(node_id) {
                        // Draw selection outline
                        ui.painter()
                            .rect_stroke(*rect, 3.0, Stroke::new(2.0, color));

                        // Draw small user indicator in corner
                        let indicator_pos = rect.right_top() + Vec2::new(-20.0, 4.0);
                        ui.painter().circle_filled(indicator_pos, 6.0, color);

                        // User initial
                        let initial = state.user.name.chars().next().unwrap_or('?');
                        ui.painter().text(
                            indicator_pos,
                            Align2::CENTER_CENTER,
                            initial.to_string(),
                            FontId::proportional(10.0),
                            Color32::WHITE,
                        );
                    }
                }
            }
        }
    }

    /// Render user list panel
    pub fn render_user_list_panel(&mut self, ctx: &egui::Context) {
        if !self.show_user_list {
            return;
        }

        egui::Window::new("👥 Collaborators")
            .default_width(250.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.heading("Active Users");
                ui.separator();

                // Connection status
                match &self.connection_status {
                    ConnectionStatus::Connected => {
                        ui.colored_label(Color32::GREEN, "● Connected");
                    }
                    ConnectionStatus::Connecting => {
                        ui.colored_label(Color32::YELLOW, "⟳ Connecting...");
                    }
                    ConnectionStatus::Disconnected => {
                        ui.colored_label(Color32::RED, "● Disconnected");
                    }
                    ConnectionStatus::Error(err) => {
                        ui.colored_label(Color32::RED, format!("✗ Error: {}", err));
                    }
                }

                ui.add_space(8.0);

                // User list
                if self.remote_users.is_empty() {
                    ui.label("No other users in session");
                } else {
                    for (user_id, state) in &self.remote_users {
                        ui.horizontal(|ui| {
                            // Color indicator
                            let color = user_color_to_egui(&state.user.color);
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 6.0, color);

                            // User name
                            ui.label(&state.user.name);

                            // Activity status
                            if state.presence.is_idle() {
                                ui.label("💤");
                            } else {
                                ui.label("✓");
                            }
                        });

                        // User activity details
                        if let Some(cursor) = &state.presence.cursor_position {
                            ui.indent(format!("user_{}", user_id.0), |ui| {
                                ui.label(format!("  Frame: {}", cursor.frame));
                            });
                        }

                        ui.separator();
                    }
                }

                ui.add_space(16.0);

                // Session info
                if let Some(session_id) = &self.session_id {
                    ui.group(|ui| {
                        ui.label(format!("Session ID:"));
                        ui.monospace(session_id.0.to_string());

                        if ui.button("📋 Copy").clicked() {
                            ui.output_mut(|o| {
                                o.copied_text = session_id.0.to_string();
                            });
                        }
                    });
                }
            });
    }

    /// Render connection status indicator in main toolbar
    pub fn render_status_indicator(&self, ui: &mut Ui) -> Response {
        let (icon, color, tooltip) = match &self.connection_status {
            ConnectionStatus::Connected => {
                let user_count = self.remote_users.len();
                (
                    format!("👥 {}", user_count),
                    Color32::GREEN,
                    format!("{} users connected", user_count),
                )
            }
            ConnectionStatus::Connecting => (
                "⟳".to_string(),
                Color32::YELLOW,
                "Connecting...".to_string(),
            ),
            ConnectionStatus::Disconnected => {
                ("●".to_string(), Color32::GRAY, "Disconnected".to_string())
            }
            ConnectionStatus::Error(err) => {
                ("✗".to_string(), Color32::RED, format!("Error: {}", err))
            }
        };

        let response = ui.button(egui::RichText::new(icon).color(color));
        response.on_hover_text(tooltip)
    }

    /// Handle presence update message from server
    pub fn handle_presence_update(&mut self, update: PresenceUpdate) {
        match update {
            PresenceUpdate::UserJoined { user } => {
                let presence = UserPresence::new(user.clone(), self.session_id.unwrap());
                self.update_presence(user.id, presence);
            }
            PresenceUpdate::UserLeft { user_id } => {
                self.remove_user(&user_id);
            }
            PresenceUpdate::CursorMoved { user_id, position } => {
                if let Some(state) = self.remote_users.get_mut(&user_id) {
                    state.presence.cursor_position = Some(position);
                    state.last_update = std::time::Instant::now();
                }
            }
            PresenceUpdate::SelectionChanged { user_id, selection } => {
                if let Some(state) = self.remote_users.get_mut(&user_id) {
                    state.presence.selection = Some(selection);
                    state.last_update = std::time::Instant::now();
                }
            }
            PresenceUpdate::ViewportChanged { user_id, viewport } => {
                if let Some(state) = self.remote_users.get_mut(&user_id) {
                    state.presence.viewport = Some(viewport);
                    state.last_update = std::time::Instant::now();
                }
            }
            _ => {}
        }
    }

    /// Cleanup stale users (no updates in 60 seconds)
    pub fn cleanup_stale_users(&mut self) {
        let now = std::time::Instant::now();
        self.remote_users
            .retain(|_, state| now.duration_since(state.last_update).as_secs() < 60);
    }
}

impl Default for CollabUI {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert collaboration UserColor to egui Color32
fn user_color_to_egui(color: &collaboration::UserColor) -> Color32 {
    Color32::from_rgb(color.r, color.g, color.b)
}

/// Conflict resolution dialog
pub struct ConflictDialog {
    pub conflicts: Vec<ConflictInfo>,
    pub show: bool,
}

#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub description: String,
    pub local_change: String,
    pub remote_change: String,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    Pending,
    UseLocal,
    UseRemote,
    Manual,
}

impl ConflictDialog {
    pub fn new() -> Self {
        Self {
            conflicts: Vec::new(),
            show: false,
        }
    }

    pub fn add_conflict(&mut self, conflict: ConflictInfo) {
        self.conflicts.push(conflict);
        self.show = true;
    }

    pub fn render(&mut self, ctx: &egui::Context) -> Vec<ConflictResolution> {
        if !self.show {
            return Vec::new();
        }

        let mut resolutions = Vec::new();

        egui::Window::new("⚠️ Merge Conflicts")
            .default_width(500.0)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("Changes conflict with other users");
                ui.label("Choose how to resolve each conflict:");
                ui.separator();

                for (idx, conflict) in self.conflicts.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(&conflict.description).strong());
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Your change:");
                            ui.monospace(&conflict.local_change);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Remote change:");
                            ui.monospace(&conflict.remote_change);
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.button("Use Mine").clicked() {
                                conflict.resolution = ConflictResolution::UseLocal;
                            }
                            if ui.button("Use Theirs").clicked() {
                                conflict.resolution = ConflictResolution::UseRemote;
                            }
                            if ui.button("Manual").clicked() {
                                conflict.resolution = ConflictResolution::Manual;
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| match &conflict.resolution {
                                    ConflictResolution::Pending => {
                                        ui.colored_label(Color32::YELLOW, "⏸ Pending");
                                    }
                                    ConflictResolution::UseLocal => {
                                        ui.colored_label(Color32::GREEN, "✓ Using local");
                                    }
                                    ConflictResolution::UseRemote => {
                                        ui.colored_label(Color32::BLUE, "✓ Using remote");
                                    }
                                    ConflictResolution::Manual => {
                                        ui.colored_label(Color32::ORANGE, "⚠ Manual");
                                    }
                                },
                            );
                        });
                    });

                    ui.add_space(8.0);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Apply Resolutions").clicked() {
                        resolutions = self
                            .conflicts
                            .iter()
                            .map(|c| c.resolution.clone())
                            .collect();
                        self.show = false;
                        self.conflicts.clear();
                    }

                    if ui.button("Cancel").clicked() {
                        self.show = false;
                    }
                });
            });

        resolutions
    }
}

impl Default for ConflictDialog {
    fn default() -> Self {
        Self::new()
    }
}
