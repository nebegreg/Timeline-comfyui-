/// Advanced edit operations for professional timeline editing
/// Phase 1: Timeline Polish & UX - Complete Implementation

use crate::{Frame, FrameRange, NodeId, TimelineGraph, TimelineNodeKind, TrackId};

/// Ripple edit: Move clip and shift all following clips on the same track
pub fn ripple_move_clip(
    graph: &mut TimelineGraph,
    node_id: &NodeId,
    new_start: Frame,
) -> Result<Vec<(NodeId, Frame)>, String> {
    let node = graph
        .nodes
        .get(node_id)
        .ok_or_else(|| "Node not found".to_string())?;

    let TimelineNodeKind::Clip(clip) = &node.kind else {
        return Err("Node is not a clip".to_string());
    };

    let old_start = clip.timeline_range.start;
    let delta = new_start - old_start;

    // Find the track containing this clip
    let track_id = graph
        .tracks
        .iter()
        .find(|t| t.node_ids.contains(node_id))
        .ok_or_else(|| "Clip not found in any track".to_string())?
        .id;

    let track = graph
        .tracks
        .iter()
        .find(|t| t.id == track_id)
        .ok_or_else(|| "Track not found".to_string())?;

    // Collect all clips after this one on the same track
    let mut moved_clips = Vec::new();

    for other_node_id in &track.node_ids {
        if other_node_id == node_id {
            continue;
        }

        if let Some(other_node) = graph.nodes.get(other_node_id) {
            if let TimelineNodeKind::Clip(other_clip) = &other_node.kind {
                // If this clip starts at or after the moved clip's original end
                let moved_clip_original_end = old_start + clip.timeline_range.duration;
                if other_clip.timeline_range.start >= moved_clip_original_end {
                    moved_clips.push((*other_node_id, other_clip.timeline_range.start + delta));
                }
            }
        }
    }

    // Apply the moves
    // First move the primary clip
    if let Some(node) = graph.nodes.get_mut(node_id) {
        if let TimelineNodeKind::Clip(clip) = &mut node.kind {
            clip.timeline_range.start = new_start;
        }
    }

    // Then shift all following clips
    for (other_id, new_pos) in &moved_clips {
        if let Some(node) = graph.nodes.get_mut(other_id) {
            if let TimelineNodeKind::Clip(clip) = &mut node.kind {
                clip.timeline_range.start = *new_pos;
            }
        }
    }

    Ok(moved_clips)
}

/// Ripple trim: Trim clip and shift all following clips
pub fn ripple_trim_clip(
    graph: &mut TimelineGraph,
    node_id: &NodeId,
    new_range: FrameRange,
) -> Result<Vec<(NodeId, Frame)>, String> {
    let node = graph
        .nodes
        .get(node_id)
        .ok_or_else(|| "Node not found".to_string())?;

    let TimelineNodeKind::Clip(clip) = &node.kind else {
        return Err("Node is not a clip".to_string());
    };

    let old_end = clip.timeline_range.start + clip.timeline_range.duration;
    let new_end = new_range.start + new_range.duration;
    let delta = new_end - old_end;

    // Find the track
    let track_id = graph
        .tracks
        .iter()
        .find(|t| t.node_ids.contains(node_id))
        .ok_or_else(|| "Clip not found in any track".to_string())?
        .id;

    let track = graph
        .tracks
        .iter()
        .find(|t| t.id == track_id)
        .ok_or_else(|| "Track not found".to_string())?;

    // Collect all clips after this one
    let mut moved_clips = Vec::new();

    for other_node_id in &track.node_ids {
        if other_node_id == node_id {
            continue;
        }

        if let Some(other_node) = graph.nodes.get(other_node_id) {
            if let TimelineNodeKind::Clip(other_clip) = &other_node.kind {
                if other_clip.timeline_range.start >= old_end {
                    moved_clips.push((*other_node_id, other_clip.timeline_range.start + delta));
                }
            }
        }
    }

    // Apply the trim
    if let Some(node) = graph.nodes.get_mut(node_id) {
        if let TimelineNodeKind::Clip(clip) = &mut node.kind {
            clip.timeline_range = new_range;
        }
    }

    // Shift following clips
    for (other_id, new_pos) in &moved_clips {
        if let Some(node) = graph.nodes.get_mut(other_id) {
            if let TimelineNodeKind::Clip(clip) = &mut node.kind {
                clip.timeline_range.start = *new_pos;
            }
        }
    }

    Ok(moved_clips)
}

