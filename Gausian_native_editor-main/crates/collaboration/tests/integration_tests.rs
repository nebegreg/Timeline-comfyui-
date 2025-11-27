/// Phase 7: Collaborative Editing - Integration Tests
/// Tests for multi-user scenarios, conflict resolution, and network recovery
use collaboration::*;
use timeline::{ClipNode, FrameRange, Marker, NodeId, TimelineNode, TimelineNodeKind};

#[tokio::test]
async fn test_two_users_basic_editing() {
    // User 1 creates a timeline
    let session_id = SessionId::new();
    let user1_id = UserId::new();
    let mut user1 = CRDTTimeline::new(session_id, user1_id);

    // User 2 joins the session
    let user2_id = UserId::new();
    let mut user2 = CRDTTimeline::new(session_id, user2_id);

    // User 1 adds a clip
    let clip = TimelineNode {
        id: NodeId::new(),
        label: Some("Clip 1".to_string()),
        kind: TimelineNodeKind::Clip(ClipNode {
            asset_id: Some("/test/clip1.mp4".to_string()),
            timeline_range: FrameRange {
                start: 0,
                duration: 100,
            },
            media_range: FrameRange {
                start: 0,
                duration: 100,
            },
            playback_rate: 1.0,
            reverse: false,
            metadata: serde_json::Value::Null,
        }),
        locked: false,
        metadata: serde_json::Value::Null,
    };

    let op1 = user1
        .apply_local_operation(OperationKind::AddNode { node: clip.clone() })
        .unwrap();

    // User 2 receives the operation
    user2.apply_remote_operation(op1.clone()).unwrap();

    // Both timelines should have the same clip
    assert!(user1.timeline.nodes.contains_key(&clip.id));
    assert!(user2.timeline.nodes.contains_key(&clip.id));
}

#[tokio::test]
async fn test_concurrent_marker_additions() {
    let session_id = SessionId::new();
    let user1_id = UserId::new();
    let user2_id = UserId::new();

    let mut user1 = CRDTTimeline::new(session_id, user1_id);
    let mut user2 = CRDTTimeline::new(session_id, user2_id);

    // User 1 adds a marker at frame 100
    let marker1 = Marker::new(100, "User 1 Marker".to_string());
    let op1 = user1
        .apply_local_operation(OperationKind::AddMarker {
            marker: marker1.clone(),
        })
        .unwrap();

    // User 2 adds a marker at frame 200
    let marker2 = Marker::new(200, "User 2 Marker".to_string());
    let op2 = user2
        .apply_local_operation(OperationKind::AddMarker {
            marker: marker2.clone(),
        })
        .unwrap();

    // Exchange operations
    user2.apply_remote_operation(op1).unwrap();
    user1.apply_remote_operation(op2).unwrap();

    // Both users should have both markers
    assert_eq!(user1.timeline.markers.len(), 2);
    assert_eq!(user2.timeline.markers.len(), 2);
    assert!(user1.timeline.markers.contains_key(&marker1.id));
    assert!(user1.timeline.markers.contains_key(&marker2.id));
}

#[tokio::test]
async fn test_conflict_resolution_last_write_wins() {
    let session_id = SessionId::new();
    let user1_id = UserId::new();
    let user2_id = UserId::new();

    let mut user1 = CRDTTimeline::new(session_id, user1_id);
    let mut user2 = CRDTTimeline::new(session_id, user2_id);

    // Both users add the same marker through proper operations
    let marker = Marker::new(100, "Original".to_string());
    let add_op = user1
        .apply_local_operation(OperationKind::AddMarker {
            marker: marker.clone(),
        })
        .unwrap();

    // User 2 receives the add operation
    user2.apply_remote_operation(add_op).unwrap();

    // Now both users have the marker. User 1 updates it
    let op1 = user1
        .apply_local_operation(OperationKind::UpdateMarker {
            marker_id: marker.id,
            new_frame: 150,
            new_label: Some("User 1 Update".to_string()),
        })
        .unwrap();

    // User 2 also updates the marker (conflict!)
    let op2 = user2
        .apply_local_operation(OperationKind::UpdateMarker {
            marker_id: marker.id,
            new_frame: 200,
            new_label: Some("User 2 Update".to_string()),
        })
        .unwrap();

    // Detect conflict
    assert!(op1.kind.conflicts_with(&op2.kind));

    // Apply both operations
    user1.apply_remote_operation(op2.clone()).unwrap();
    user2.apply_remote_operation(op1.clone()).unwrap();

    // Both users should have the marker (conflict resolved by operations)
    assert!(user1.timeline.markers.contains_key(&marker.id));
    assert!(user2.timeline.markers.contains_key(&marker.id));
}

#[tokio::test]
async fn test_operation_log_compaction() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let mut crdt = CRDTTimeline::new(session_id, user_id);

    // Add many operations
    for i in 0..150 {
        let marker = Marker::new(i as i64, format!("Marker {}", i));
        crdt.apply_local_operation(OperationKind::AddMarker { marker })
            .unwrap();
    }

    assert_eq!(crdt.operation_log.operations.len(), 150);

    // Compact the log
    crdt.operation_log.compact(&crdt.timeline, 100);

    // Should keep only the last 100 operations
    assert_eq!(crdt.operation_log.operations.len(), 100);

    // Timeline should still be correct
    assert_eq!(crdt.timeline.markers.len(), 150);
}

