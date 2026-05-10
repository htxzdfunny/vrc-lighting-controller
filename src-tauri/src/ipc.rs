use std::sync::Arc;
use std::path::PathBuf;
use tauri::{State, AppHandle, Manager};
use crate::state::{AppState, LightingState, EffectState, CueEntry, AppConfig};
use crate::state::fixture::{Fixture, Color};
use crate::state::group::Group;

fn resolve_runtime_path(path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        candidate
    } else {
        crate::persistence::get_runtime_dir().join(candidate)
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_state(state: State<'_, Arc<AppState>>) -> LightingState {
    state.lighting.read().clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_fixture(state: State<'_, Arc<AppState>>, fixture: Fixture) {
    let mut lighting = state.lighting.write();
    if let Some(f) = lighting.fixtures.get_mut(fixture.id) {
        *f = fixture;
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_fixture_color(state: State<'_, Arc<AppState>>, fixture_id: usize, r: f64, g: f64, b: f64) {
    let mut lighting = state.lighting.write();
    if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
        f.color = Color { r, g, b };
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_fixture_position(state: State<'_, Arc<AppState>>, fixture_id: usize, pan: f64, tilt: f64) {
    let mut lighting = state.lighting.write();
    if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
        f.pan = pan.clamp(-180.0, 180.0);
        f.tilt = tilt.clamp(-180.0, 180.0);
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_fixture_dimmer(state: State<'_, Arc<AppState>>, fixture_id: usize, dimmer: f64) {
    let mut lighting = state.lighting.write();
    if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
        f.dimmer = dimmer.clamp(0.0, 1.0);
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_fixture_strobe(state: State<'_, Arc<AppState>>, fixture_id: usize, strobe_on: bool, strobe_speed: f64) {
    let mut lighting = state.lighting.write();
    if let Some(f) = lighting.fixtures.get_mut(fixture_id) {
        f.strobe_on = strobe_on;
        f.strobe_speed = strobe_speed.clamp(0.0, 30.0);
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn select_fixtures(state: State<'_, Arc<AppState>>, ids: Vec<usize>) {
    let mut lighting = state.lighting.write();
    lighting.selected_fixture_ids = ids;
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_group(state: State<'_, Arc<AppState>>, id: String, name: String, fixture_ids: Vec<usize>) {
    let mut lighting = state.lighting.write();
    lighting.groups.push(Group { id, name, fixture_ids });
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_group(state: State<'_, Arc<AppState>>, id: String) {
    let mut lighting = state.lighting.write();
    lighting.groups.retain(|g| g.id != id);
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_group_fixtures(state: State<'_, Arc<AppState>>, id: String, fixture_ids: Vec<usize>) {
    let mut lighting = state.lighting.write();
    if let Some(g) = lighting.groups.iter_mut().find(|g| g.id == id) {
        g.fixture_ids = fixture_ids;
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_groups(state: State<'_, Arc<AppState>>) -> Vec<Group> {
    state.lighting.read().groups.clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_output_frame(state: State<'_, Arc<AppState>>) -> Vec<u8> {
    state.frame_buffer.read().clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_fixture_count(state: State<'_, Arc<AppState>>, count: usize) {
    let count = count.clamp(1, 32);
    let mut lighting = state.lighting.write();
    lighting.config.fixture_count = count;
    while lighting.fixtures.len() < count {
        let id = lighting.fixtures.len();
        lighting.fixtures.push(Fixture::new(id));
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_cue(state: State<'_, Arc<AppState>>, id: String, name: String, fade_time: f64) {
    let mut lighting = state.lighting.write();
    let fixtures = lighting.fixtures.clone();
    let cue = CueEntry {
        id: id.clone(),
        name,
        fixtures,
        fade_time,
    };
    if let Some(existing) = lighting.cues.iter_mut().find(|c| c.id == id) {
        *existing = cue;
    } else {
        lighting.cues.push(cue);
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn go_cue(state: State<'_, Arc<AppState>>, id: String) {
    let mut lighting = state.lighting.write();
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
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_cue(state: State<'_, Arc<AppState>>, id: String) {
    let mut lighting = state.lighting.write();
    lighting.cues.retain(|c| c.id != id);
    if lighting.active_cue_id.as_deref() == Some(&id) {
        lighting.active_cue_id = None;
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_cues(state: State<'_, Arc<AppState>>) -> Vec<CueEntry> {
    state.lighting.read().cues.clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_effect(state: State<'_, Arc<AppState>>, effect: EffectState) {
    let mut lighting = state.lighting.write();
    lighting.effect = effect;
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn clear_effect(state: State<'_, Arc<AppState>>) {
    let mut lighting = state.lighting.write();
    lighting.effect = EffectState::default();
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_config(state: State<'_, Arc<AppState>>) -> AppConfig {
    state.lighting.read().config.clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_config(state: State<'_, Arc<AppState>>, config: AppConfig) {
    let mut lighting = state.lighting.write();
    lighting.config = config;
    let new_count = lighting.config.fixture_count.clamp(1, 32);
    lighting.config.fixture_count = new_count;
    while lighting.fixtures.len() < new_count {
        let id = lighting.fixtures.len();
        lighting.fixtures.push(Fixture::new(id));
    }
    drop(lighting);
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_output_window_visible(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    visible: bool,
) -> Result<(), String> {
    {
        let mut lighting = state.lighting.write();
        lighting.config.output_window_visible = visible;
    }

    if let Some(window) = app.get_webview_window("output") {
        if visible {
            window.show().map_err(|e| e.to_string())?;
        } else {
            window.hide().map_err(|e| e.to_string())?;
        }
    }

    state.bump_version();
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_ndi_config(state: State<'_, Arc<AppState>>, enabled: bool, name: String) {
    {
        let mut lighting = state.lighting.write();
        lighting.config.ndi_enabled = enabled;
        lighting.config.ndi_name = name;
    }
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_spout_config(state: State<'_, Arc<AppState>>, enabled: bool, name: String) {
    {
        let mut lighting = state.lighting.write();
        lighting.config.spout_enabled = enabled;
        lighting.config.spout_name = name;
    }
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn reset_state(state: State<'_, Arc<AppState>>) {
    {
        let mut lighting = state.lighting.write();
        *lighting = LightingState::default();
    }
    state.bump_version();
}

#[tauri::command(rename_all = "snake_case")]
pub fn export_state(state: State<'_, Arc<AppState>>, path: String) -> Result<(), String> {
    let lighting = state.lighting.read().clone();
    let json = serde_json::to_string_pretty(&lighting).map_err(|e| e.to_string())?;
    let export_path = resolve_runtime_path(&path);
    std::fs::write(export_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_state(state: State<'_, Arc<AppState>>, path: String) -> Result<(), String> {
    let import_path = resolve_runtime_path(&path);
    let json = std::fs::read_to_string(import_path).map_err(|e| e.to_string())?;
    let imported: LightingState = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    {
        let mut lighting = state.lighting.write();
        *lighting = imported;
    }
    state.bump_version();
    Ok(())
}
