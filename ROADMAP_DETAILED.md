# 🚀 Gausian Native Editor - Roadmap Détaillée 2025-2026

## 📊 État Actuel de l'Architecture

### ✅ Fonctionnalités Existantes Solides
- Timeline UI fonctionnelle avec drag & drop, trim, snap
- Rendu GPU WGPU avec pipelines YUV→RGB optimisés
- Support color spaces: BT.709, BT.2020, DCI-P3, sRGB
- Décodage matériel: VideoToolbox (macOS), GStreamer (multi-plateforme)
- Base de données SQLite avec migrations
- Export professionnel: FCPXML, EDL, FCP7, JSON
- Intégration ComfyUI pour workflows IA
- Générateurs: Solid Color, Text
- Audio engine avec waveform visualization

### ⚠️ Limitations Identifiées
- Effets/transitions en stub (architecture définie, rendu non implémenté)
- Système de plugins incomplet (WASM/Python définis, exécution manquante)
- Pas de LUT 3D / Color Grading avancé
- UI mono-fenêtre (pas de multi-window workspace)
- Édition collaborative: aucune infrastructure
- Marketplace plugins: définition seulement

---

## 🎯 Roadmap Priorisée par Phase

---

## 📅 PHASE 1 : Timeline Polish & UX Improvements (Q1 2025)
**Priorité:** 🔴 CRITIQUE | **Durée estimée:** 6-8 semaines

### 1.1 Améliorations Timeline Core
**Fichiers concernés:** `apps/desktop/src/timeline/ui.rs`, `crates/timeline/src/`

#### Objectifs:
- [ ] **Magnetisme intelligent multi-couches**
  - Snap vers playhead, bords de clips, marqueurs, secondes
  - Tolérance configurable par utilisateur
  - Indicateur visuel du snap actif

- [ ] **Sélection multiple de clips**
  - Rectangle de sélection (drag sans clic sur clip)
  - Shift-click pour ajouter à la sélection
  - Cmd/Ctrl-A pour tout sélectionner
  - Opérations groupées: déplacement, suppression, copie

- [ ] **Ripple & Roll Edit modes**
  ```rust
  pub enum EditMode {
      Normal,       // Déplacement standard
      Ripple,       // Décale tous les clips suivants
      Roll,         // Ajuste point de coupe entre 2 clips
      Slide,        // Déplace contenu sans changer position
      Slip,         // Change in/out sans déplacer
  }
  ```

- [ ] **Timeline markers & regions**
  ```rust
  pub struct Marker {
      frame: Frame,
      label: String,
      color: Color32,
      marker_type: MarkerType,  // In, Out, Chapter, Comment
  }
  ```

- [ ] **Raccourcis clavier professionnels**
  - J/K/L: rewind/pause/forward playback
  - I/O: set in/out points
  - E: append to timeline
  - Q/W: trim start/end to playhead
  - [ / ]: jump to previous/next edit
  - Cmd+Z/Shift+Cmd+Z: undo/redo (déjà implémenté dans CommandHistory)

#### Améliorations Visuelles:
- [ ] **Affichage timecode**
  - HH:MM:SS:FF en haut de la timeline
  - Drop-frame / non-drop frame support

- [ ] **Zoom adaptatif**
  - Fit to window (Shift+Z)
  - Zoom sur sélection
  - Mini-map de la timeline complète

- [ ] **Thèmes de couleur**
  - Track colors personnalisables
  - Mode sombre/clair pour UI
  - Highlight de clips sélectionnés plus visible

#### Performances:
- [ ] **Optimisation rendering timeline**
  - Culling des clips hors écran
  - LOD pour waveforms (plus de détails au zoom)
  - Cache des positions calculées

- [ ] **Préchargement intelligent**
  - Decode frames autour du playhead (+/- 5 secondes)
  - Invalidation cache sur changement de séquence

---

## 📅 PHASE 2 : Rich Effects & Transitions (Q2 2025)
**Priorité:** 🟠 HAUTE | **Durée estimée:** 10-12 semaines

### 2.1 Système d'Effets GPU

**Architecture proposée:**

```rust
// crates/renderer/src/effects/mod.rs
pub trait Effect: Send + Sync {
    fn name(&self) -> &str;
    fn parameters(&self) -> &[EffectParameter];
    fn apply(&self,
             input: &wgpu::Texture,
             output: &wgpu::Texture,
             params: &HashMap<String, f32>,
             device: &wgpu::Device,
             queue: &wgpu::Queue) -> Result<()>;
}

pub struct EffectParameter {
    pub name: String,
    pub default: f32,
    pub min: f32,
    pub max: f32,
    pub param_type: ParamType,  // Slider, Color, Bool, Angle
}
```

