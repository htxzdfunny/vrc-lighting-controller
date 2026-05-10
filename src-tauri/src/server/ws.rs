use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State as AxumState,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use crate::state::{
    AppState, AppConfig, CueEntry, EffectState,
    PaletteSlot, StateSnapshot, PALETTE_SLOT_COUNT, SNAPSHOT_SLOT_COUNT,
};
use crate::state::fixture::{Color, Fixture};
use crate::state::group::Group;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WsCommand {
    GetState,
    SetFixtureColor {
        fixture_id: usize,
        r: f64,
        g: f64,
        b: f64,
    },
    SetFixturePosition {
        fixture_id: usize,
        pan: f64,
        tilt: f64,
    },
    SetFixtureDimmer {
        fixture_id: usize,
        dimmer: f64,
    },
    SetFixtureStrobe {
        fixture_id: usize,
        strobe_on: bool,
        strobe_speed: f64,
    },
    SelectFixtures {
        ids: Vec<usize>,
    },
    SetSelectedColor {
        r: f64,
        g: f64,
        b: f64,
    },
    SetSelectedPosition {
        pan: f64,
        tilt: f64,
    },
    SetSelectedDimmer {
        dimmer: f64,
    },
    SetConfig {
        config: AppConfig,
    },
    CreateGroup {
        id: String,
        name: String,
        fixture_ids: Vec<usize>,
    },
    DeleteGroup {
        id: String,
    },
    SetFixtureCount {
        count: usize,
    },
    SaveCue {
        id: String,
        name: String,
        fade_time: f64,
    },
    GoCue {
        id: String,
    },
    DeleteCue {
        id: String,
    },
    SetEffect {
        effect: EffectState,
    },
    ClearEffect,
    SetPaletteSlot {
        index: usize,
        color: Option<Color>,
    },
    ApplyPaletteSlot {
        index: usize,
        fixture_ids: Vec<usize>,
    },
    SaveSnapshot {
        index: usize,
        label: String,
    },
    RecallSnapshot {
        index: usize,
    },
    ClearSnapshot {
        index: usize,
    },
    SetFixtureSync {
        fixture_id: usize,
        sync_fader: bool,
        sync_knob: bool,
    },
    SetFixtureOn {
        fixture_id: usize,
        on: bool,
    },
    SetButtonMode {
        enabled: bool,
    },
    ApplyMasterFader {
        value: f64,
    },
    ApplyMasterKnob {
        value: f64,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WsResponse {
    State(crate::state::LightingState),
    Error { message: String },
    Updated { version: u64 },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    use std::sync::atomic::Ordering;

    // 连接建立：递增计数并通知前端
    let ws_count = state.ws_clients.fetch_add(1, Ordering::SeqCst) + 1;
    {
        let mut lighting = state.lighting.write();
        lighting.connected_clients.ws = ws_count;
    }
    state.bump_version();

    let (mut sender, mut receiver) = socket.split();

    // Send initial state
    {
        let lighting = state.lighting.read().clone();
        let msg = serde_json::to_string(&WsResponse::State(lighting)).unwrap_or_default();
        let _ = sender.send(Message::Text(msg.into())).await;
    }

    let state_clone = state.clone();
    let mut send_task = tokio::spawn(async move {
        let mut last_version = state_clone.current_version();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let current = state_clone.current_version();
            if current != last_version {
                last_version = current;
                let lighting = state_clone.lighting.read().clone();
                let msg = serde_json::to_string(&WsResponse::State(lighting)).unwrap_or_default();
                if sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<WsCommand>(&text) {
                    Ok(cmd) => process_command(&state_clone, cmd),
                    Err(e) => {
                        log::warn!("Invalid WS command: {}", e);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // 连接断开：递减计数并通知前端
    let ws_count = state.ws_clients.fetch_sub(1, Ordering::SeqCst) - 1;
    {
        let mut lighting = state.lighting.write();
        lighting.connected_clients.ws = ws_count;
    }
    state.bump_version();
}

fn process_command(state: &AppState, cmd: WsCommand) {
    let mut lighting = state.lighting.write();

    match cmd {
        WsCommand::GetState => {}
        WsCommand::SetFixtureColor {
            fixture_id,
            r,
            g,
            b,
        } => {
            if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
                f.color = Color { r, g, b };
            }
        }
        WsCommand::SetFixturePosition {
            fixture_id,
            pan,
            tilt,
        } => {
            if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
                f.pan = pan.clamp(-180.0, 180.0);
                f.tilt = tilt.clamp(-180.0, 180.0);
            }
        }
        WsCommand::SetFixtureDimmer { fixture_id, dimmer } => {
            if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
                f.dimmer = dimmer.clamp(0.0, 1.0);
            }
        }
        WsCommand::SetFixtureStrobe {
            fixture_id,
            strobe_on,
            strobe_speed,
        } => {
            if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
                f.strobe_on = strobe_on;
                f.strobe_speed = strobe_speed.clamp(0.0, 30.0);
            }
        }
        WsCommand::SelectFixtures { ids } => {
            lighting.selected_fixture_ids = ids;
        }
        WsCommand::SetSelectedColor { r, g, b } => {
            let ids = lighting.selected_fixture_ids.clone();
            for id in ids {
                if let Some(f) = lighting.fixtures.get_mut(id) {
                    f.color = Color { r, g, b };
                }
            }
        }
        WsCommand::SetSelectedPosition { pan, tilt } => {
            let ids = lighting.selected_fixture_ids.clone();
            for id in ids {
                if let Some(f) = lighting.fixtures.get_mut(id) {
                    f.pan = pan.clamp(-180.0, 180.0);
                    f.tilt = tilt.clamp(-180.0, 180.0);
                }
            }
        }
        WsCommand::SetSelectedDimmer { dimmer } => {
            let ids = lighting.selected_fixture_ids.clone();
            for id in ids {
                if let Some(f) = lighting.fixtures.get_mut(id) {
                    f.dimmer = dimmer.clamp(0.0, 1.0);
                }
            }
        }
        WsCommand::SetConfig { config } => {
            lighting.config = config;
            let new_count = lighting.config.fixture_count.clamp(1, 32);
            lighting.config.fixture_count = new_count;
            while lighting.fixtures.len() < new_count {
                let id = lighting.fixtures.len();
                lighting.fixtures.push(Fixture::new(id));
            }
        }
        WsCommand::CreateGroup { id, name, fixture_ids } => {
            lighting.groups.push(Group { id, name, fixture_ids });
        }
        WsCommand::DeleteGroup { id } => {
            lighting.groups.retain(|g| g.id != id);
        }
        WsCommand::SetFixtureCount { count } => {
            let count = count.clamp(1, 32);
            lighting.config.fixture_count = count;
            while lighting.fixtures.len() < count {
                let id = lighting.fixtures.len();
                lighting.fixtures.push(Fixture::new(id));
            }
        }
        WsCommand::SaveCue { id, name, fade_time } => {
            let fixtures = lighting.fixtures.clone();
            let cue = CueEntry { id: id.clone(), name, fixtures, fade_time };
            if let Some(existing) = lighting.cues.iter_mut().find(|c| c.id == id) {
                *existing = cue;
            } else {
                lighting.cues.push(cue);
            }
        }
        WsCommand::GoCue { id } => {
            if let Some(cue) = lighting.cues.iter().find(|c| c.id == id) {
                let target_fixtures = cue.fixtures.clone();
                let count = lighting.config.fixture_count.min(target_fixtures.len());
                for i in 0..count {
                    if i < lighting.fixtures.len() {
                        lighting.fixtures[i] = target_fixtures[i].clone();
                    }
                }
                lighting.active_cue_id = Some(id);
            }
        }
        WsCommand::DeleteCue { id } => {
            lighting.cues.retain(|c| c.id != id);
            if lighting.active_cue_id.as_deref() == Some(&id) {
                lighting.active_cue_id = None;
            }
        }
        WsCommand::SetEffect { effect } => {
            lighting.effect = effect;
        }
        WsCommand::ClearEffect => {
            lighting.effect = EffectState::default();
        }
        WsCommand::SetPaletteSlot { index, color } => {
            if index < PALETTE_SLOT_COUNT {
                ensure_palette_len(&mut lighting);
                if let Some(slot) = lighting.color_palette.get_mut(index) {
                    *slot = PaletteSlot { color };
                }
            }
        }
        WsCommand::ApplyPaletteSlot { index, fixture_ids } => {
            if index < PALETTE_SLOT_COUNT {
                ensure_palette_len(&mut lighting);
                let color = lighting
                    .color_palette
                    .get(index)
                    .and_then(|s| s.color.clone())
                    .unwrap_or_else(|| Color { r: 0.0, g: 0.0, b: 0.0 });
                let target_ids: Vec<usize> = if fixture_ids.is_empty() {
                    (0..lighting.fixtures.len()).collect()
                } else {
                    fixture_ids
                };
                for id in target_ids {
                    if let Some(f) = lighting.fixtures.get_mut(id) {
                        f.color = color.clone();
                    }
                }
            }
        }
        WsCommand::SaveSnapshot { index, label } => {
            if index < SNAPSHOT_SLOT_COUNT {
                ensure_snapshot_len(&mut lighting);
                let snapshot = StateSnapshot {
                    label,
                    fixtures: lighting.fixtures.clone(),
                };
                if let Some(slot) = lighting.state_snapshots.get_mut(index) {
                    *slot = Some(snapshot);
                }
            }
        }
        WsCommand::RecallSnapshot { index } => {
            if index < SNAPSHOT_SLOT_COUNT {
                ensure_snapshot_len(&mut lighting);
                let snapshot_opt = lighting.state_snapshots.get(index).cloned().flatten();
                if let Some(snapshot) = snapshot_opt {
                    let live_sync: Vec<(bool, bool)> = lighting
                        .fixtures
                        .iter()
                        .map(|f| (f.sync_master_fader, f.sync_master_knob))
                        .collect();
                    let count = snapshot.fixtures.len().min(lighting.fixtures.len());
                    for i in 0..count {
                        let mut copy = snapshot.fixtures[i].clone();
                        if let Some(sync) = live_sync.get(i) {
                            copy.sync_master_fader = sync.0;
                            copy.sync_master_knob = sync.1;
                        }
                        lighting.fixtures[i] = copy;
                    }
                }
            }
        }
        WsCommand::ClearSnapshot { index } => {
            if index < SNAPSHOT_SLOT_COUNT {
                ensure_snapshot_len(&mut lighting);
                if let Some(slot) = lighting.state_snapshots.get_mut(index) {
                    *slot = None;
                }
            }
        }
        WsCommand::SetFixtureSync { fixture_id, sync_fader, sync_knob } => {
            if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
                f.sync_master_fader = sync_fader;
                f.sync_master_knob = sync_knob;
            }
        }
        WsCommand::SetFixtureOn { fixture_id, on } => {
            if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
                f.is_on = on;
            }
        }
        WsCommand::SetButtonMode { enabled } => {
            lighting.button_mode = enabled;
        }
        WsCommand::ApplyMasterFader { value } => {
            let v = value.clamp(0.0, 1.0);
            let tilt = v * 360.0 - 180.0;
            for f in lighting.fixtures.iter_mut() {
                if f.sync_master_fader {
                    f.tilt = tilt;
                }
            }
        }
        WsCommand::ApplyMasterKnob { value } => {
            let v = value.clamp(0.0, 1.0);
            let pan = v * 360.0 - 180.0;
            for f in lighting.fixtures.iter_mut() {
                if f.sync_master_knob {
                    f.pan = pan;
                }
            }
        }
    }

    drop(lighting);
    state.bump_version();
}

fn ensure_palette_len(lighting: &mut crate::state::LightingState) {
    while lighting.color_palette.len() < PALETTE_SLOT_COUNT {
        lighting.color_palette.push(PaletteSlot::default());
    }
}

fn ensure_snapshot_len(lighting: &mut crate::state::LightingState) {
    while lighting.state_snapshots.len() < SNAPSHOT_SLOT_COUNT {
        lighting.state_snapshots.push(None);
    }
}
