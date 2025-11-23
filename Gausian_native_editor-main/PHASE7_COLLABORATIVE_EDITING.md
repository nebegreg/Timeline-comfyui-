# Phase 7: Collaborative Editing - Implementation Guide
## Real-Time Multi-User Timeline Collaboration

---

## 🎯 Overview

Phase 7 implements **professional collaborative editing** with real-time synchronization, conflict resolution, and user presence tracking. Multiple editors can work on the same timeline simultaneously with automatic conflict resolution and live cursor tracking.

**Status:** 🔄 **Core Implementation Complete (60%)**

---

## 📦 Architecture

### CRDT-Based Synchronization

The collaboration system uses a **hybrid CRDT (Conflict-free Replicated Data Type)** approach:

1. **Operation-based CRDT**: Timeline modifications are represented as operations
2. **Vector clocks**: Track causality between operations
3. **Lamport timestamps**: Ensure total ordering of concurrent operations
4. **Operation log**: Maintains complete history for synchronization

### Components

```
crates/collaboration/
├── src/
│   ├── lib.rs              # Main exports & types
│   ├── operations.rs       # Timeline operations (Add/Remove/Move/Edit)
│   ├── crdt.rs            # CRDT timeline implementation
│   ├── presence.rs        # User presence tracking
│   ├── conflict.rs        # Conflict detection & resolution
│   └── sync.rs            # WebSocket sync protocol
```

---

## 🔧 Core Types

### Timeline Operations

All timeline modifications are represented as atomic operations:

```rust
pub enum OperationKind {
    // Node operations
    AddNode { node: TimelineNode },
    RemoveNode { node_id: NodeId },
    UpdateNodePosition { node_id: NodeId, new_start: Frame },
    UpdateNodeDuration { node_id: NodeId, new_range: FrameRange },

    // Track operations
    AddTrack { track: TrackBinding },
    RemoveTrack { track_id: TrackId },
    ReorderTracks { track_order: Vec<TrackId> },

    // Automation operations
    CreateAutomationLane { lane_id: LaneId, target_node: NodeId, parameter_path: String },
    AddKeyframe { lane_id: LaneId, keyframe: AutomationKeyframe },

    // Advanced edits
    RippleEdit { node_id: NodeId, new_start: Frame },
    RollEdit { left_node_id: NodeId, right_node_id: NodeId, new_edit_point: Frame },
    SlideEdit { node_id: NodeId, media_offset: Frame },
}
```

Each operation includes:
- **Operation ID**: UUID for unique identification
- **User ID**: Who created the operation
- **Lamport clock**: Causality tracking
- **Timestamp**: Client creation time
- **Parents**: Causal dependencies

### CRDT Timeline

```rust
pub struct CRDTTimeline {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub clock: LamportClock,
    pub operation_log: OperationLog,
    pub timeline: TimelineGraph,
    pub vector_clock: VectorClock,
    pending_operations: Vec<TimelineOperation>,
}
```

**Key Methods:**
- `apply_local_operation()` - Apply changes made by current user
- `apply_remote_operation()` - Apply changes from other users
- `merge()` - Merge states from different replicas
- `get_operations_since()` - Get operations for sync

---

## 🔀 Conflict Resolution

### Conflict Types

1. **Duplicate Delete**: Two users delete the same clip
   - **Resolution**: First deletion wins (idempotent)

2. **Concurrent Move**: Two users move the same clip
   - **Resolution**: Last-write-wins (based on Lamport clock)

3. **Property Conflict**: Two users edit the same property
   - **Resolution**: Configurable strategy (LWW, User Priority, Manual)

4. **Structural Conflict**: Incompatible graph modifications
   - **Resolution**: Manual resolution required

### Resolution Strategies

```rust
pub enum ResolutionStrategy {
    LastWriteWins,     // Use operation with highest clock
    UserPriority,      // Prefer operations from priority users
    Manual,            // Require manual resolution
    Merge,             // Attempt intelligent merging
}
```

---

## 👥 User Presence

### Presence Tracking

Track what each user is doing in real-time:

```rust
pub struct UserPresence {
    pub user: User,
    pub cursor_position: Option<CursorPosition>,
    pub selection: Option<Selection>,
    pub viewport: Option<Viewport>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
}
```

### Presence Updates

```rust
pub enum PresenceUpdate {
    UserJoined { user: User },
    UserLeft { user_id: UserId },
    CursorMoved { user_id: UserId, position: CursorPosition },
    SelectionChanged { user_id: UserId, selection: Selection },
    ViewportChanged { user_id: UserId, viewport: Viewport },
}
```