#### Effets Vidéo à Implémenter:

##### Corrections de Base:
- [ ] **Brightness/Contrast**
  - Shader: `effects/brightness_contrast.wgsl`
  - Uniforms: brightness (-1.0 to 1.0), contrast (0.0 to 2.0)

- [ ] **Saturation/Hue**
  - Conversion RGB → HSL → RGB sur GPU
  - Uniforms: saturation (0.0 to 2.0), hue rotation (-180° to 180°)

- [ ] **Exposure/Gamma**
  - Shader: pow(color, gamma) * exposure
  - Support HDR (P010 textures)

##### Corrections Avancées:
- [ ] **Curves (RGB/Luma)**
  - Texture 1D lookup table (256 entries)
  - Séparation canaux R/G/B/Master
  - UI: courbe Bézier interactive (egui_plot)

- [ ] **Color Wheels (Shadows/Midtones/Highlights)**
  ```rust
  pub struct ColorWheels {
      shadows: ColorWheel,    // Hue/Sat pour ombres
      midtones: ColorWheel,
      highlights: ColorWheel,
      luminance_ranges: [f32; 3],  // Seuils pour S/M/H
  }
  ```
  - Shader: range masks par luminance

- [ ] **Vignette**
  - Shader: distance radiale du centre
  - Paramètres: intensity, softness, center offset

##### Effets Stylisés:
- [ ] **Gaussian Blur**
  - Two-pass separable blur (horizontal + vertical)
  - Paramètre: radius (0-100px)

- [ ] **Sharpen/Unsharp Mask**
  - Convolution kernel 3x3
  - Paramètres: strength, radius

- [ ] **Film Grain**
  - Noise texture procedural (shader)
  - Paramètres: intensity, size

- [ ] **Chromatic Aberration**
  - Offset R/G/B channels radialement
  - Paramètres: strength, center

##### Effets Géométriques:
- [ ] **Transform**
  - Position (X/Y)
  - Scale (uniform ou X/Y séparés)
  - Rotation (degrés)
  - Anchor point
  - Matrice 4x4 calculée sur CPU

- [ ] **Crop/Padding**
  - Crop rectangle avec feathering
  - Padding avec couleur de remplissage

- [ ] **Corner Pin / Perspective**
  - 4 coins déplaçables
  - Matrice de projection calculée

##### Compositing:
- [ ] **Keying (Chroma Key)**
  ```rust
  pub struct ChromaKey {
      key_color: [f32; 3],      // RGB couleur clé
      tolerance: f32,           // Seuil de distance couleur
      edge_feather: f32,        // Adoucissement bords
      spill_suppression: f32,   // Réduction spill vert/bleu
  }
  ```
  - Shader: distance euclidienne en YUV space

- [ ] **Blend Modes**
  - Déjà défini dans `renderer/src/lib.rs`, implémenter tous:
    - Normal, Multiply, Screen, Overlay
    - SoftLight, HardLight, ColorDodge, ColorBurn
    - Darken, Lighten, Difference, Exclusion
  - Shader blend.wgsl à compléter

### 2.2 Système de Transitions

**Architecture:**

```rust
// crates/timeline/src/transitions.rs
pub trait Transition {
    fn render(&self,
              from_frame: &wgpu::Texture,
              to_frame: &wgpu::Texture,
              progress: f32,  // 0.0 to 1.0
              output: &wgpu::Texture,
              device: &wgpu::Device,
              queue: &wgpu::Queue) -> Result<()>;
}
```

#### Transitions à Implémenter:

- [ ] **Dissolve (Cross-fade)**
  - Simple mix(a, b, progress)

- [ ] **Wipe (Directional)**
  - 8 directions: Left, Right, Up, Down, + diagonales
  - Paramètres: angle, feathering

- [ ] **Slide**
  - From/To frames translate
  - Directions: push, peel, reveal

- [ ] **Zoom/Scale**
  - Scale up outgoing, scale down incoming

- [ ] **Spin/Rotate**
  - Rotation 3D avec perspective

- [ ] **Morphing Transitions**
  - Mesh warp entre 2 frames (avancé)

#### UI Timeline pour Transitions:
- [ ] Glisser transition entre 2 clips
- [ ] Ajustement durée par drag des poignées
- [ ] Preview temps réel dans viewer

### 2.3 Stack d'Effets

