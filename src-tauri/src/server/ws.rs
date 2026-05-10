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
use crate::state::{AppState, AppConfig, CueEntry, EffectState};
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
    }

    drop(lighting);
    state.bump_version();
}