#[tokio::test]
async fn test_operation_log_optimization() {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let mut crdt = CRDTTimeline::new(session_id, user_id);

    // Add a node
    let node = TimelineNode {
        id: NodeId::new(),
        label: Some("Test Clip".to_string()),
        kind: TimelineNodeKind::Clip(ClipNode {
            asset_id: Some("/test/clip.mp4".to_string()),
            timeline_range: FrameRange {
                start: 0,
                duration: 100,
            },
            media_range: FrameRange {
                start: 0,
                duration: 100,
            },
            playback_rate: 1.0,
            reverse: false,
            metadata: serde_json::Value::Null,
        }),
        locked: false,
        metadata: serde_json::Value::Null,
    };

    crdt.apply_local_operation(OperationKind::AddNode { node: node.clone() })
        .unwrap();

    // Update position multiple times
    for i in 0..10 {
        crdt.apply_local_operation(OperationKind::UpdateNodePosition {
            node_id: node.id,
            new_start: i * 10,
        })
        .unwrap();
    }

    assert_eq!(crdt.operation_log.operations.len(), 11); // 1 add + 10 updates

    // Optimize the log
    crdt.operation_log.optimize();

    // Should merge consecutive position updates
    assert!(crdt.operation_log.operations.len() < 11);
}

#[tokio::test]
async fn test_offline_queue() {
    use collaboration::OfflineQueueManager;
    use std::path::PathBuf;

    let session_id = SessionId::new();
    let user_id = UserId::new();

    let storage_dir = PathBuf::from("/tmp/collab_test_offline");
    let mut manager = OfflineQueueManager::new(&storage_dir);

    // Start queue
    manager.start_queue(session_id, user_id);

    // Add operations while offline
    for i in 0..5 {
        let marker = Marker::new(i * 10, format!("Offline {}", i));
        let op = TimelineOperation::new(
            user_id,
            LamportClock::new(),
            OperationKind::AddMarker { marker },
        );
        manager.enqueue(op).unwrap();
    }

    assert_eq!(manager.pending_count(), 5);

    // Save to disk
    manager.save_queue().await.unwrap();

    // Create new manager and load
    let mut manager2 = OfflineQueueManager::new(&storage_dir);
    manager2.load_queue(session_id).await.unwrap();

    assert_eq!(manager2.pending_count(), 5);

    // Drain and cleanup
    let ops = manager2.drain_pending();
    assert_eq!(ops.len(), 5);
    manager2.delete_saved_queue(session_id).await.unwrap();
}

#[tokio::test]
async fn test_presence_tracking() {
    let session_id = SessionId::new();
    let user1 = User::new(UserId::new(), "Alice".to_string());
    let user2 = User::new(UserId::new(), "Bob".to_string());

    let mut manager = PresenceManager::new();

    // Add users
    manager.update_user(UserPresence::new(user1.clone(), session_id));
    manager.update_user(UserPresence::new(user2.clone(), session_id));

    assert_eq!(manager.get_active_users().len(), 2);

    // Update cursor
    manager.update_cursor(
        user1.id,
        CursorPosition {
            frame: 100,
            track_index: Some(0),
        },
    );

    let presence = manager.get_user(&user1.id).unwrap();
    assert_eq!(presence.cursor_position.as_ref().unwrap().frame, 100);

    // Remove user
    manager.remove_user(&user2.id);
    assert_eq!(manager.get_all_users().len(), 1);
}

#[tokio::test]
async fn test_merge_from_different_branches() {
    let session_id = SessionId::new();
    let user1_id = UserId::new();
    let user2_id = UserId::new();

    // User 1's timeline
    let mut user1 = CRDTTimeline::new(session_id, user1_id);

    // User 2's timeline (starts from same state)
    let mut user2 = user1.clone();
    user2.user_id = user2_id;

    // User 1 adds a marker
    let marker1 = Marker::new(100, "User 1".to_string());
    user1
        .apply_local_operation(OperationKind::AddMarker {
            marker: marker1.clone(),
        })
        .unwrap();

    // User 2 adds a different marker
    let marker2 = Marker::new(200, "User 2".to_string());
    user2
        .apply_local_operation(OperationKind::AddMarker {
            marker: marker2.clone(),
        })
        .unwrap();

    // Merge user2's changes into user1
    user1.merge(&user2).unwrap();

    // User 1 should have both markers
    assert_eq!(user1.timeline.markers.len(), 2);
    assert!(user1.timeline.markers.contains_key(&marker1.id));
    assert!(user1.timeline.markers.contains_key(&marker2.id));
}

#[test]
fn test_vector_clock_causality() {
    let mut clock1 = VectorClock::new();
    let mut clock2 = VectorClock::new();

    let user1 = UserId::new();
    let user2 = UserId::new();

    // User 1 makes changes
    clock1.increment(user1);
    clock1.increment(user1);

    // User 2 makes changes
    clock2.increment(user2);

    // Merge clocks
    clock1.merge(&clock2);

    assert_eq!(clock1.get(user1), 2);
    assert_eq!(clock1.get(user2), 1);

    // Check concurrency
    let mut clock3 = VectorClock::new();
    clock3.increment(user1);
    clock3.increment(user2);
    clock3.increment(user2);

    assert!(clock1.is_concurrent(&clock3));
}