**Implémentation dans Timeline:**

```rust
pub struct ClipNode {
    // ... existing fields
    pub effects_stack: Vec<EffectInstance>,
}

pub struct EffectInstance {
    pub effect_id: String,              // "blur", "color_correction", etc.
    pub enabled: bool,
    pub parameters: HashMap<String, f32>,
    pub keyframes: HashMap<String, Vec<Keyframe>>,  // Animation
}
```

**UI Inspector Panel:**
- [ ] Liste effets appliqués (drag to reorder)
- [ ] Activation/désactivation par effet
- [ ] Paramètres avec sliders/color pickers
- [ ] Keyframing pour animation (voir Phase 4)

---

## 📅 PHASE 3 : Advanced Color Management & LUTs (Q2-Q3 2025)
**Priorité:** 🟡 MOYENNE | **Durée estimée:** 6-8 semaines

### 3.1 Système LUT 3D

**Formats supportés:**
- [ ] **.cube** (Adobe/Resolve standard)
- [ ] **.3dl** (Autodesk/Lustre)
- [ ] **.csp** (Rising Sun Research ColorSpace)

**Parser LUT:**

```rust
// crates/renderer/src/lut/mod.rs
pub struct Lut3D {
    pub size: u32,                    // 17, 33, 65 typical
    pub data: Vec<[f32; 3]>,          // RGB triplets
    pub input_range: (f32, f32),      // (0.0, 1.0) ou (0, 1023)
    pub name: String,
}

impl Lut3D {
    pub fn from_cube_file(path: &Path) -> Result<Self>;
    pub fn from_3dl_file(path: &Path) -> Result<Self>;

    pub fn to_texture(&self, device: &wgpu::Device) -> wgpu::Texture {
        // Créer texture 3D GPU
        // Size^3 texels RGB
    }
}
```

**Shader LUT Application:**

```wgsl
// crates/renderer/src/shaders/lut_apply.wgsl
@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var lut_tex: texture_3d<f32>;
@group(0) @binding(2) var lut_sampler: sampler;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let input_color = textureSample(input_tex, sampler, uv).rgb;

    // Trilinear interpolation dans LUT 3D
    let lut_coord = input_color * (lut_size - 1.0) / lut_size + 0.5 / lut_size;
    let graded_color = textureSample(lut_tex, lut_sampler, lut_coord);

    return vec4<f32>(graded_color, 1.0);
}
```

### 3.2 Color Space Management

**ACES Workflow Support:**

```rust
pub enum ColorSpaceTransform {
    Rec709ToLinear,
    LinearToRec709,
    Rec2020ToACEScg,      // ACES color grading space
    ACEScgToRec2020,
    sRGBToLinear,
    LinearToACES2065,     // ACES archival
}
```

**Implémentation:**
- [ ] Matrices de conversion en shaders
- [ ] ODT (Output Device Transform) pour différents displays
- [ ] IDT (Input Device Transform) pour caméras connues
- [ ] RRT (Reference Rendering Transform)

**UI Settings:**
- [ ] Working color space selector (project-wide)
- [ ] Per-clip input color space override
- [ ] Output transform pour export

### 3.3 Scopes Vidéo

**Outils d'analyse couleur:**

```rust
pub enum VideoScope {
    Waveform,         // Luminance par position X
    Vectorscope,      // Chrominance U/V
    Histogram,        // Distribution RGB
    Parade,           // R/G/B waveforms séparés
}
```

**Implémentation GPU:**
- [ ] Compute shaders pour analyse frame
- [ ] Readback vers CPU pour affichage
- [ ] UI Panel séparé avec egui_plot

---

## 📅 PHASE 4 : Automatic LORA Creator (Q3 2025)
**Priorité:** 🟣 SPÉCIALISÉE | **Durée estimée:** 8-10 semaines

### 4.1 Architecture IA/ML

**Dépendances à ajouter:**
```toml
# crates/ai-pipeline/Cargo.toml
[dependencies]
candle-core = "0.4"           # Framework ML en Rust
candle-nn = "0.4"
candle-transformers = "0.4"
tokenizers = "0.15"
image = "0.25"
reqwest = { version = "0.11", features = ["json"] }  # API calls
serde_json = "1"
```

### 4.2 Pipeline LORA Training

**Workflow proposé:**

