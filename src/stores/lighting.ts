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
  Color,
  PaletteSlot,
  StateSnapshot,
} from "../types/lighting";
import { PALETTE_SLOT_COUNT, SNAPSHOT_SLOT_COUNT } from "../types/lighting";
import { api } from "../api/adapter";

export type { EffectType } from "../types/lighting";

function makeDefaultPalette(): PaletteSlot[] {
  return Array.from({ length: PALETTE_SLOT_COUNT }, () => ({ color: null }));
}

function makeDefaultSnapshots(): (StateSnapshot | null)[] {
  return Array.from({ length: SNAPSHOT_SLOT_COUNT }, () => null);
}

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

  const color_palette = ref<PaletteSlot[]>(makeDefaultPalette());
  const state_snapshots = ref<(StateSnapshot | null)[]>(makeDefaultSnapshots());
  const button_mode = ref(false);

  // 仅前端本地状态（不存后端）
  const editMode = ref(false);
  // 颜色编辑器中暂存的工作色（编辑模式下保存到预设的来源）
  const workingColor = ref<Color>({ r: 1, g: 1, b: 1 });
  const workingMaster = ref(1.0);

  // 拖拽调参期间抑制服务端状态回弹覆盖
  const suppressServerUpdates = ref(false);
  let suppressTimer: ReturnType<typeof setTimeout> | null = null;
  function beginSuppressUpdates() {
    suppressServerUpdates.value = true;
    if (suppressTimer) clearTimeout(suppressTimer);
  }
  function endSuppressUpdates() {
    suppressTimer = setTimeout(() => {
      suppressServerUpdates.value = false;
    }, 150);
  }

  const selectedFixtures = computed(() =>
    fixtures.value.filter((f) => selected_fixture_ids.value.includes(f.id))
  );

  const currentFixture = computed<Fixture | null>(
    () => selectedFixtures.value[0] ?? null
  );

  function applyState(state: LightingState) {
    if (!suppressServerUpdates.value) {
      fixtures.value = state.fixtures;
    }
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

    const palette = state.color_palette ?? [];
    const padded = palette.slice(0, PALETTE_SLOT_COUNT);
    while (padded.length < PALETTE_SLOT_COUNT) padded.push({ color: null });
    color_palette.value = padded;

    const snaps = state.state_snapshots ?? [];
    const snapsPadded = snaps.slice(0, SNAPSHOT_SLOT_COUNT) as (StateSnapshot | null)[];
    while (snapsPadded.length < SNAPSHOT_SLOT_COUNT) snapsPadded.push(null);
    state_snapshots.value = snapsPadded;

    button_mode.value = state.button_mode ?? false;
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
    beginSuppressUpdates();
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) f.color = { r, g, b };
    await api.setFixtureColor(fixtureId, r, g, b);
    endSuppressUpdates();
  }

  async function setFixturePosition(
    fixtureId: number,
    pan: number,
    tilt: number
  ) {
    beginSuppressUpdates();
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) {
      f.pan = pan;
      f.tilt = tilt;
    }
    await api.setFixturePosition(fixtureId, pan, tilt);
    endSuppressUpdates();
  }

  async function setFixtureDimmer(fixtureId: number, dimmer: number) {
    beginSuppressUpdates();
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) f.dimmer = dimmer;
    await api.setFixtureDimmer(fixtureId, dimmer);
    endSuppressUpdates();
  }

  async function setFixtureStrobe(
    fixtureId: number,
    strobeOn: boolean,
    strobeSpeed: number
  ) {
    beginSuppressUpdates();
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) {
      f.strobe_on = strobeOn;
      f.strobe_speed = strobeSpeed;
    }
    await api.setFixtureStrobe(fixtureId, strobeOn, strobeSpeed);
    endSuppressUpdates();
  }

  async function selectFixtures(ids: number[]) {
    // 选区为本地状态，不向服务端同步，避免多用户协作时覆盖他人选区
    selected_fixture_ids.value = ids;
  }

  async function setSelectedColor(r: number, g: number, b: number) {
    beginSuppressUpdates();
    for (const f of selectedFixtures.value) {
      f.color = { r, g, b };
    }
    await api.setSelectedColor(selected_fixture_ids.value, r, g, b);
    endSuppressUpdates();
  }

  async function setSelectedPosition(pan: number, tilt: number) {
    beginSuppressUpdates();
    for (const f of selectedFixtures.value) {
      f.pan = pan;
      f.tilt = tilt;
    }
    await api.setSelectedPosition(selected_fixture_ids.value, pan, tilt);
    endSuppressUpdates();
  }

  async function setSelectedDimmer(dimmer: number) {
    beginSuppressUpdates();
    for (const f of selectedFixtures.value) {
      f.dimmer = dimmer;
    }
    await api.setSelectedDimmer(selected_fixture_ids.value, dimmer);
    endSuppressUpdates();
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

  // ── Performance / palette / snapshot / on-off / sync / button mode ──

  function getEffectiveColor(): Color {
    const m = Math.max(0, Math.min(1, workingMaster.value));
    return {
      r: Math.max(0, Math.min(1, workingColor.value.r * m)),
      g: Math.max(0, Math.min(1, workingColor.value.g * m)),
      b: Math.max(0, Math.min(1, workingColor.value.b * m)),
    };
  }

  async function setPaletteSlot(index: number, color: Color | null) {
    if (index < 0 || index >= PALETTE_SLOT_COUNT) return;
    const slot = color_palette.value[index];
    if (slot) slot.color = color;
    await api.setPaletteSlot(index, color);
  }

  async function applyPaletteSlot(index: number, fixtureIds: number[] | null = null) {
    if (index < 0 || index >= PALETTE_SLOT_COUNT) return;
    const slot = color_palette.value[index];
    const color = slot?.color ?? { r: 0, g: 0, b: 0 };
    const ids = fixtureIds ?? selected_fixture_ids.value;
    for (const id of ids) {
      const f = fixtures.value.find((fx) => fx.id === id);
      if (f) f.color = { ...color };
    }
    await api.applyPaletteSlot(index, ids);
  }

  async function clearPaletteSlot(index: number) {
    await setPaletteSlot(index, null);
  }

  async function savePaletteFromWorking(index: number) {
    await setPaletteSlot(index, getEffectiveColor());
  }

  async function saveSnapshot(index: number, label = "") {
    if (index < 0 || index >= SNAPSHOT_SLOT_COUNT) return;
    const snapshot: StateSnapshot = {
      label,
      fixtures: fixtures.value.map((f) => ({ ...f, color: { ...f.color } })),
    };
    state_snapshots.value[index] = snapshot;
    await api.saveSnapshot(index, label);
  }

  async function recallSnapshot(index: number) {
    if (index < 0 || index >= SNAPSHOT_SLOT_COUNT) return;
    const snapshot = state_snapshots.value[index];
    if (!snapshot) return;
    const liveSync = fixtures.value.map((f) => ({
      sync_master_fader: f.sync_master_fader,
      sync_master_knob: f.sync_master_knob,
    }));
    const count = Math.min(snapshot.fixtures.length, fixtures.value.length);
    for (let i = 0; i < count; i++) {
      const src = snapshot.fixtures[i];
      const sync = liveSync[i] ?? {
        sync_master_fader: true,
        sync_master_knob: true,
      };
      fixtures.value[i] = {
        ...src,
        color: { ...src.color },
        sync_master_fader: sync.sync_master_fader,
        sync_master_knob: sync.sync_master_knob,
      };
    }
    await api.recallSnapshot(index);
  }

  async function clearSnapshot(index: number) {
    if (index < 0 || index >= SNAPSHOT_SLOT_COUNT) return;
    state_snapshots.value[index] = null;
    await api.clearSnapshot(index);
  }

  async function setFixtureSync(
    fixtureId: number,
    syncFader: boolean,
    syncKnob: boolean
  ) {
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) {
      f.sync_master_fader = syncFader;
      f.sync_master_knob = syncKnob;
    }
    await api.setFixtureSync(fixtureId, syncFader, syncKnob);
  }

  async function setFixtureOn(fixtureId: number, on: boolean) {
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) f.is_on = on;
    await api.setFixtureOn(fixtureId, on);
  }

  async function toggleFixtureOn(fixtureId: number) {
    const f = fixtures.value.find((fx) => fx.id === fixtureId);
    if (f) await setFixtureOn(fixtureId, !f.is_on);
  }

  async function setButtonMode(enabled: boolean) {
    button_mode.value = enabled;
    await api.setButtonMode(enabled);
  }

  async function applyMasterFader(value: number) {
    beginSuppressUpdates();
    const v = Math.max(0, Math.min(1, value));
    const tilt = v * 360 - 180;
    for (const f of fixtures.value) {
      if (f.sync_master_fader) f.tilt = tilt;
    }
    await api.applyMasterFader(v);
    endSuppressUpdates();
  }

  async function applyMasterKnob(value: number) {
    beginSuppressUpdates();
    const v = Math.max(0, Math.min(1, value));
    const pan = v * 360 - 180;
    for (const f of fixtures.value) {
      if (f.sync_master_knob) f.pan = pan;
    }
    await api.applyMasterKnob(v);
    endSuppressUpdates();
  }

  function setEditMode(value: boolean) {
    editMode.value = value;
  }

  function toggleEditMode() {
    editMode.value = !editMode.value;
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
    color_palette,
    state_snapshots,
    button_mode,
    editMode,
    workingColor,
    workingMaster,
    getEffectiveColor,
    setPaletteSlot,
    applyPaletteSlot,
    clearPaletteSlot,
    savePaletteFromWorking,
    saveSnapshot,
    recallSnapshot,
    clearSnapshot,
    setFixtureSync,
    setFixtureOn,
    toggleFixtureOn,
    setButtonMode,
    applyMasterFader,
    applyMasterKnob,
    setEditMode,
    toggleEditMode,
  };
});
