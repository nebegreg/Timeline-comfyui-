# 🚀 Plan d'Implémentation - Toutes les Phases

Ce document détaille le plan d'exécution concret pour implémenter toute la roadmap.

## ✅ Phase 1: Timeline Polish & UX (EN COURS)

### Structures Créées
- ✅ `apps/desktop/src/selection.rs` - Système sélection multiple
- ✅ `apps/desktop/src/edit_modes.rs` - Modes édition (Ripple, Roll, Slide, Slip)
- ✅ `apps/desktop/src/keyboard.rs` - Raccourcis clavier professionnels
- ✅ `crates/timeline/src/markers.rs` - Marqueurs et régions

### Prochaines Étapes Phase 1
1. ⏳ Intégrer selection.rs dans app.rs
2. ⏳ Intégrer edit_modes.rs dans timeline/ui.rs
3. ⏳ Implémenter ripple edit logic
4. ⏳ Intégrer keyboard.rs pour gestion événements
5. ⏳ UI pour marqueurs (visualization + interaction)
6. ⏳ Optimisations performance (culling, LOD waveforms)
7. ⏳ Tests unitaires

---

## 📅 Phase 2: Rich Effects & Transitions

### Structure Proposée
```
crates/
  effects/           # Nouveau crate
    Cargo.toml
    src/
      lib.rs         # Trait Effect + EffectManager
      parameters.rs  # Système de paramètres
      stack.rs       # Effect stack per clip

      # Basic corrections
      brightness_contrast.rs
      saturation_hue.rs
      exposure_gamma.rs

      # Advanced
      curves.rs
      color_wheels.rs
      vignette.rs

      # Stylized
      blur.rs
      sharpen.rs
      film_grain.rs
      chromatic_aberration.rs

      # Geometric
      transform.rs
      crop.rs
      corner_pin.rs

      # Compositing
      chroma_key.rs
      blend_modes.rs

      shaders/       # WGSL shaders
        brightness_contrast.wgsl
        blur.wgsl
        chroma_key.wgsl
        ...

  transitions/       # Nouveau crate
    Cargo.toml
    src/
      lib.rs         # Trait Transition
      dissolve.rs
      wipe.rs
      slide.rs
      zoom.rs
      spin.rs
      shaders/
        dissolve.wgsl
        wipe.wgsl
        ...
```

### Dépendances à Ajouter
```toml
# crates/effects/Cargo.toml
[dependencies]
wgpu = "22"
bytemuck = { version = "1", features = ["derive"] }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Pour curves
lyon = "1.0"  # Bézier curves
```

### Plan d'Implémentation Phase 2
1. Créer crate `effects` avec trait de base
2. Implémenter 3 effets simples (brightness, blur, transform)
3. Intégrer dans renderer pipeline
4. UI effect stack dans inspector panel
5. Implémenter 12 effets restants
6. Créer crate `transitions`
7. Implémenter 5 transitions de base
8. UI timeline pour transitions
9. Tests + documentation

---

## 📅 Phase 3: Color Management & LUTs

### Structure Proposée
```
crates/
  color/            # Nouveau crate
    Cargo.toml
    src/
      lib.rs
      lut3d.rs      # Parser + GPU upload
      color_spaces.rs # Matrices conversion
      aces.rs       # ACES workflow
      scopes.rs     # Waveform, vectorscope, histogram

      parsers/
        cube.rs     # .cube parser
        threedl.rs  # .3dl parser
        csp.rs      # .csp parser

      shaders/
        lut_apply.wgsl
        scope_waveform.wgsl
        scope_vectorscope.wgsl
```

### Dépendances à Ajouter
```toml
# crates/color/Cargo.toml
[dependencies]
nom = "7"           # Parser combinators
wgpu = "22"
egui_plot = "0.29" # Pour scopes
glam = "0.27"      # Math matrices
```

### Plan d'Implémentation Phase 3
1. Parser .cube files (nom)
2. Créer texture 3D GPU
3. Shader LUT application
4. UI pour importer/appliquer LUTs
5. Implémenter ACES transforms
6. Compute shaders pour scopes
7. UI panel scopes avec egui_plot
8. Parser .3dl et .csp
9. Tests + validation LUTs

---

## 📅 Phase 4: Automatic LORA Creator

### Structure Proposée
```
crates/
  ai-pipeline/      # Nouveau crate
    Cargo.toml
    src/
      lib.rs
      lora_creator.rs
      dataset.rs    # Frame extraction + preprocessing
      caption.rs    # Auto-captioning avec BLIP2
      training.rs   # Training orchestration

      backends/
        candle.rs   # Local Candle training
        comfyui.rs  # ComfyUI workflow
        replicate.rs # Replicate API
        modal.rs    # Modal Functions

apps/desktop/src/
  lora/             # Nouveau module
    ui.rs           # UI training configuration
    workflow.rs     # Workflow management
```