```rust
// crates/ai-pipeline/src/lora_creator.rs
pub struct LoraCreator {
    pub base_model: String,           // "stabilityai/sdxl-1.0"
    pub training_images: Vec<PathBuf>,
    pub captions: Vec<String>,
    pub config: LoraConfig,
}

pub struct LoraConfig {
    pub rank: u32,                    // LoRA rank (4, 8, 16, 32)
    pub alpha: f32,                   // LoRA alpha (scaling)
    pub learning_rate: f32,
    pub batch_size: u32,
    pub epochs: u32,
    pub resolution: (u32, u32),       // (512, 512) ou (1024, 1024)
    pub trigger_word: Option<String>, // Mot-clé activant LoRA
}

impl LoraCreator {
    pub async fn prepare_dataset(&self) -> Result<PathBuf> {
        // Extraction frames de timeline
        // Resize/crop uniformisation
        // Génération captions (BLIP2/LLaVA)
    }

    pub async fn train_lora(&self, dataset_path: &Path) -> Result<LoraWeights> {
        // Option 1: Local avec Candle (limité)
        // Option 2: API cloud (Replicate, Modal)
        // Option 3: Local ComfyUI workflow
    }

    pub async fn validate_lora(&self, weights: &LoraWeights) -> Vec<GeneratedImage> {
        // Génération images test avec prompts variés
    }
}
```

### 4.3 Intégration Timeline → LORA

**UI Workflow:**

1. **Sélection de clips/frames**
   - [ ] Bouton "Extract Training Set" dans Assets panel
   - [ ] Sélectionner range de frames ou clips entiers
   - [ ] Auto-sampling intelligent (1 frame/seconde, déduplication)

