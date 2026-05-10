use std::path::PathBuf;
use std::sync::Arc;
use crate::state::{AppState, LightingState};

pub fn get_runtime_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn get_save_path() -> PathBuf {
    get_runtime_dir().join("state.json")
}

pub fn save_state(state: &AppState) {
    let lighting = state.lighting.read().clone();
    let path = get_save_path();
    match serde_json::to_string_pretty(&lighting) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::warn!("Failed to save state: {}", e);
            } else {
                log::debug!("State saved to {:?}", path);
            }
        }
        Err(e) => log::warn!("Failed to serialize state: {}", e),
    }
}

pub fn load_state(state: &AppState) {
    let path = get_save_path();
    if !path.exists() {
        log::info!("No saved state found at {:?}", path);
        return;
    }
    match std::fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str::<LightingState>(&json) {
            Ok(loaded) => {
                let mut lighting = state.lighting.write();
                *lighting = loaded;
                log::info!("State loaded from {:?}", path);
            }
            Err(e) => log::warn!("Failed to parse saved state: {}", e),
        },
        Err(e) => log::warn!("Failed to read saved state: {}", e),
    }
}

pub fn start_autosave(state: Arc<AppState>) {
    std::thread::spawn(move || {
        let mut last_version = 0u64;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let current = state.current_version();
            if current != last_version {
                last_version = current;
                save_state(&state);
            }
        }
    });
}