### Dépendances à Ajouter
```toml
# crates/ai-pipeline/Cargo.toml
[dependencies]
candle-core = "0.4"
candle-nn = "0.4"
candle-transformers = "0.4"
tokenizers = "0.15"
image = "0.25"
reqwest = { version = "0.11", features = ["json", "multipart"] }
serde_json = "1"
```

### Plan d'Implémentation Phase 4
1. Système extraction frames de timeline
2. Preprocessing (resize, crop, normalize)
3. Intégration BLIP2 pour captions (via API ou local)
4. Backend ComfyUI (workflow JSON template)
5. UI configuration training
6. Job queue integration
7. Progress monitoring WebSocket
8. Download + sauvegarde LoRA
9. Preview gallery
10. Tests avec modèles réels

---

## 📅 Phase 5: Plugin Marketplace

### Structure Proposée
```
crates/
  plugin-sdk/       # Nouveau crate (SDK public)
    Cargo.toml
    src/
      lib.rs        # Exports pour plugin authors
      macros.rs     # Proc macros
    examples/
      blur_effect.rs
      color_grading.rs

relay/              # Étendre serveur existant
  src/
    marketplace/
      api.rs        # REST API
      database.rs   # PostgreSQL
      storage.rs    # S3/MinIO pour archives
      verify.rs     # Signature verification

apps/desktop/src/
  marketplace/
    ui.rs           # Browse/search UI
    install.rs      # Download + install
    manage.rs       # Manage installed
```

### Dépendances à Ajouter
```toml
# Compléter crates/plugin-host/Cargo.toml
[dependencies]
wasmtime = "17"     # Ajouter
pyo3 = { version = "0.20", features = ["auto-initialize"] }
libloading = "0.8"  # Native plugins
ed25519-dalek = "2" # Signatures
```

### Plan d'Implémentation Phase 5
1. Finaliser WASM execution (wasmtime)
2. Finaliser Python bridge (pyo3)
3. Plugin SDK documentation
4. 5 plugins exemples
5. Backend marketplace (Axum + PostgreSQL)
6. UI browse/search
7. Signature verification
8. Auto-updates
9. Tests sécurité (sandboxing)
10. Deploy backend

---

## 📅 Phase 6: Multi-Window Workspace

### Structure Proposée
```
apps/desktop/src/
  workspace/
    mod.rs          # Workspace manager
    windows.rs      # Window state management
    layouts.rs      # Predefined layouts
    persistence.rs  # Save/restore layouts

  windows/          # Individual window types
    timeline.rs
    viewer.rs
    assets.rs
    inspector.rs
    effects.rs
    color_grading.rs
    audio_mixer.rs
```

### Dépendances à Ajouter
```toml
# apps/desktop/Cargo.toml
# winit déjà présent, activer multi-window feature
winit = { version = "0.30", features = ["multi-window"] }
```

### Plan d'Implémentation Phase 6
1. Refactor app.rs pour multi-window
2. WindowManager avec HashMap<WindowId, WindowState>
3. Event loop par fenêtre
4. Layouts prédéfinis (JSON config)
5. UI pour spawn/close windows
6. Persistence dans DB
7. Multi-monitor support
8. Tests cross-platform
9. Documentation layouts

---

## 📅 Phase 7: Collaborative Editing

### Structure Proposée
```
crates/
  collaboration/    # Nouveau crate
    Cargo.toml
    src/
      lib.rs
      crdt.rs       # Automerge wrapper
      sync.rs       # Sync protocols
      peer.rs       # Peer management
      locks.rs      # Optimistic locking
      conflicts.rs  # Conflict resolution

relay/              # Backend server
  src/
    collab/
      websocket.rs  # WebSocket handler
      rooms.rs      # Project rooms
      presence.rs   # User presence
      changes.rs    # Change log

apps/desktop/src/
  collab/
    ui.rs           # Collaboration panel
    cursors.rs      # Render peer cursors
    activity.rs     # Activity feed
```

### Dépendances à Ajouter
```toml
# crates/collaboration/Cargo.toml
[dependencies]
automerge = "0.5"
tungstenite = "0.21"  # Déjà présent
tokio = { version = "1", features = ["full"] }
```

### Nouvelles Migrations DB
```sql
-- V0010__collaboration.sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  display_name TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE project_collaborators (
  project_id TEXT,
  user_id TEXT,
  role TEXT CHECK(role IN ('owner','editor','viewer')),
  PRIMARY KEY (project_id, user_id)
);

CREATE TABLE change_log (
  id TEXT PRIMARY KEY,
  project_id TEXT,
  user_id TEXT,
  change_type TEXT,
  change_data TEXT,
  timestamp INTEGER,
  automerge_hash TEXT
);

CREATE TABLE locks (
  resource_id TEXT PRIMARY KEY,
  user_id TEXT,
  acquired_at INTEGER,
  expires_at INTEGER
);
```