2. **Configuration Training**
   - [ ] UI panel avec formulaire LoraConfig
   - [ ] Preview du dataset (grid d'images)
   - [ ] Option caption manuelle ou auto (BLIP2)

3. **Exécution Training**
   - [ ] Job queue intégration (table `jobs` DB)
   - [ ] Progress bar avec loss/epoch
   - [ ] Backend sélectionnable:
     - ComfyUI local (workflow .json pré-configuré)
     - Replicate API (cloud)
     - Modal Functions (cloud)
     - Local Candle (expérimental)

4. **Résultat LoRA**
   - [ ] Sauvegarde dans projet (`/loras/my_lora.safetensors`)
   - [ ] Ajout automatique à ComfyUI LoRA folder
   - [ ] Preview gallery avec prompts test
   - [ ] Metadata: base model, rank, trigger word

### 4.4 Intégration ComfyUI Workflow

**Auto-configuration ComfyUI:**

```json
// Gausian_native_editor-main/formats/lora_training_workflow.json
{
  "nodes": {
    "load_checkpoint": { "class_type": "CheckpointLoaderSimple", ... },
    "load_lora_trainer": { "class_type": "LoraTrainer", ... },
    "dataset_loader": { "class_type": "DatasetLoader", "inputs": { "path": "$DATASET_PATH" } },
    "train_node": { "class_type": "TrainLoop", "inputs": { "epochs": "$EPOCHS", ... } },
    "save_lora": { "class_type": "SaveLora", "outputs": { "path": "$OUTPUT_PATH" } }
  }
}
```

**Intégration:**
- [ ] Template workflow JSON
- [ ] Substitution variables dynamiques
- [ ] Upload dataset vers ComfyUI input folder
- [ ] WebSocket monitoring du training
- [ ] Download LoRA résultant

---

## 📅 PHASE 5 : Plugin Marketplace (Q3-Q4 2025)
**Priorité:** 🔵 MOYENNE-BASSE | **Durée estimée:** 10-12 semaines

### 5.1 Finalisation Plugin System

**Complément du système existant:**

#### WASM Plugin Execution:

```rust
// crates/plugin-host/src/wasm_runtime.rs
impl PluginHost {
    pub fn execute_wasm_plugin(
        &self,
        plugin_id: &str,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let plugin = self.plugins.read().get(plugin_id)?;

        match &plugin.runtime_handle {
            PluginRuntimeHandle::Wasm(module) => {
                // Créer instance avec WASI
                let mut store = wasmtime::Store::new(&self.wasm_engine, ());
                let instance = wasmtime::Instance::new(&mut store, module, &[])?;

                // Appeler fonction "process_frame"
                let process = instance.get_typed_func::<(u32, u32), i32>(&mut store, "process_frame")?;

                // Passer contexte via linear memory
                // Récupérer résultat
            }
            _ => Err(anyhow!("Not a WASM plugin"))
        }
    }
}
```

**ABI Plugin WASM:**

```rust
// Plugin SDK (separate crate: gausian-plugin-sdk)
#[no_mangle]
pub extern "C" fn process_frame(width: u32, height: u32) -> i32 {
    // Plugin code here
    // Access to linear memory for frame data
    0  // Success
}

// Export plugin manifest
#[no_mangle]
pub extern "C" fn get_manifest() -> *const u8 {
    // Return JSON manifest
}
```

#### Python Plugin Bridge:

```rust
// crates/plugin-host/src/python_bridge.rs (compléter stub)
use pyo3::prelude::*;

pub struct PythonBridge {
    interpreter: Py<PyAny>,
}

impl PythonBridge {
    pub fn execute_plugin(&self, script_path: &Path, context: &PluginContext) -> Result<PluginResult> {
        Python::with_gil(|py| {
            let module = PyModule::from_code(
                py,
                &std::fs::read_to_string(script_path)?,
                "plugin.py",
                "plugin"
            )?;

            let process_fn = module.getattr("process_frame")?;
            let result = process_fn.call1((context.to_python(py)?,))?;

            PluginResult::from_python(result)
        })
    }
}
```

### 5.2 Plugin Marketplace Backend

**Architecture:**

```rust
// crates/plugin-host/src/marketplace.rs (complément)
pub struct MarketplaceClient {
    base_url: String,                 // "https://plugins.gausian.xyz/api"
    api_key: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub category: PluginCategory,
    pub downloads: u64,
    pub rating: f32,
    pub price: Option<f32>,           // USD, None = free
    pub download_url: String,
    pub signature: String,            // Ed25519 signature
    pub screenshots: Vec<String>,
    pub verified: bool,               // Par l'équipe Gausian
}

pub enum PluginCategory {
    Effects,
    Transitions,
    Generators,
    ColorGrading,
    AI,
    Audio,
    Import,
    Export,
}
```

**API Endpoints:**

```rust
impl MarketplaceClient {
    pub async fn search(&self, query: &str, category: Option<PluginCategory>) -> Result<Vec<MarketplacePlugin>>;
    pub async fn get_plugin(&self, id: &str) -> Result<MarketplacePlugin>;
    pub async fn download_plugin(&self, id: &str, dest: &Path) -> Result<PathBuf>;
    pub async fn verify_signature(&self, plugin_path: &Path, signature: &str) -> Result<bool>;
    pub async fn submit_plugin(&self, manifest: &PluginManifest, archive: &Path) -> Result<String>;
}
```

### 5.3 Plugin Marketplace UI

**Panels dans App:**

```rust
pub struct MarketplacePanel {
    pub search_query: String,
    pub selected_category: Option<PluginCategory>,
    pub plugins: Vec<MarketplacePlugin>,
    pub installed_plugins: HashSet<String>,
    pub downloading: HashMap<String, f32>,  // plugin_id → progress
}
```

**UI Flow:**
- [ ] Browse/search plugins
- [ ] Filters: catégorie, prix, rating
- [ ] Plugin details popup (description, screenshots, reviews)
- [ ] Install button → download + verify signature + extract
- [ ] Update notifications pour plugins installés
- [ ] Manage installed plugins (enable/disable/uninstall)

### 5.4 Sécurité & Sandboxing

**Sandbox WASM:**
- [ ] Limites mémoire (configurables dans SecurityPolicy)
- [ ] Limites CPU time
- [ ] Pas d'accès réseau par défaut (capability required)
- [ ] Accès filesystem restreint à temp dir

**Signature Vérification:**
- [ ] Ed25519 keypair pour signing
- [ ] Vérification obligatoire sauf override dans settings
- [ ] Liste de révocation (plugins malveillants)

**Code Review:**
- [ ] Plugins verified: review manuel par équipe
- [ ] Community plugins: avertissement utilisateur

---

## 📅 PHASE 6 : Multi-Window Workspace (Q4 2025)
**Priorité:** 🟢 BASSE | **Durée estimée:** 6-8 semaines

### 6.1 Architecture Multi-Fenêtre

**Problème:** egui/eframe actuel = single window

**Solution:** Migration vers winit + egui multi-viewport

```rust
// apps/desktop/src/workspace/mod.rs
pub struct Workspace {
    pub windows: HashMap<WindowId, WorkspaceWindow>,
    pub main_window: WindowId,
}

pub enum WorkspaceWindow {
    Timeline {
        seq: Sequence,
        playhead: i64,
        zoom: f32,
    },
    Viewer {
        preview: PreviewState,
        scopes: Vec<VideoScope>,
    },
    Assets {
        browser: AssetBrowser,
    },
    Inspector {
        selected_node: Option<NodeId>,
    },
    Effects {
        effect_stack: Vec<EffectInstance>,
    },
    ColorGrading {
        lut: Option<Lut3D>,
        wheels: ColorWheels,
    },
}
```

**Implémentation:**

```rust
use winit::window::{Window, WindowBuilder};
use egui_winit::State as EguiWinitState;

pub struct MultiWindowApp {
    pub windows: HashMap<winit::window::WindowId, WindowState>,
}

pub struct WindowState {
    pub window: Arc<Window>,
    pub egui_state: EguiWinitState,
    pub renderer: wgpu::Surface,
    pub content: WorkspaceWindow,
}

impl MultiWindowApp {
    pub fn spawn_window(&mut self, content: WorkspaceWindow, event_loop: &winit::event_loop::EventLoopWindowTarget<()>) {
        let window = WindowBuilder::new()
            .with_title(content.title())
            .with_inner_size(content.default_size())
            .build(event_loop)?;

        // Setup egui + wgpu pour cette fenêtre
        // ...
    }
}
```

### 6.2 Layouts Prédéfinis

**Workspaces:**
- [ ] **Editing**: Timeline + Viewer + Assets (layout par défaut)
- [ ] **Color Grading**: Large viewer + Scopes + Color wheels
- [ ] **Effects**: Timeline + Effect controls + Keyframe editor
- [ ] **Audio**: Waveform timeline + Mixer + VU meters
- [ ] **Custom**: User-defined layouts sauvegardés

**Persistence:**
```rust
// Sauvegarder dans project DB
pub struct WorkspaceLayout {
    pub name: String,
    pub windows: Vec<WindowConfig>,
}

pub struct WindowConfig {
    pub content_type: String,     // "timeline", "viewer", etc.
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub monitor: Option<u32>,     // Multi-monitor support
}
```

---

## 📅 PHASE 7 : Collaborative Editing (Q1-Q2 2026)
**Priorité:** 🔴 CRITIQUE (pour équipes) | **Durée estimée:** 16-20 semaines

### 7.1 Architecture Collaborative

**Approche:** Operational Transform (OT) ou CRDT

**Choix recommandé:** **CRDT (Conflict-free Replicated Data Type)**

```rust
// crates/collaboration/src/lib.rs
use automerge::Automerge;  // Bibliothèque CRDT en Rust

pub struct CollaborativeProject {
    pub doc: Automerge,               // Document CRDT
    pub local_actor: ActorId,         // UUID de cet utilisateur
    pub peers: HashMap<ActorId, PeerState>,
}

pub struct PeerState {
    pub name: String,
    pub color: Color32,               // Couleur curseur/sélection
    pub playhead: i64,                // Position de lecture
    pub selection: Vec<NodeId>,       // Clips sélectionnés
    pub last_seen: Instant,
}
```

**Synchronisation:**

```rust
pub trait SyncBackend {
    fn send_change(&self, change: Change) -> Result<()>;
    fn receive_changes(&self) -> Result<Vec<Change>>;
}

// Implémentations possibles:
pub struct WebSocketSync { url: String }
pub struct WebRTCSync { peer_connections: Vec<PeerConnection> }
pub struct CloudStorageSync { bucket: S3Bucket }  // Conflict resolution via timestamps
```

### 7.2 Modifications Base de Données

**Nouvelles tables:**

```sql
-- V0010__collaboration.sql

CREATE TABLE users (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  display_name TEXT,
  avatar_url TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE project_collaborators (
  project_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('owner', 'editor', 'viewer')),
  joined_at INTEGER NOT NULL,
  PRIMARY KEY (project_id, user_id)
);

CREATE TABLE change_log (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  change_type TEXT NOT NULL,  -- 'insert_clip', 'move_clip', 'delete_clip', etc.
  change_data TEXT NOT NULL,  -- JSON payload
  timestamp INTEGER NOT NULL,
  automerge_hash TEXT         -- CRDT hash for deduplication
);

CREATE TABLE locks (
  resource_id TEXT PRIMARY KEY,  -- NodeId or TrackId
  user_id TEXT NOT NULL,
  acquired_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);
```

### 7.3 UI Collaborative Features

**Indicateurs visuels:**
- [ ] **Curseurs des autres utilisateurs** sur timeline
  - Nom + couleur
  - Position playhead en temps réel

- [ ] **Sélection partagée**
  - Outline coloré sur clips sélectionnés par peers
  - Tooltip "Bob is editing this clip"

- [ ] **Locks visuels**
  - Icône cadenas sur clips lockés par autrui
  - Tentative d'édition → notification "Locked by Alice"

**Collaboration panel:**
```rust
pub struct CollaborationPanel {
    pub online_users: Vec<PeerState>,
    pub chat_messages: Vec<ChatMessage>,
    pub activity_feed: Vec<ActivityEvent>,  // "Bob added clip at 00:30"
}
```

### 7.4 Conflict Resolution

**Stratégies:**

1. **Last-Write-Wins (LWW)**
   - Timestamp-based
   - Simple mais peut perdre des édits

2. **CRDT Merge**
   - Automerge automatic resolution
   - Préserve toutes les intentions
   - Recommandé pour timeline

3. **Manual Resolution**
   - UI pour choisir version en cas de conflit détecté
   - Diff view (before/after)

**Implémentation:**

```rust
impl CollaborativeProject {
    pub fn merge_remote_changes(&mut self, changes: Vec<Change>) -> Result<Vec<Conflict>> {
        let conflicts = Vec::new();

        for change in changes {
            match self.doc.apply_change(change) {
                Ok(_) => {},
                Err(AutomergeError::Conflict { .. }) => {
                    conflicts.push(Conflict { ... });
                }
            }
        }

        Ok(conflicts)
    }
}
```

### 7.5 Backend Serveur (Optionnel)

**Stack suggéré:**
- Rust + Axum (async web framework)
- WebSocket pour real-time sync
- PostgreSQL pour persistence
- Redis pour cache/session

**Endpoints:**

```rust
// relay/src/main.rs (déjà existant, à étendre)
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/projects/:id/changes", post(push_changes))
        .route("/projects/:id/changes", get(pull_changes))
        .route("/projects/:id/collaborators", get(list_collaborators));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

---

## 📅 PHASE 8 : Animation & Keyframing (Bonus - Q2 2026)
**Priorité:** 🟡 MOYENNE | **Durée estimée:** 6 semaines

### 8.1 Keyframe System

**Extension du système d'automation existant:**

```rust
// crates/timeline/src/graph.rs - déjà partiellement défini
pub struct AutomationLane {
    pub id: LaneId,
    pub target: AutomationTarget,
    pub interpolation: AutomationInterpolation,
    pub keyframes: Vec<AutomationKeyframe>,
}

pub enum AutomationTarget {
    EffectParameter { node_id: NodeId, effect_idx: usize, param_name: String },
    Transform { node_id: NodeId, property: TransformProperty },
    Opacity { node_id: NodeId },
    AudioVolume { node_id: NodeId },
    AudioPan { node_id: NodeId },
}

pub enum TransformProperty {
    PositionX, PositionY,
    ScaleX, ScaleY,
    Rotation,
    AnchorX, AnchorY,
}

pub enum AutomationInterpolation {
    Step,           // Sauts instantanés
    Linear,         // Interpolation linéaire
    Bezier { control_points: [(f32, f32); 2] },  // Courbe Bézier
    Hold,           // Garde valeur jusqu'au prochain keyframe
}

pub struct AutomationKeyframe {
    pub frame: Frame,
    pub value: f64,
    pub easing: KeyframeEasing,
}

pub enum KeyframeEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Custom { curve: BezierCurve },
}
```

### 8.2 Keyframe Editor UI

**Timeline Integration:**
- [ ] Bouton "Show Keyframes" par effet/transform
- [ ] Automation lanes sous clips (expandable)
- [ ] Ajout keyframe: Cmd+K au playhead
- [ ] Déplacement keyframes par drag
- [ ] Valeurs éditables par double-click
- [ ] Copier/coller keyframes

**Graph Editor:**
- [ ] Vue graphique des courbes (egui_plot)
- [ ] Handles Bézier pour ajuster courbes
- [ ] Zoom/pan dans graph
- [ ] Snap to grid (frames)

---

## 🎯 Récapitulatif des Priorités

### 🔴 PRIORITÉ MAXIMALE (Commencer immédiatement)
1. **Phase 1: Timeline Polish & UX** - Fondation pour expérience utilisateur professionnelle
2. **Phase 7: Collaborative Editing** - Différenciation marché (si cible équipes)

### 🟠 PRIORITÉ HAUTE (Q2 2025)
3. **Phase 2: Rich Effects & Transitions** - Fonctionnalités attendues pour éditeur vidéo
4. **Phase 3: Color Management & LUTs** - Workflow professionnel

### 🟡 PRIORITÉ MOYENNE (Q3-Q4 2025)
5. **Phase 5: Plugin Marketplace** - Écosystème extensible
6. **Phase 8: Animation & Keyframing** - Animation avancée

### 🟢 PRIORITÉ BASSE (Peut attendre)
7. **Phase 6: Multi-Window Workspace** - Nice-to-have mais pas bloquant

### 🟣 PRIORITÉ SPÉCIALISÉE (Niche market)
8. **Phase 4: Automatic LORA Creator** - Fonctionnalité unique pour marché IA

---

## 📊 Plan d'Implémentation Suggéré

### Sprint 1-4 (Semaines 1-8): Timeline UX
- Sélection multiple
- Ripple/Roll edits
- Marqueurs & régions
- Raccourcis clavier
- Optimisations performance

### Sprint 5-10 (Semaines 9-20): Effets Core
- Brightness/Contrast/Saturation
- Curves & Color Wheels
- Blur, Sharpen, Vignette
- Transform (Position/Scale/Rotation)
- Chroma Key basique

### Sprint 11-13 (Semaines 21-26): Transitions
- Dissolve, Wipe, Slide
- UI timeline pour transitions
- Preview temps réel

### Sprint 14-16 (Semaines 27-32): LUT & Color
- Parser .cube/.3dl
- Shader LUT application
- Scopes vidéo (Waveform, Vectorscope)

### Sprint 17-20 (Semaines 33-40): Plugin System
- Finaliser WASM/Python execution
- Plugin SDK documentation
- Marketplace UI
- 3-5 plugins example

### Sprint 21-24 (Semaines 41-48): LORA Creator
- Intégration dataset extraction
- ComfyUI workflow automation
- UI training configuration

### Sprint 25-32 (Semaines 49-64): Collaborative Editing
- CRDT integration
- Backend sync serveur
- UI collaborative features
- Conflict resolution

---

## 🛠️ Outils & Dépendances Additionnelles

### Nouvelles crates à ajouter:
```toml
# Keyframing & Animation
cubic-spline = "0.3"
lyon = "1.0"  # Path rendering pour courbes Bézier

