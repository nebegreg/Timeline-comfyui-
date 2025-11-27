# 🛍️ Marketplace Integration Guide - Final 5%

**Status:** Phase 5 at 95% - Almost Complete!
**Remaining:** UI Integration + Testing (5%)

---

## ✅ What's Complete (95%)

### Backend Infrastructure (100%)
- ✅ REST API server (`apps/marketplace-server/`) with all endpoints
- ✅ JSON storage with persistence and seeding
- ✅ Search, filtering, sorting, pagination
- ✅ Rating/review system
- ✅ Download tracking and statistics

### Client Library (100%)
- ✅ `PluginMarketplace` client (`crates/plugin-host/src/marketplace.rs`)
- ✅ Async HTTP operations with reqwest
- ✅ Plugin installation (download, extract, verify)
- ✅ SHA256 checksum verification
- ✅ Update checking
- ✅ Plugin submission for developers

### UI Components (100%)
- ✅ `marketplace_ui.rs` - Full marketplace browsing interface
- ✅ `plugin_details.rs` - Detailed plugin information panel
- ✅ Search, filters, sorting, pagination UI
- ✅ Plugin cards with ratings and stats

### **NEW! Async Integration Bridge (100%)**
- ✅ `marketplace_manager.rs` - Async-to-sync bridge
- ✅ Channel-based communication (mpsc)
- ✅ Background thread with tokio runtime
- ✅ Command/Response pattern for UI integration

---

## 🚧 Remaining Work (5%)

### 1. Integrate Manager into App Struct

**File:** `apps/desktop/src/app.rs`

**Add to App struct:**
```rust
pub struct App {
    // ... existing fields ...

    // Phase 5: Plugin Marketplace
    marketplace_manager: Option<MarketplaceManager>,
}
```

**Initialize in `App::new()`:**
```rust
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // ... existing initialization ...

        // Initialize marketplace manager
        let marketplace_manager = Some(MarketplaceManager::new(
            "http://127.0.0.1:3000".to_string()
        ));

        Self {
            // ... existing fields ...
            marketplace_manager,
        }
    }
}
```

**Poll for responses in `update()`:**
```rust
fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    // ... existing update code ...

    // Phase 5: Poll marketplace responses
    if let Some(manager) = &self.marketplace_manager {
        let responses = manager.poll_responses();
        for response in responses {
            self.handle_marketplace_response(response);
        }
    }
}
```

### 2. Update MarketplaceUI to Use Manager

**File:** `apps/desktop/src/marketplace_ui.rs`

**Changes needed:**

**Add command sender field:**
```rust
pub struct MarketplaceUI {
    // ... existing fields ...

    /// Command sender to marketplace manager
    command_sender: Option<Sender<MarketplaceCommand>>,
}
```

**Replace mock data with real commands in `refresh_plugins()`:**
```rust
fn refresh_plugins(&mut self) {
    self.connection_status = ConnectionStatus::Connecting;

    // Build search query
    let query = SearchQuery {
        query: if self.search_query.is_empty() { None } else { Some(self.search_query.clone()) },
        category: self.selected_category.clone(),
        plugin_type: self.selected_type.clone(),
        sort_by: match self.sort_by {
            SortMode::Downloads => plugin_host::marketplace::SortBy::Downloads,
            SortMode::Rating => plugin_host::marketplace::SortBy::Rating,
            SortMode::Recent => plugin_host::marketplace::SortBy::Updated,
        },
        page: self.current_page as u32,
        per_page: 20,
        free_only: false,
        tags: vec![],
    };

    // Send command to manager
    if let Some(sender) = &self.command_sender {
        let _ = sender.send(MarketplaceCommand::Search(query));
    }
}
```

**Replace mock install with real command in `install_plugin()`:**
```rust
fn install_plugin(&mut self, plugin_id: String) {
    self.installing = Some(plugin_id.clone());

    if let Some(sender) = &self.command_sender {
        let _ = sender.send(MarketplaceCommand::InstallPlugin { plugin_id });
    }
}
```

### 3. Handle Marketplace Responses in App

**File:** `apps/desktop/src/app.rs`