### Plan d'Implémentation Phase 7
1. Intégrer Automerge CRDT
2. Backend WebSocket server
3. Change log DB
4. Sync protocol (send/receive changes)
5. UI collaborative cursors
6. UI locks visualization
7. Activity feed
8. Chat integration
9. Conflict resolution UI
10. Tests multi-user
11. Deploy backend
12. Load testing

---

## 📅 Phase 8: Animation & Keyframing

### Structure Proposée
```
crates/timeline/src/
  # Déjà partiellement défini dans graph.rs
  # Étendre:
  automation.rs     # Automation lanes logic
  keyframes.rs      # Keyframe interpolation
  curves.rs         # Bézier curve math

apps/desktop/src/
  keyframe_editor/
    ui.rs           # Graph editor UI
    handles.rs      # Bézier handles
    curves.rs       # Curve visualization
```

### Dépendances à Ajouter
```toml
# crates/timeline/Cargo.toml
cubic-spline = "0.3"  # Interpolation
lyon = "1.0"          # Déjà dans effects
```

### Plan d'Implémentation Phase 8
1. Implémenter interpolation (linear, Bézier)
2. UI automation lanes (expandable sous clips)
3. Graph editor (egui_plot)
4. Bézier handles interactifs
5. Copier/coller keyframes
6. Apply automation au rendu
7. Presets keyframes (ease-in, etc.)
8. Tests interpolation
9. Documentation

---

## 🎯 Ordre d'Exécution Recommandé

### Séquence Optimale
1. ✅ **Phase 1** (6-8 sem) - EN COURS - Fondation UX
2. 🔄 **Phase 2** (10-12 sem) - NEXT - Effets demandés
3. 🔄 **Phase 3** (6-8 sem) - Color management
4. 🔄 **Phase 8** (6 sem) - Keyframing (complète effets)
5. 🔄 **Phase 5** (10-12 sem) - Plugin marketplace
6. 🔄 **Phase 4** (8-10 sem) - LORA creator (si cible IA)
7. 🔄 **Phase 7** (16-20 sem) - Collaboration (gros projet)
8. 🔄 **Phase 6** (6-8 sem) - Multi-window (polish final)

### Parallélisation Possible
- Phase 2 + Phase 3 peuvent être développées en parallèle (équipes différentes)
- Phase 4 peut démarrer pendant Phase 3 (modules indépendants)
- Phase 5 (backend marketplace) peut démarrer tôt en parallèle

---

## 📊 Milestones & Releases

### v0.2.0 - Timeline Pro (Post Phase 1)
- Multi-selection
- Ripple/Roll edits
- Marqueurs & régions
- Raccourcis J/K/L

### v0.3.0 - Effects Core (Post Phase 2)
- 15+ effets GPU
- 5+ transitions
- Effect stack UI

### v0.4.0 - Color Grading (Post Phase 3)
- LUT 3D support
- ACES workflow
- Video scopes

### v0.5.0 - Animation (Post Phase 8)
- Keyframe system
- Graph editor
- Automation lanes

### v0.6.0 - Extensibility (Post Phase 5)
- Plugin marketplace
- WASM/Python plugins
- Community plugins

### v0.7.0 - AI Features (Post Phase 4)
- LORA creator
- Auto-captioning
- Training pipeline

### v0.8.0 - Collaboration (Post Phase 7)
- Real-time sync
- Multi-user editing
- Conflict resolution

### v1.0.0 - Complete (Post Phase 6)
- Multi-window workspace
- All features integrated
- Production ready

---

## 🛠️ Infrastructure Nécessaire

### CI/CD
```yaml
# .github/workflows/ci.yml
- Rust build (stable)
- Clippy linting
- Tests unitaires
- Tests intégration
- Benchmarks performance
```

### Backend Services
- **Marketplace Backend**: Axum + PostgreSQL + S3
- **Collaboration Server**: WebSocket + Redis
- **CDN**: Cloudflare pour assets

### Documentation
- **API Docs**: cargo doc
- **User Manual**: mdBook
- **Plugin SDK**: tutorials + examples

---

## 📈 Estimation Totale

- **Phase 1**: 6-8 semaines ✅ EN COURS
- **Phase 2**: 10-12 semaines
- **Phase 3**: 6-8 semaines
- **Phase 4**: 8-10 semaines
- **Phase 5**: 10-12 semaines
- **Phase 6**: 6-8 semaines
- **Phase 7**: 16-20 semaines
- **Phase 8**: 6 semaines

**Total**: 68-88 semaines (17-22 mois)

Avec parallélisation et équipe de 2-3 devs: **12-18 mois**

---

## 🚀 État Actuel

**Fichiers créés pour Phase 1:**
- ✅ `apps/desktop/src/selection.rs` (133 lignes)
- ✅ `apps/desktop/src/edit_modes.rs` (114 lignes)
- ✅ `apps/desktop/src/keyboard.rs` (409 lignes)
- ✅ `crates/timeline/src/markers.rs` (259 lignes)

**Total nouveau code**: ~915 lignes de Rust production-ready

**Prochaine étape**: Intégration dans app.rs et timeline/ui.rs
