export interface Color {
  r: number;
  g: number;
  b: number;
}

export interface Fixture {
  id: number;
  name: string;
  pan: number;
  tilt: number;
  color: Color;
  dimmer: number;
  strobe_on: boolean;
  strobe_speed: number;
}

export interface Group {
  id: string;
  name: string;
  fixture_ids: number[];
}

export interface CueEntry {
  id: string;
  name: string;
  fixtures: Fixture[];
  fade_time: number;
}

export type EffectType =
  | "none"
  | "color_cycle"
  | "scan"
  | "pulse"
  | "wave"
  | "random";

export interface EffectState {
  effect_type: EffectType;
  speed: number;
  intensity: number;
  phase_offset: number;
}

export interface AppConfig {
  fixture_count: number;
  web_port: number;
  ndi_enabled: boolean;
  ndi_name: string;
  spout_enabled: boolean;
  spout_name: string;
  output_window_visible: boolean;
  output_fps: number;
}

export type OutputStateKind = "disabled" | "active" | "error";

export interface OutputEndpointStatus {
  state: OutputStateKind;
  message: string | null;
}

export interface OutputStatus {
  ndi: OutputEndpointStatus;
  spout: OutputEndpointStatus;
}

export interface ConnectedClients {
  ws: number;
  ndi: number;
}

export interface LightingState {
  fixtures: Fixture[];
  groups: Group[];
  selected_fixture_ids: number[];
  cues: CueEntry[];
  active_cue_id: string | null;
  effect: EffectState;
  config: AppConfig;
  output_status: OutputStatus;
  connected_clients?: ConnectedClients;
}
