/// Professional keyboard shortcuts system
/// Phase 1: Timeline Polish & UX Improvements

use eframe::egui;

/// Keyboard command
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    // Playback
    PlayPause,
    PlayReverse,
    PlayForward,
    StepBackward,
    StepForward,
    JumpToStart,
    JumpToEnd,

    // Editing
    Cut,
    Copy,
    Paste,
    Delete,
    Duplicate,
    Undo,
    Redo,

    // Selection
    SelectAll,
    DeselectAll,

    // Markers
    AddMarker,
    SetInPoint,
    SetOutPoint,
    ClearInOut,
    NextMarker,
    PrevMarker,

    // Trimming
    TrimStartToPlayhead,
    TrimEndToPlayhead,

    // Edit Modes
    SetNormalMode,
    SetRippleMode,
    SetRollMode,
    SetSlideMode,
    SetSlipMode,

    // Navigation
    NextEdit,
    PrevEdit,
    GoToTimecode,

    // Timeline
    ZoomIn,
    ZoomOut,
    ZoomToFit,
    ZoomToSelection,

    // Append/Insert
    AppendToTimeline,
    InsertAtPlayhead,

    // Tools
    ToggleSnapping,
    ToggleLinking,

    // View
    ToggleFullscreen,
    ToggleTimecode,
}

impl KeyCommand {
    /// Check if keyboard input matches this command
    pub fn check(&self, ctx: &egui::Context) -> bool {
        ctx.input(|i| {
            match self {
                // Playback (J/K/L)
                Self::PlayReverse => i.key_pressed(egui::Key::J),
                Self::PlayPause => i.key_pressed(egui::Key::K),
                Self::PlayForward => i.key_pressed(egui::Key::L),

                // Step frame
                Self::StepBackward => i.key_pressed(egui::Key::ArrowLeft) && !i.modifiers.shift,
                Self::StepForward => i.key_pressed(egui::Key::ArrowRight) && !i.modifiers.shift,

                // Jump to start/end
                Self::JumpToStart => i.key_pressed(egui::Key::Home),
                Self::JumpToEnd => i.key_pressed(egui::Key::End),

                // Editing
                Self::Cut => i.modifiers.command && i.key_pressed(egui::Key::X),
                Self::Copy => i.modifiers.command && i.key_pressed(egui::Key::C),
                Self::Paste => i.modifiers.command && i.key_pressed(egui::Key::V),
                Self::Delete => {
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                }
                Self::Duplicate => i.modifiers.command && i.key_pressed(egui::Key::D),
                Self::Undo => i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift,
                Self::Redo => {
                    (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
                        || (i.modifiers.command && i.key_pressed(egui::Key::Y))
                }

                // Selection
                Self::SelectAll => i.modifiers.command && i.key_pressed(egui::Key::A),
                Self::DeselectAll => {
                    i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::A)
                }

                // Markers
                Self::AddMarker => i.key_pressed(egui::Key::M),
                Self::SetInPoint => i.key_pressed(egui::Key::I),
                Self::SetOutPoint => i.key_pressed(egui::Key::O),
                Self::ClearInOut => {
                    i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::X)
                }
                Self::NextMarker => i.modifiers.shift && i.key_pressed(egui::Key::M),
                Self::PrevMarker => {
                    i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::M)
                }

                // Trimming
                Self::TrimStartToPlayhead => i.key_pressed(egui::Key::Q),
                Self::TrimEndToPlayhead => i.key_pressed(egui::Key::W),

                // Edit Modes
                Self::SetNormalMode => i.key_pressed(egui::Key::N) && !i.modifiers.command,
                Self::SetRippleMode => i.key_pressed(egui::Key::R) && !i.modifiers.command,
                Self::SetRollMode => i.key_pressed(egui::Key::T) && !i.modifiers.command,
                Self::SetSlideMode => i.key_pressed(egui::Key::S) && !i.modifiers.command,
                Self::SetSlipMode => i.key_pressed(egui::Key::Y) && !i.modifiers.command,

                // Navigation
                Self::NextEdit => {
                    i.modifiers.shift && i.key_pressed(egui::Key::ArrowRight)
                }
                Self::PrevEdit => {
                    i.modifiers.shift && i.key_pressed(egui::Key::ArrowLeft)
                }
                Self::GoToTimecode => {
                    i.modifiers.command && i.key_pressed(egui::Key::G)
                }

                // Timeline zoom
                Self::ZoomIn => i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus),
                Self::ZoomOut => i.key_pressed(egui::Key::Minus),
                Self::ZoomToFit => i.modifiers.shift && i.key_pressed(egui::Key::Z),
                Self::ZoomToSelection => {
                    i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)
                }

