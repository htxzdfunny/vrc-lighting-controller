import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  LightingState,
  Fixture,
  EffectState,
  Group,
  CueEntry,
  AppConfig,
  OutputStatus,
} from "../types/lighting";
import { api } from "../api/adapter";

export type { EffectType } from "../types/lighting";

const defaultEffect: EffectState = {
  effect_type: "none",
  speed: 1.0,
  intensity: 1.0,
  phase_offset: 0.0,
};

const defaultConfig: AppConfig = {
  fixture_count: 10,
  web_port: 9000,
  ndi_enabled: false,
  ndi_name: "VRC Lighting",
  spout_enabled: false,
  spout_name: "VRC Lighting Spout2",
  output_window_visible: true,
  output_fps: 60,
};

const defaultOutputStatus: OutputStatus = {
  ndi: { state: "disabled", message: null },
  spout: { state: "disabled", message: null },
};

export const useLightingStore = defineStore("lighting", () => {
  const fixtures = ref<Fixture[]>([]);
  const groups = ref<Group[]>([]);
  const selected_fixture_ids = ref<number[]>([0]);
  const cues = ref<CueEntry[]>([]);
  const active_cue_id = ref<string | null>(null);
  const effect = ref<EffectState>({ ...defaultEffect });
  const config = ref<AppConfig>({ ...defaultConfig });
  const output_status = ref<OutputStatus>({ ...defaultOutputStatus });
  const connected = ref(false);
  const connectedClients = ref(0);
  const wsClientCount = ref(0);
  const ndiClientCount = ref(0);

  const selectedFixtures = computed(() =>
    fixtures.value.filter((f) => selected_fixture_ids.value.includes(f.id))
  );

  const currentFixture = computed<Fixture | null>(
    () => selectedFixtures.value[0] ?? null
  );

  function applyState(state: LightingState) {
    fixtures.value = state.fixtures;
    groups.value = state.groups;
    // 选区为每客户端独立的本地状态，不从服务端推送覆盖
    cues.value = state.cues;
    active_cue_id.value = state.active_cue_id;
    effect.value = state.effect;
    config.value = state.config;
    output_status.value = state.output_status ?? { ...defaultOutputStatus };
    if (state.connected_clients) {
      wsClientCount.value = state.connected_clients.ws;
      ndiClientCount.value = state.connected_clients.ndi;
      connectedClients.value = state.connected_clients.ws + state.connected_clients.ndi;
    }
  }

  function initConnection() {
    api.init((state: LightingState) => applyState(state));
    api.onConnectionChange((isConnected: boolean) => {
      connected.value = isConnected;
      if (isConnected) fetchState();
    });
    connected.value = api.isTauri;
    if (api.isTauri) fetchState();
  }

  async function fetchState() {
    try {
      const state = await api.getState();
      applyState(state);
    } catch {
      // will retry on reconnect
    }
  }

  async function setFixtureColor(
    fixtureId: number,
    r: number,
    g: number,
    b: number
  ) {
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) f.color = { r, g, b };
    await api.setFixtureColor(fixtureId, r, g, b);
  }

  async function setFixturePosition(
    fixtureId: number,
    pan: number,
    tilt: number
  ) {
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) {
      f.pan = pan;
      f.tilt = tilt;
    }
    await api.setFixturePosition(fixtureId, pan, tilt);
  }

  async function setFixtureDimmer(fixtureId: number, dimmer: number) {
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) f.dimmer = dimmer;
    await api.setFixtureDimmer(fixtureId, dimmer);
  }

  async function setFixtureStrobe(
    fixtureId: number,
    strobeOn: boolean,
    strobeSpeed: number
  ) {
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) {
      f.strobe_on = strobeOn;
      f.strobe_speed = strobeSpeed;
    }
    await api.setFixtureStrobe(fixtureId, strobeOn, strobeSpeed);
  }

  async function selectFixtures(ids: number[]) {
    // 选区为本地状态，不向服务端同步，避免多用户协作时覆盖他人选区
    selected_fixture_ids.value = ids;
  }

  async function setSelectedColor(r: number, g: number, b: number) {
    for (const f of selectedFixtures.value) {
      f.color = { r, g, b };
    }
    await api.setSelectedColor(selected_fixture_ids.value, r, g, b);
  }

  async function setSelectedPosition(pan: number, tilt: number) {
    for (const f of selectedFixtures.value) {
      f.pan = pan;
      f.tilt = tilt;
    }
    await api.setSelectedPosition(selected_fixture_ids.value, pan, tilt);
  }

  async function setSelectedDimmer(dimmer: number) {
    for (const f of selectedFixtures.value) {
      f.dimmer = dimmer;
    }
    await api.setSelectedDimmer(selected_fixture_ids.value, dimmer);
  }

  async function createGroup(name: string, fixtureIds: number[]) {
    await api.createGroup(name, fixtureIds);
    await fetchState();
  }

  async function deleteGroup(id: string) {
    groups.value = groups.value.filter((g) => g.id !== id);
    await api.deleteGroup(id);
  }

  async function setFixtureCount(count: number) {
    await api.setFixtureCount(count);
    await fetchState();
  }

  async function saveCue(name: string, fadeTime: number) {
    await api.saveCue(name, fadeTime);
    await fetchState();
  }

  async function goCue(id: string) {
    active_cue_id.value = id;
    await api.goCue(id);
    await fetchState();
  }

  async function deleteCue(id: string) {
    cues.value = cues.value.filter((c) => c.id !== id);
    if (active_cue_id.value === id) active_cue_id.value = null;
    await api.deleteCue(id);
  }

  async function setEffect(newEffect: EffectState) {
    effect.value = newEffect;
    await api.setEffect(newEffect);
  }

  async function clearEffect() {
    effect.value = { ...defaultEffect };
    await api.clearEffect();
  }

  async function setConfig(newConfig: AppConfig) {
    config.value = newConfig;
    await api.setConfig(newConfig);
    await fetchState();
  }

  async function setOutputWindowVisible(visible: boolean) {
    await api.setOutputWindowVisible(visible);
    config.value = { ...config.value, output_window_visible: visible };
  }

  async function resetState() {
    await api.resetState();
    await fetchState();
  }

  async function exportState(path: string) {
    await api.exportState(path);
  }

  async function importState(path: string) {
    await api.importState(path);
    await fetchState();
  }

  return {
    fixtures,
    groups,
    selected_fixture_ids,
    cues,
    active_cue_id,
    effect,
    config,
    output_status,
    connected,
    connectedClients,
    wsClientCount,
    ndiClientCount,
    selectedFixtures,
    currentFixture,
    initConnection,
    fetchState,
    setFixtureColor,
    setFixturePosition,
    setFixtureDimmer,
    setFixtureStrobe,
    selectFixtures,
    setSelectedColor,
    setSelectedPosition,
    setSelectedDimmer,
    createGroup,
    deleteGroup,
    setFixtureCount,
    saveCue,
    goCue,
    deleteCue,
    setEffect,
    clearEffect,
    setConfig,
    setOutputWindowVisible,
    resetState,
    exportState,
    importState,
  };
});
