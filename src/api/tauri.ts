import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { LightingState, EffectState, AppConfig, Color } from "../types/lighting";

type StateUpdateCallback = (state: LightingState) => void;

/**
 * Subscribe to backend-initiated state changes (e.g. from LAN WebSocket clients).
 * Returns an unlisten function to cancel the subscription.
 */
let _pendingStateFetch = false;

export async function subscribeToStateChanges(
  callback: StateUpdateCallback
): Promise<UnlistenFn> {
  return listen<number>("state-changed", async () => {
    if (_pendingStateFetch) return;
    _pendingStateFetch = true;
    try {
      const state = await getState();
      callback(state);
    } catch {
      // ignore transient errors; next event will retry
    }
    _pendingStateFetch = false;
  });
}

export async function getState(): Promise<LightingState> {
  return invoke<LightingState>("get_state");
}

export async function getAppVersion(): Promise<string> {
  return invoke<string>("get_app_version");
}

export async function setFixtureColor(fixtureId: number, r: number, g: number, b: number) {
  await invoke("set_fixture_color", { fixture_id: fixtureId, r, g, b });
}

export async function setFixturePosition(fixtureId: number, pan: number, tilt: number) {
  await invoke("set_fixture_position", { fixture_id: fixtureId, pan, tilt });
}

export async function setFixtureDimmer(fixtureId: number, dimmer: number) {
  await invoke("set_fixture_dimmer", { fixture_id: fixtureId, dimmer });
}

export async function setFixtureStrobe(fixtureId: number, strobeOn: boolean, strobeSpeed: number) {
  await invoke("set_fixture_strobe", { fixture_id: fixtureId, strobe_on: strobeOn, strobe_speed: strobeSpeed });
}

export async function selectFixtures(ids: number[]) {
  await invoke("select_fixtures", { ids });
}

export async function setSelectedColor(ids: number[], r: number, g: number, b: number) {
  await invoke("set_selected_color", { fixture_ids: ids, r, g, b });
}

export async function setSelectedPosition(ids: number[], pan: number, tilt: number) {
  await invoke("set_selected_position", { fixture_ids: ids, pan, tilt });
}

export async function setSelectedDimmer(ids: number[], dimmer: number) {
  await invoke("set_selected_dimmer", { fixture_ids: ids, dimmer });
}

export async function createGroup(name: string, fixtureIds: number[]) {
  const id = crypto.randomUUID();
  await invoke("create_group", { id, name, fixture_ids: fixtureIds });
}

export async function deleteGroup(id: string) {
  await invoke("delete_group", { id });
}

export async function setFixtureCount(count: number) {
  await invoke("set_fixture_count", { count });
}

export async function saveCue(name: string, fadeTime: number) {
  const id = crypto.randomUUID();
  await invoke("save_cue", { id, name, fade_time: fadeTime });
}

export async function goCue(id: string) {
  await invoke("go_cue", { id });
}

export async function deleteCue(id: string) {
  await invoke("delete_cue", { id });
}

export async function setEffect(effect: EffectState) {
  await invoke("set_effect", { effect });
}

export async function clearEffect() {
  await invoke("clear_effect");
}

export async function getOutputFrame(): Promise<number[]> {
  return invoke<number[]>("get_output_frame");
}

export async function setConfig(config: AppConfig) {
  await invoke("set_config", { config });
}

export async function setOutputWindowVisible(visible: boolean) {
  await invoke("set_output_window_visible", { visible });
}

export async function resetState() {
  await invoke("reset_state");
}

export async function exportState(path: string) {
  await invoke("export_state", { path });
}

export async function importState(path: string) {
  await invoke("import_state", { path });
}

export async function setPaletteSlot(index: number, color: Color | null) {
  await invoke("set_palette_slot", { index, color });
}

export async function applyPaletteSlot(index: number, fixtureIds: number[]) {
  await invoke("apply_palette_slot", { index, fixture_ids: fixtureIds });
}

export async function saveSnapshot(index: number, label: string) {
  await invoke("save_snapshot", { index, label });
}

export async function recallSnapshot(index: number) {
  await invoke("recall_snapshot", { index });
}

export async function clearSnapshot(index: number) {
  await invoke("clear_snapshot", { index });
}

export async function setFixtureSync(fixtureId: number, syncFader: boolean, syncKnob: boolean) {
  await invoke("set_fixture_sync", {
    fixture_id: fixtureId,
    sync_fader: syncFader,
    sync_knob: syncKnob,
  });
}

export async function setFixtureOn(fixtureId: number, on: boolean) {
  await invoke("set_fixture_on", { fixture_id: fixtureId, on });
}

export async function setButtonMode(enabled: boolean) {
  await invoke("set_button_mode", { enabled });
}

export async function applyMasterFader(value: number) {
  await invoke("apply_master_fader", { value });
}

export async function applyMasterKnob(value: number) {
  await invoke("apply_master_knob", { value });
}