**Add response handler method:**
```rust
impl App {
    fn handle_marketplace_response(&mut self, response: MarketplaceResponse) {
        match response {
            MarketplaceResponse::SearchResults(results) => {
                self.marketplace_ui.results = Some(results);
                self.marketplace_ui.connection_status = ConnectionStatus::Connected;
            }

            MarketplaceResponse::InstallProgress { plugin_id, progress } => {
                // Update UI progress indicator
                tracing::info!("Plugin {} install progress: {:.0}%", plugin_id, progress * 100.0);
            }

            MarketplaceResponse::InstallComplete { plugin_id, success, error } => {
                self.marketplace_ui.installing = None;
                if success {
                    self.marketplace_ui.installed_plugins.insert(plugin_id, "1.0.0".to_string());
                    tracing::info!("Plugin installed successfully");
                } else {
                    tracing::error!("Plugin install failed: {:?}", error);
                    self.marketplace_ui.connection_status = ConnectionStatus::Error(
                        error.unwrap_or_else(|| "Unknown error".to_string())
                    );
                }
            }

            MarketplaceResponse::PluginDetails(plugin) => {
                self.marketplace_ui.selected_plugin = Some(plugin);
            }

            MarketplaceResponse::FeaturedPlugins(plugins) => {
                // Handle featured plugins list
                tracing::info!("Received {} featured plugins", plugins.len());
            }

            MarketplaceResponse::UpdatesAvailable(updates) => {
                tracing::info!("{} plugin updates available", updates.len());
            }

            MarketplaceResponse::Error { message } => {
                tracing::error!("Marketplace error: {}", message);
                self.marketplace_ui.connection_status = ConnectionStatus::Error(message);
            }
        }
    }
}
```

### 4. Connect Manager to UI

**In App initialization, pass command sender to UI:**
```rust
// In App::new() after creating marketplace_manager
if let Some(manager) = &self.marketplace_manager {
    self.marketplace_ui.set_command_sender(manager.get_sender());
}
```

**Add method to MarketplaceManager:**
```rust
impl MarketplaceManager {
    pub fn get_sender(&self) -> Sender<MarketplaceCommand> {
        self.command_tx.clone()
    }
}
```

**Add method to MarketplaceUI:**
```rust
impl MarketplaceUI {
    pub fn set_command_sender(&mut self, sender: Sender<MarketplaceCommand>) {
        self.command_sender = Some(sender);
    }
}
```

### 5. Add Loading States

**Update ConnectionStatus usage in marketplace_ui.rs:**
```rust
// Show loading spinner during searches
match self.connection_status {
    ConnectionStatus::Connecting => {
        ui.spinner();
        ui.label("Loading plugins...");
    }
    ConnectionStatus::Connected => {
        // Show results
    }
    ConnectionStatus::Error(ref err) => {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
    }
    _ => {}
}
```

### 6. Testing

**Test the full flow:**
1. Start marketplace server: `cargo run --bin marketplace-server`
2. Start desktop app: `cargo run --bin desktop`
3. Open marketplace panel
4. Test search with filters
5. Test plugin installation
6. Verify error handling

**Expected behavior:**
- Search results load from server
- Plugin cards display real data
- Install downloads and extracts plugins
- Progress updates show during install
- Errors are handled gracefully

---

## 📚 Plugin SDK Documentation (2%)

**Create:** `PLUGIN_SDK_GUIDE.md`

**Contents:**
1. **Quick Start** - Create your first plugin in 5 minutes
2. **Plugin Types** - WASM vs Python plugins
3. **Manifest Format** - plugin.json structure
4. **API Reference** - Available host functions
5. **Examples** - Step-by-step tutorials
6. **Publishing** - How to submit to marketplace
7. **Security** - Signing and verification
8. **Testing** - Local testing workflow

---

## 🎯 Success Criteria for 100%

- [x] Backend server running and seeded
- [x] Client library complete
- [x] UI components complete
- [x] Async bridge complete
- [ ] UI connected to real backend (no mock data)
- [ ] End-to-end plugin install working
- [ ] Loading states and error handling
- [ ] Plugin SDK documentation
- [ ] Integration tested

---

## 📝 Estimated Time to Complete

**Remaining 5%:**
- UI Integration: 2-3 hours
- Testing: 1 hour
- Documentation: 2 hours

**Total:** ~5-6 hours to reach 100%

---

## 💡 Tips

1. **Test early, test often** - Start the marketplace server first
2. **Use logging** - Add tracing::info! to track data flow
3. **Handle errors gracefully** - Always provide user feedback
4. **Keep UI responsive** - Async bridge ensures non-blocking operations

---

**Phase 5: 95% Complete** - Final push to 100%! 🚀