                // Append/Insert (E key)
                Self::AppendToTimeline => i.key_pressed(egui::Key::E),
                Self::InsertAtPlayhead => i.modifiers.shift && i.key_pressed(egui::Key::E),

                // Tools
                Self::ToggleSnapping => i.key_pressed(egui::Key::Num1),
                Self::ToggleLinking => {
                    i.modifiers.command && i.key_pressed(egui::Key::L)
                }

                // View
                Self::ToggleFullscreen => i.key_pressed(egui::Key::F11),
                Self::ToggleTimecode => {
                    i.modifiers.command && i.key_pressed(egui::Key::T)
                }
            }
        })
    }

    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            Self::PlayPause => "Play/Pause",
            Self::PlayReverse => "Play Reverse",
            Self::PlayForward => "Play Forward",
            Self::StepBackward => "Step Backward",
            Self::StepForward => "Step Forward",
            Self::JumpToStart => "Jump to Start",
            Self::JumpToEnd => "Jump to End",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::Delete => "Delete",
            Self::Duplicate => "Duplicate",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::SelectAll => "Select All",
            Self::DeselectAll => "Deselect All",
            Self::AddMarker => "Add Marker",
            Self::SetInPoint => "Set In Point",
            Self::SetOutPoint => "Set Out Point",
            Self::ClearInOut => "Clear In/Out",
            Self::NextMarker => "Next Marker",
            Self::PrevMarker => "Previous Marker",
            Self::TrimStartToPlayhead => "Trim Start to Playhead",
            Self::TrimEndToPlayhead => "Trim End to Playhead",
            Self::SetNormalMode => "Normal Edit Mode",
            Self::SetRippleMode => "Ripple Edit Mode",
            Self::SetRollMode => "Roll Edit Mode",
            Self::SetSlideMode => "Slide Edit Mode",
            Self::SetSlipMode => "Slip Edit Mode",
            Self::NextEdit => "Next Edit",
            Self::PrevEdit => "Previous Edit",
            Self::GoToTimecode => "Go to Timecode",
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
            Self::ZoomToFit => "Zoom to Fit",
            Self::ZoomToSelection => "Zoom to Selection",
            Self::AppendToTimeline => "Append to Timeline",
            Self::InsertAtPlayhead => "Insert at Playhead",
            Self::ToggleSnapping => "Toggle Snapping",
            Self::ToggleLinking => "Toggle Linking",
            Self::ToggleFullscreen => "Toggle Fullscreen",
            Self::ToggleTimecode => "Toggle Timecode Display",
        }
    }

    /// Get keyboard shortcut display string
    pub fn shortcut_text(&self) -> &str {
        match self {
            Self::PlayPause => "K",
            Self::PlayReverse => "J",
            Self::PlayForward => "L",
            Self::StepBackward => "←",
            Self::StepForward => "→",
            Self::JumpToStart => "Home",
            Self::JumpToEnd => "End",
            Self::Cut => "Cmd+X",
            Self::Copy => "Cmd+C",
            Self::Paste => "Cmd+V",
            Self::Delete => "Del",
            Self::Duplicate => "Cmd+D",
            Self::Undo => "Cmd+Z",
            Self::Redo => "Cmd+Shift+Z",
            Self::SelectAll => "Cmd+A",
            Self::DeselectAll => "Cmd+Shift+A",
            Self::AddMarker => "M",
            Self::SetInPoint => "I",
            Self::SetOutPoint => "O",
            Self::ClearInOut => "Cmd+Shift+X",
            Self::NextMarker => "Shift+M",
            Self::PrevMarker => "Cmd+Shift+M",
            Self::TrimStartToPlayhead => "Q",
            Self::TrimEndToPlayhead => "W",
            Self::SetNormalMode => "N",
            Self::SetRippleMode => "R",
            Self::SetRollMode => "T",
            Self::SetSlideMode => "S",
            Self::SetSlipMode => "Y",
            Self::NextEdit => "Shift+→",
            Self::PrevEdit => "Shift+←",
            Self::GoToTimecode => "Cmd+G",
            Self::ZoomIn => "+",
            Self::ZoomOut => "-",
            Self::ZoomToFit => "Shift+Z",
            Self::ZoomToSelection => "Cmd+Shift+Z",
            Self::AppendToTimeline => "E",
            Self::InsertAtPlayhead => "Shift+E",
            Self::ToggleSnapping => "1",
            Self::ToggleLinking => "Cmd+L",
            Self::ToggleFullscreen => "F11",
            Self::ToggleTimecode => "Cmd+T",
        }
    }

    /// Get all commands for help display
    pub fn all_commands() -> Vec<(&'static str, Vec<Self>)> {
        vec![
            (
                "Playback",
                vec![
                    Self::PlayReverse,
                    Self::PlayPause,
                    Self::PlayForward,
                    Self::StepBackward,
                    Self::StepForward,
                    Self::JumpToStart,
                    Self::JumpToEnd,
                ],
            ),
            (
                "Editing",
                vec![
                    Self::Cut,
                    Self::Copy,
                    Self::Paste,
                    Self::Delete,
                    Self::Duplicate,
                    Self::Undo,
                    Self::Redo,
                ],
            ),
            (
                "Selection",
                vec![Self::SelectAll, Self::DeselectAll],
            ),
            (
                "Markers",
                vec![
                    Self::AddMarker,
                    Self::SetInPoint,
                    Self::SetOutPoint,
                    Self::ClearInOut,
                    Self::NextMarker,
                    Self::PrevMarker,
                ],
            ),
            (
                "Trimming",
                vec![Self::TrimStartToPlayhead, Self::TrimEndToPlayhead],
            ),
            (
                "Edit Modes",
                vec![
                    Self::SetNormalMode,
                    Self::SetRippleMode,
                    Self::SetRollMode,
                    Self::SetSlideMode,
                    Self::SetSlipMode,
                ],
            ),
            (
                "Navigation",
                vec![
                    Self::NextEdit,
                    Self::PrevEdit,
                    Self::GoToTimecode,
                ],
            ),
            (
                "Timeline",
                vec![
                    Self::ZoomIn,
                    Self::ZoomOut,
                    Self::ZoomToFit,
                    Self::ZoomToSelection,
                ],
            ),
        ]
    }
}

/// Playback speed multiplier for J/K/L
#[derive(Clone, Copy, Debug)]
pub struct PlaybackSpeed {
    pub speed: f32,
    pub reverse: bool,
}

impl PlaybackSpeed {
    pub fn new() -> Self {
        Self {
            speed: 0.0,
            reverse: false,
        }
    }

    /// Increase speed (multiple L presses)
    pub fn faster(&mut self) {
        self.reverse = false;
        if self.speed == 0.0 {
            self.speed = 1.0;
        } else if self.speed < 8.0 {
            self.speed *= 2.0;
        }
    }

    /// Decrease/reverse speed (multiple J presses)
    pub fn slower(&mut self) {
        self.reverse = true;
        if self.speed == 0.0 {
            self.speed = 1.0;
        } else if self.speed < 8.0 {
            self.speed *= 2.0;
        }
    }

    /// Pause (K press)
    pub fn pause(&mut self) {
        self.speed = 0.0;
        self.reverse = false;
    }

    /// Get final speed with direction
    pub fn final_speed(&self) -> f32 {
        if self.reverse {
            -self.speed
        } else {
            self.speed
        }
    }

    /// Is paused?
    pub fn is_paused(&self) -> bool {
        self.speed == 0.0
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self::new()
    }
}