/// Roll edit: Adjust the edit point between two adjacent clips
/// Extends one clip and trims the other to maintain timeline continuity
pub fn roll_edit(
    graph: &mut TimelineGraph,
    left_node_id: &NodeId,
    right_node_id: &NodeId,
    new_edit_point: Frame,
) -> Result<(), String> {
    // Get both clips
    let left_node = graph
        .nodes
        .get(left_node_id)
        .ok_or_else(|| "Left node not found".to_string())?;

    let right_node = graph
        .nodes
        .get(right_node_id)
        .ok_or_else(|| "Right node not found".to_string())?;

    let TimelineNodeKind::Clip(left_clip) = &left_node.kind else {
        return Err("Left node is not a clip".to_string());
    };

    let TimelineNodeKind::Clip(right_clip) = &right_node.kind else {
        return Err("Right node is not a clip".to_string());
    };

    let left_start = left_clip.timeline_range.start;
    let right_end = right_clip.timeline_range.start + right_clip.timeline_range.duration;

    // Validate edit point is between the clips
    if new_edit_point <= left_start || new_edit_point >= right_end {
        return Err("Edit point must be between the two clips".to_string());
    }

    // Calculate new durations
    let new_left_duration = new_edit_point - left_start;
    let new_right_duration = right_end - new_edit_point;

    // Validate we don't exceed media bounds
    let left_media_end = left_clip.media_range.start + left_clip.media_range.duration;
    let new_left_media_end = left_clip.media_range.start + new_left_duration;

    if new_left_media_end > left_media_end {
        return Err("Cannot extend left clip beyond media bounds".to_string());
    }

    let right_media_start = right_clip.media_range.start;
    let media_delta = new_edit_point - right_clip.timeline_range.start;
    let new_right_media_start = right_media_start + media_delta;
    let right_media_end = right_clip.media_range.start + right_clip.media_range.duration;

    if new_right_media_start >= right_media_end {
        return Err("Cannot trim right clip beyond media bounds".to_string());
    }

    // Apply the roll edit
    if let Some(node) = graph.nodes.get_mut(left_node_id) {
        if let TimelineNodeKind::Clip(clip) = &mut node.kind {
            clip.timeline_range.duration = new_left_duration;
            // Extend media range
            clip.media_range.duration = new_left_duration;
        }
    }

    if let Some(node) = graph.nodes.get_mut(right_node_id) {
        if let TimelineNodeKind::Clip(clip) = &mut node.kind {
            clip.timeline_range.start = new_edit_point;
            clip.timeline_range.duration = new_right_duration;
            // Trim media range
            clip.media_range.start = new_right_media_start;
            clip.media_range.duration = new_right_duration;
        }
    }

    Ok(())
}

/// Slide edit: Move clip content without changing timeline position
/// Changes in/out points but keeps timeline range fixed
pub fn slide_edit(
    graph: &mut TimelineGraph,
    node_id: &NodeId,
    media_offset: Frame,
) -> Result<(), String> {
    let node = graph
        .nodes
        .get(node_id)
        .ok_or_else(|| "Node not found".to_string())?;

    let TimelineNodeKind::Clip(clip) = &node.kind else {
        return Err("Node is not a clip".to_string());
    };

    let new_media_start = clip.media_range.start + media_offset;

    // Validate bounds
    if new_media_start < 0 {
        return Err("Cannot slide before media start".to_string());
    }

    let media_total_duration = clip.media_range.duration; // Assuming this is total available media
    let new_media_end = new_media_start + clip.timeline_range.duration;

    if new_media_end > clip.media_range.start + media_total_duration {
        return Err("Cannot slide beyond media end".to_string());
    }

    // Apply slide
    if let Some(node) = graph.nodes.get_mut(node_id) {
        if let TimelineNodeKind::Clip(clip) = &mut node.kind {
            clip.media_range.start = new_media_start;
            // Timeline position stays the same
        }
    }

    Ok(())
}

/// Slip edit: Change what portion of media is visible
/// Moves in/out points together without changing timeline position
pub fn slip_edit(
    graph: &mut TimelineGraph,
    node_id: &NodeId,
    slip_amount: Frame,
) -> Result<(), String> {
    // Slip is similar to slide but conceptually about changing visible portion
    slide_edit(graph, node_id, slip_amount)
}

