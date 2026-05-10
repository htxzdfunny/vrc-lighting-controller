pub mod fixture;
pub mod group;
pub mod palette;
pub mod preset;

use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use fixture::Fixture;
use group::Group;
pub use palette::{PaletteSlot, StateSnapshot};

pub const MAX_FIXTURES: usize = 32;
pub const DEFAULT_FIXTURE_COUNT: usize = 10;
pub const PALETTE_SLOT_COUNT: usize = 10;
pub const SNAPSHOT_SLOT_COUNT: usize = 7;

fn default_palette() -> Vec<PaletteSlot> {
    (0..PALETTE_SLOT_COUNT).map(|_| PaletteSlot::default()).collect()
}

fn default_snapshots() -> Vec<Option<StateSnapshot>> {
    (0..SNAPSHOT_SLOT_COUNT).map(|_| None).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectState {
    pub effect_type: EffectType,
    pub speed: f64,
    pub intensity: f64,
    pub phase_offset: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    None,
    ColorCycle,
    Scan,
    Pulse,
    Wave,
    Random,
}

impl Default for EffectState {
    fn default() -> Self {
        Self {
            effect_type: EffectType::None,
            speed: 1.0,
            intensity: 1.0,
            phase_offset: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CueEntry {
    pub id: String,
    pub name: String,
    pub fixtures: Vec<Fixture>,
    pub fade_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub fixture_count: usize,
    pub web_port: u16,
    pub ndi_enabled: bool,
    pub ndi_name: String,
    pub spout_enabled: bool,
    pub spout_name: String,
    pub output_window_visible: bool,
    pub output_fps: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            fixture_count: DEFAULT_FIXTURE_COUNT,
            web_port: 9000,
            ndi_enabled: false,
            ndi_name: "VRC Lighting".into(),
            spout_enabled: false,
            spout_name: "VRC Lighting Spout2".into(),
            output_window_visible: true,
            output_fps: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStateKind {
    Disabled,
    Active,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputEndpointStatus {
    pub state: OutputStateKind,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputStatus {
    pub ndi: OutputEndpointStatus,
    pub spout: OutputEndpointStatus,
}

impl Default for OutputStatus {
    fn default() -> Self {
        Self {
            ndi: OutputEndpointStatus {
                state: OutputStateKind::Disabled,
                message: None,
            },
            spout: OutputEndpointStatus {
                state: OutputStateKind::Disabled,
                message: None,
            },
        }
    }
}

impl Default for OutputEndpointStatus {
    fn default() -> Self {
        Self {
            state: OutputStateKind::Disabled,
            message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectedClients {
    pub ws: u64,
    pub ndi: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LightingState {
    pub fixtures: Vec<Fixture>,
    pub groups: Vec<Group>,
    pub selected_fixture_ids: Vec<usize>,
    pub cues: Vec<CueEntry>,
    pub active_cue_id: Option<String>,
    pub effect: EffectState,
    pub config: AppConfig,
    pub output_status: OutputStatus,
    pub connected_clients: ConnectedClients,
    #[serde(default = "default_palette")]
    pub color_palette: Vec<PaletteSlot>,
    #[serde(default = "default_snapshots")]
    pub state_snapshots: Vec<Option<StateSnapshot>>,
    #[serde(default)]
    pub button_mode: bool,
}

impl Default for LightingState {
    fn default() -> Self {
        let fixtures = (0..DEFAULT_FIXTURE_COUNT)
            .map(|i| Fixture::new(i))
            .collect();
        Self {
            fixtures,
            groups: Vec::new(),
            selected_fixture_ids: vec![0],
            cues: Vec::new(),
            active_cue_id: None,
            effect: EffectState::default(),
            config: AppConfig::default(),
            output_status: OutputStatus::default(),
            connected_clients: ConnectedClients::default(),
            color_palette: default_palette(),
            state_snapshots: default_snapshots(),
            button_mode: false,
        }
    }
}

pub struct AppState {
    pub lighting: RwLock<LightingState>,
    pub frame_buffer: RwLock<Vec<u8>>,
    pub version: AtomicU64,
    pub ws_clients: AtomicU64,
    pub app_handle: RwLock<Option<tauri::AppHandle>>,
}

impl AppState {
    pub fn new() -> Self {
        let width = 90usize;
        let height = 720usize;
        Self {
            lighting: RwLock::new(LightingState::default()),
            frame_buffer: RwLock::new(vec![0u8; width * height * 4]),
            version: AtomicU64::new(0),
            ws_clients: AtomicU64::new(0),
            app_handle: RwLock::new(None),
        }
    }

    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        *self.app_handle.write() = Some(handle);
    }

    pub fn bump_version(&self) -> u64 {
        let v = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(handle) = self.app_handle.read().as_ref() {
            let _ = handle.emit("state-changed", v);
        }
        v
    }

    pub fn current_version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }
}