**UI Integration:**
- Color-coded user cursors
- Selection highlighting
- Viewport indicators
- Activity status (active/idle)

---

## 🌐 Sync Protocol

### WebSocket Messages

```rust
pub enum SyncMessage {
    // Connection
    Connect { session_id: SessionId, user: User, vector_clock: VectorClock },
    Connected { session_id: SessionId, user_id: UserId, initial_state: Vec<TimelineOperation> },

    // Operations
    Operation { operation: TimelineOperation },
    OperationAck { operation_id: OperationId },

    // Sync
    SyncRequest { since: VectorClock },
    SyncResponse { operations: Vec<TimelineOperation>, vector_clock: VectorClock },

    // Presence
    Presence { update: PresenceUpdate },

    // Heartbeat
    Ping, Pong,
}
```

### Session Management

**Server-side:**
```rust
pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
}

impl SessionManager {
    pub async fn create_session(&self, session_id: SessionId);
    pub async fn join_session(&self, session_id: SessionId, user: User);
    pub async fn broadcast_operation(&self, session_id: SessionId, operation: TimelineOperation);
}
```

**Client-side:**
```rust
pub struct SyncClient {
    session_id: SessionId,
    user_id: UserId,
    vector_clock: VectorClock,
    pending_operations: Vec<TimelineOperation>,
}

impl SyncClient {
    pub fn send_operation(&mut self, operation: TimelineOperation);
    pub fn send_presence(&self, update: PresenceUpdate);
    pub fn request_sync(&self);
}
```

---

## 🎨 UI Components (To Be Implemented)

### Cursor Indicators

Show other users' cursor positions:

```rust
fn render_remote_cursors(ui: &mut egui::Ui, presence_manager: &PresenceManager) {
    for user in presence_manager.get_active_users() {
        if let Some(cursor) = &user.cursor_position {
            let color = user.user.color.to_egui();
            ui.painter().circle_filled(
                cursor_screen_pos,
                4.0,
                color,
            );
            ui.painter().text(
                cursor_screen_pos + vec2(8.0, -8.0),
                Align2::LEFT_BOTTOM,
                &user.user.name,
                FontId::default(),
                color,
            );
        }
    }
}
```

### Selection Highlighting

```rust
fn render_remote_selections(ui: &mut egui::Ui, presence_manager: &PresenceManager) {
    for user in presence_manager.get_active_users() {
        if let Some(selection) = &user.selection {
            for node_id in &selection.node_ids {
                // Render selection outline with user's color
                let color = user.user.color.to_egui().linear_multiply(0.5);
                ui.painter().rect_stroke(
                    node_rect,
                    2.0,
                    egui::Stroke::new(2.0, color),
                );
            }
        }
    }
}
```

---

## 📊 Performance Characteristics

### Operation Complexity

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Apply local operation | O(1) | Append to log |
| Apply remote operation | O(n) parents check | n = parent count |
| Merge timelines | O(m log m) | m = operation count |
| Conflict detection | O(1) | Per operation pair |
| Broadcast to N users | O(N) | WebSocket send |

### Memory Usage

- **Operation log**: ~1KB per operation
- **Vector clock**: 16 bytes per user
- **Presence data**: ~200 bytes per user

**Optimization**: Operation log compaction after 1000 operations

---

## 🧪 Testing

### Unit Tests (13/13 Passing ✅)

```bash
cargo test -p collaboration
```

**Coverage:**
- ✅ CRDT operations (local & remote)
- ✅ Vector clock causality
- ✅ Conflict detection
- ✅ Conflict resolution (LWW)
- ✅ Session management
- ✅ Broadcast messaging
- ✅ Presence tracking
- ✅ User color generation

### Integration Tests (Pending)

- [ ] Multi-user editing scenarios
- [ ] Network partition recovery
- [ ] Large operation log handling
- [ ] Concurrent conflict resolution

---

## 🚀 Usage Example

### Client Setup

```rust
use collaboration::*;

// Create CRDT timeline
let session_id = SessionId::new();
let user_id = UserId::new();
let mut crdt = CRDTTimeline::new(session_id, user_id);

// Apply local edit
let node = create_clip(0, 100);
let op = crdt.apply_local_operation(OperationKind::AddNode { node })?;

// Send to server
sync_client.send_operation(op).await?;

// Receive remote operation
let remote_op = sync_client.receive_message().await;
crdt.apply_remote_operation(remote_op)?;
```