/// Find adjacent clips for roll edit
pub fn find_adjacent_clips(
    graph: &TimelineGraph,
    node_id: &NodeId,
) -> Result<Option<(NodeId, NodeId)>, String> {
    let node = graph
        .nodes
        .get(node_id)
        .ok_or_else(|| "Node not found".to_string())?;

    let TimelineNodeKind::Clip(clip) = &node.kind else {
        return Err("Node is not a clip".to_string());
    };

    let clip_end = clip.timeline_range.start + clip.timeline_range.duration;

    // Find the track
    let track = graph
        .tracks
        .iter()
        .find(|t| t.node_ids.contains(node_id))
        .ok_or_else(|| "Clip not found in any track".to_string())?;

    // Find clip immediately after this one
    let mut next_clip: Option<(NodeId, Frame)> = None;

    for other_node_id in &track.node_ids {
        if other_node_id == node_id {
            continue;
        }

        if let Some(other_node) = graph.nodes.get(other_node_id) {
            if let TimelineNodeKind::Clip(other_clip) = &other_node.kind {
                let gap = other_clip.timeline_range.start - clip_end;

                // If clips are adjacent (no gap)
                if gap == 0 {
                    match next_clip {
                        None => next_clip = Some((*other_node_id, other_clip.timeline_range.start)),
                        Some((_, existing_start)) => {
                            if other_clip.timeline_range.start < existing_start {
                                next_clip = Some((*other_node_id, other_clip.timeline_range.start));
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some((next_id, _)) = next_clip {
        Ok(Some((*node_id, next_id)))
    } else {
        Ok(None)
    }
}

/// Remove gap between clips (ripple delete a gap)
pub fn close_gap(
    graph: &mut TimelineGraph,
    track_id: &TrackId,
    gap_start: Frame,
    gap_end: Frame,
) -> Result<Vec<(NodeId, Frame)>, String> {
    let track = graph
        .tracks
        .iter()
        .find(|t| t.id == *track_id)
        .ok_or_else(|| "Track not found".to_string())?;

    let gap_duration = gap_end - gap_start;
    let mut moved_clips = Vec::new();

    // Find all clips after the gap
    for node_id in &track.node_ids {
        if let Some(node) = graph.nodes.get(node_id) {
            if let TimelineNodeKind::Clip(clip) = &node.kind {
                if clip.timeline_range.start >= gap_end {
                    moved_clips.push((*node_id, clip.timeline_range.start - gap_duration));
                }
            }
        }
    }

    // Move clips
    for (node_id, new_pos) in &moved_clips {
        if let Some(node) = graph.nodes.get_mut(node_id) {
            if let TimelineNodeKind::Clip(clip) = &mut node.kind {
                clip.timeline_range.start = *new_pos;
            }
        }
    }

    Ok(moved_clips)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TimelineNode, TrackBinding, TrackKind};

    fn create_test_clip(start: Frame, duration: Frame) -> TimelineNode {
        use crate::ClipNode;
        TimelineNode {
            id: NodeId::new(),
            label: Some("Test Clip".to_string()),
            kind: TimelineNodeKind::Clip(ClipNode {
                asset_id: Some("test.mp4".to_string()),
                // Give plenty of media duration for roll edits (3x the timeline duration)
                media_range: FrameRange { start: 0, duration: duration * 3 },
                timeline_range: FrameRange { start, duration },
                playback_rate: 1.0,
                reverse: false,
                metadata: serde_json::Value::Null,
            }),
            locked: false,
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_ripple_move() {
        let mut graph = TimelineGraph::default();

        // Create a track
        let track = TrackBinding {
            id: TrackId::new(),
            name: "Video 1".to_string(),
            kind: TrackKind::Video,
            node_ids: vec![],
        };
        let track_id = track.id;
        graph.tracks.push(track);

        // Add three clips: 0-100, 100-200, 200-300
        let clip1 = create_test_clip(0, 100);
        let clip1_id = clip1.id;

        let clip2 = create_test_clip(100, 100);
        let clip2_id = clip2.id;

        let clip3 = create_test_clip(200, 100);
        let clip3_id = clip3.id;

        graph.nodes.insert(clip1_id, clip1);
        graph.nodes.insert(clip2_id, clip2);
        graph.nodes.insert(clip3_id, clip3);

        graph.tracks[0].node_ids = vec![clip1_id, clip2_id, clip3_id];

        // Ripple move clip2 to start at 150 (50 frame delay)
        let result = ripple_move_clip(&mut graph, &clip2_id, 150);
        assert!(result.is_ok());

        // Check clip2 moved to 150
        if let Some(node) = graph.nodes.get(&clip2_id) {
            if let TimelineNodeKind::Clip(clip) = &node.kind {
                assert_eq!(clip.timeline_range.start, 150);
            }
        }

        // Check clip3 also moved by 50 frames
        if let Some(node) = graph.nodes.get(&clip3_id) {
            if let TimelineNodeKind::Clip(clip) = &node.kind {
                assert_eq!(clip.timeline_range.start, 250); // Originally 200, now 250
            }
        }
    }

    #[test]
    fn test_roll_edit() {
        let mut graph = TimelineGraph::default();

        let track = TrackBinding {
            id: TrackId::new(),
            name: "Video 1".to_string(),
            kind: TrackKind::Video,
            node_ids: vec![],
        };
        graph.tracks.push(track);

        // Two adjacent clips: 0-100, 100-200
        let clip1 = create_test_clip(0, 100);
        let clip1_id = clip1.id;

        let clip2 = create_test_clip(100, 100);
        let clip2_id = clip2.id;

        graph.nodes.insert(clip1_id, clip1);
        graph.nodes.insert(clip2_id, clip2);

        graph.tracks[0].node_ids = vec![clip1_id, clip2_id];

        // Roll edit to frame 120 (extend clip1 by 20, trim clip2 by 20)
        let result = roll_edit(&mut graph, &clip1_id, &clip2_id, 120);
        assert!(result.is_ok());

        // Check clip1 extended
        if let Some(node) = graph.nodes.get(&clip1_id) {
            if let TimelineNodeKind::Clip(clip) = &node.kind {
                assert_eq!(clip.timeline_range.duration, 120);
            }
        }

        // Check clip2 trimmed and moved
        if let Some(node) = graph.nodes.get(&clip2_id) {
            if let TimelineNodeKind::Clip(clip) = &node.kind {
                assert_eq!(clip.timeline_range.start, 120);
                assert_eq!(clip.timeline_range.duration, 80);
            }
        }
    }
}
