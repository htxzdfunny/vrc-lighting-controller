import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { LightingState, EffectState, AppConfig } from "../types/lighting";

type StateUpdateCallback = (state: LightingState) => void;

/**
 * Subscribe to backend-initiated state changes (e.g. from LAN WebSocket clients).
 * Returns an unlisten function to cancel the subscription.
 */
export async function subscribeToStateChanges(
  callback: StateUpdateCallback
): Promise<UnlistenFn> {
  return listen<number>("state-changed", async () => {
    try {
      const state = await getState();
      callback(state);
    } catch {
      // ignore transient errors; next event will retry
    }
  });
}

export async function getState(): Promise<LightingState> {
  return invoke<LightingState>("get_state");
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
  for (const id of ids) {
    await invoke("set_fixture_color", { fixture_id: id, r, g, b });
  }
}

export async function setSelectedPosition(ids: number[], pan: number, tilt: number) {
  for (const id of ids) {
    await invoke("set_fixture_position", { fixture_id: id, pan, tilt });
  }
}

export async function setSelectedDimmer(ids: number[], dimmer: number) {
  for (const id of ids) {
    await invoke("set_fixture_dimmer", { fixture_id: id, dimmer });
  }
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
