#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;
mod engine;
mod render;
mod server;
mod output;
mod ipc;
mod persistence;

use std::sync::Arc;
use tauri::Manager;
use state::AppState;

fn init_logger() {
    use simplelog::*;
    use std::fs::File;

    let log_path = persistence::get_runtime_dir().join("app.log");

    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    CombinedLogger::init(vec![
        TermLogger::new(log_level, Config::default(), TerminalMode::Stderr, ColorChoice::Auto),
        WriteLogger::new(log_level, Config::default(), File::create(&log_path).unwrap()),
    ])
    .expect("failed to init logger");

    log::info!("日志文件: {}", log_path.display());
}

fn main() {
    init_logger();

    let app_state = Arc::new(AppState::new());

    persistence::load_state(&app_state);
    persistence::start_autosave(app_state.clone());

    let server_state = app_state.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(server::run(server_state));
    });

    let render_state = app_state.clone();
    std::thread::spawn(move || {
        render::run_render_loop(render_state);
    });

    output::ndi::start_ndi_output(app_state.clone());
    output::spout::start_spout_output(app_state.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state.clone())
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    let app = window.app_handle();
                    let state = app_state.clone();
                    persistence::save_state(&state);
                    app.exit(0);
                }
            }
        })
        .setup(|app| {
            let state = app.state::<Arc<AppState>>();
            // Inject the AppHandle so bump_version() can emit "state-changed" events
            // to the frontend, keeping the desktop UI in sync with WS-driven updates.
            state.set_app_handle(app.handle().clone());
            let visible = {
                let lighting = state.lighting.read();
                lighting.config.output_window_visible
            };
            if !visible {
                if let Some(win) = app.get_webview_window("output") {
                    let _ = win.hide();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::get_state,
            ipc::set_fixture,
            ipc::set_fixture_color,
            ipc::set_fixture_position,
            ipc::set_fixture_dimmer,
            ipc::set_fixture_strobe,
            ipc::select_fixtures,
            ipc::create_group,
            ipc::delete_group,
            ipc::set_group_fixtures,
            ipc::get_groups,
            ipc::get_output_frame,
            ipc::set_fixture_count,
            ipc::save_cue,
            ipc::go_cue,
            ipc::delete_cue,
            ipc::get_cues,
            ipc::set_effect,
            ipc::clear_effect,
            ipc::get_config,
            ipc::set_config,
            ipc::set_output_window_visible,
            ipc::set_ndi_config,
            ipc::set_spout_config,
            ipc::reset_state,
            ipc::export_state,
            ipc::import_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