### Server Setup

```rust
use collaboration::*;

let server = CollaborationServer::new();

// Create session
server.create_session(session_id).await?;

// User joins
let rx = server.join_session(session_id, user).await?;

// Handle messages
while let Some(msg) = rx.recv().await {
    server.handle_message(session_id, user_id, msg).await?;
}
```

---

## 🔜 Remaining Work (40%)

### High Priority

1. **WebSocket Server Implementation** (1-2 weeks)
   - Production-ready sync server
   - Connection pooling & scaling
   - Persistent session storage
   - Authentication & authorization

2. **UI Integration** (2-3 weeks)
   - Remote cursor rendering
   - Selection highlighting
   - Conflict resolution UI
   - User list panel
   - Activity indicators

3. **Marker Support** (3-4 days)
   - Add markers to TimelineGraph
   - Implement marker operations
   - Collaborative marker editing

### Medium Priority

4. **Offline Support** (1 week)
   - Operation queue for offline edits
   - Automatic sync on reconnect
   - Merge conflict resolution

5. **Performance Optimization** (3-4 days)
   - Operation log compaction
   - Delta synchronization
   - Binary protocol (instead of JSON)

6. **Advanced Conflict Resolution** (1 week)
   - Intelligent property merging
   - Three-way merge for timeline structure
   - User-guided resolution UI

### Low Priority

7. **Collaboration Analytics** (2-3 days)
   - Edit history visualization
   - User contribution tracking
   - Session replay

8. **Permission System** (3-4 days)
   - Read-only collaborators
   - Track-level permissions
   - Admin/editor roles

---

## 🏗️ Architecture Decisions

### Why CRDT?

**Alternative:** Operational Transform (OT)
- **CRDT Pros:**
  - Simpler to implement correctly
  - No central server required for correctness
  - Automatic commutativity
- **OT Pros:**
  - More compact operation history
  - Better UX for text editing
- **Choice:** CRDT for timeline (not text-heavy)

### Why Operation-Based?

**Alternative:** State-based CRDT
- **Operation-Based Pros:**
  - Smaller sync payloads
  - Easier debugging (operation history)
  - Better for undo/redo
- **State-Based Pros:**
  - Simpler merge logic
  - Idempotent sync
- **Choice:** Operation-based for better UX

### Why WebSocket?

**Alternative:** HTTP polling, Server-Sent Events (SSE)
- **WebSocket Pros:**
  - Bidirectional real-time
  - Low latency (<10ms)
  - Efficient for high-frequency updates
- **Choice:** WebSocket for professional feel

---

## 📚 Dependencies

```toml
[dependencies]
automerge = "0.5"           # CRDT library
tokio = "1.35"              # Async runtime
tokio-tungstenite = "0.21"  # WebSocket
serde = "1.0"               # Serialization
uuid = "1.6"                # Unique IDs
chrono = "0.4"              # Timestamps
```

---

## 🎓 Learning Resources

### CRDT Papers
- [Conflict-free Replicated Data Types](https://hal.inria.fr/inria-00609399/document)
- [Automerge: Real-Time Data Sync](https://automerge.org)

### Collaboration Patterns
- [Google Docs Collaboration](https://www.youtube.com/watch?v=uOFzWZrsPV0)
- [Figma Multiplayer Tech](https://www.figma.com/blog/how-figmas-multiplayer-technology-works/)

---

## ✅ Completion Checklist

**Core Implementation:**
- [x] CRDT timeline structure
- [x] Operation types & application
- [x] Vector clock causality
- [x] Conflict detection
- [x] Conflict resolution (LWW, User Priority)
- [x] User presence tracking
- [x] WebSocket sync protocol
- [x] Session management
- [x] Unit tests (13 passing)

**Remaining:**
- [ ] Production WebSocket server
- [ ] UI integration (cursors, selections)
- [ ] Marker collaboration
- [ ] Offline support
- [ ] Operation log compaction
- [ ] Advanced conflict resolution UI
- [ ] Integration tests
- [ ] Performance benchmarks

---

**Phase 7 Status:** 60% Complete

**Estimated Time to v1.0:** 4-6 weeks

---

## 🐛 Known Issues

1. **Markers not in TimelineGraph** - Marker operations are stubbed (TODO)
2. **No operation compaction** - Operation log grows unbounded
3. **Binary protocol** - Currently using JSON (slower)
4. **No persistence** - Sessions lost on server restart

---

**Next Steps:** Implement production WebSocket server & UI integration