# LUT parsing
nom = "7"     # Parser combinator pour .cube/.3dl

# ML/AI
candle-core = "0.4"
tokenizers = "0.15"

# Collaboration
automerge = "0.5"
tungstenite = "0.21"  # Déjà présent

# Multi-window
winit = "0.30"  # Déjà présent, utiliser multi-viewport feature

# Plugin Python
pyo3 = { version = "0.20", features = ["auto-initialize"] }

# Scopes visualization
egui_plot = "0.29"
```

### Infrastructure:
- CI/CD pour marketplace (GitHub Actions)
- Serveur sync collaborative (VPS ou cloud)
- CDN pour assets plugins
- Documentation site (mdBook)

---

## 📈 Métriques de Succès

### Phase 1 (Timeline):
- ✅ Raccourcis J/K/L fonctionnels
- ✅ Sélection multiple + opérations groupées
- ✅ Ripple edit ne laisse aucun gap
- ✅ Performance: 60 FPS avec 100+ clips

### Phase 2 (Effets):
- ✅ 15+ effets fonctionnels
- ✅ Stack d'effets avec reorder
- ✅ Preview temps réel à 30 FPS minimum

### Phase 3 (Color):
- ✅ Import .cube LUT en <100ms
- ✅ Application LUT temps réel
- ✅ Scopes rafraîchis à 10 FPS

### Phase 5 (Plugins):
- ✅ 10+ plugins communauté dans marketplace
- ✅ Temps d'installation plugin <30s
- ✅ Sandbox: aucun crash app si plugin fail

### Phase 7 (Collaboration):
- ✅ Latence sync <500ms
- ✅ 0 perte de données en cas de conflit
- ✅ Support 10+ utilisateurs simultanés

---

## 🚀 Prochaines Étapes Immédiates

Recommandation pour **démarrer dès maintenant**:

1. **Créer branches Git** pour chaque phase
2. **Phase 1 Sprint 1**: Implémenter sélection multiple (semaine 1)
3. **Documenter API** existante (timeline, renderer) pour onboarding contributeurs
4. **Setup CI/CD** pour tests automatisés
5. **Créer issues GitHub** pour chaque tâche de cette roadmap

---

**Cette roadmap transformera Gausian Native Editor d'un MVP solide en éditeur vidéo professionnel de classe mondiale avec des fonctionnalités uniques (LORA creator, collaboration temps réel) que les concurrents n'ont pas.**

Souhaitez-vous que je commence l'implémentation d'une phase spécifique ?
