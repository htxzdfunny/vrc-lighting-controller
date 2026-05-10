import type { LightingState, EffectState } from "../types/lighting";

type StateUpdateCallback = (state: LightingState) => void;
type ConnectionCallback = (connected: boolean) => void;

let socket: WebSocket | null = null;
let stateCallback: StateUpdateCallback | null = null;
let connectionCallback: ConnectionCallback | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

function getWsUrl(): string {
  // Use the port the page was served on (matches web_port setting).
  // Falls back to 9000 when opened as a local file or in Tauri dev mode.
  const port = location.port || "9000";
  return `ws://${location.hostname}:${port}/ws`;
}

function scheduleReconnect() {
  if (reconnectTimer) return;
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    connect();
  }, 3000);
}

export function connect() {
  if (
    socket &&
    (socket.readyState === WebSocket.OPEN ||
      socket.readyState === WebSocket.CONNECTING)
  ) {
    return;
  }

  socket = new WebSocket(getWsUrl());

  socket.onopen = () => {
    connectionCallback?.(true);
  };

  socket.onclose = () => {
    connectionCallback?.(false);
    socket = null;
    scheduleReconnect();
  };

  socket.onerror = () => {
    socket?.close();
  };

  socket.onmessage = (event: MessageEvent) => {
    try {
      const msg = JSON.parse(event.data as string);
      if (msg.type === "state" && msg.data) {
        stateCallback?.(msg.data as LightingState);
      }
    } catch {
      // malformed message
    }
  };
}

export function disconnect() {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  socket?.close();
  socket = null;
}

export function onStateUpdate(callback: StateUpdateCallback) {
  stateCallback = callback;
}

export function onConnectionChange(callback: ConnectionCallback) {
  connectionCallback = callback;
}

function sendCommand(cmd: Record<string, unknown>) {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(cmd));
  }
}

export async function getState(): Promise<LightingState> {
  sendCommand({ type: "get_state" });
  return new Promise((resolve) => {
    const origCb = stateCallback;
    stateCallback = (state) => {
      stateCallback = origCb;
      resolve(state);
      origCb?.(state);
    };
  });
}

export async function setFixtureColor(fixtureId: number, r: number, g: number, b: number) {
  sendCommand({ type: "set_fixture_color", data: { fixture_id: fixtureId, r, g, b } });
}

export async function setFixturePosition(fixtureId: number, pan: number, tilt: number) {
  sendCommand({ type: "set_fixture_position", data: { fixture_id: fixtureId, pan, tilt } });
}

export async function setFixtureDimmer(fixtureId: number, dimmer: number) {
  sendCommand({ type: "set_fixture_dimmer", data: { fixture_id: fixtureId, dimmer } });
}

export async function setFixtureStrobe(fixtureId: number, strobeOn: boolean, strobeSpeed: number) {
  sendCommand({ type: "set_fixture_strobe", data: { fixture_id: fixtureId, strobe_on: strobeOn, strobe_speed: strobeSpeed } });
}

export async function selectFixtures(ids: number[]) {
  sendCommand({ type: "select_fixtures", data: { ids } });
}

// 批量命令：先同步选区到服务端，再用单条命令修改所有选中灯具，
// 避免对 N 个灯具发送 N 条消息造成卡顿。
export async function setSelectedColor(ids: number[], r: number, g: number, b: number) {
  sendCommand({ type: "select_fixtures", data: { ids } });
  sendCommand({ type: "set_selected_color", data: { r, g, b } });
}

export async function setSelectedPosition(ids: number[], pan: number, tilt: number) {
  sendCommand({ type: "select_fixtures", data: { ids } });
  sendCommand({ type: "set_selected_position", data: { pan, tilt } });
}

export async function setSelectedDimmer(ids: number[], dimmer: number) {
  sendCommand({ type: "select_fixtures", data: { ids } });
  sendCommand({ type: "set_selected_dimmer", data: { dimmer } });
}

export async function createGroup(name: string, fixtureIds: number[]) {
  const id = crypto.randomUUID();
  sendCommand({ type: "create_group", data: { id, name, fixture_ids: fixtureIds } });
}

export async function deleteGroup(id: string) {
  sendCommand({ type: "delete_group", data: { id } });
}

export async function setFixtureCount(count: number) {
  sendCommand({ type: "set_fixture_count", data: { count } });
}

export async function saveCue(name: string, fadeTime: number) {
  const id = crypto.randomUUID();
  sendCommand({ type: "save_cue", data: { id, name, fade_time: fadeTime } });
}

export async function goCue(id: string) {
  sendCommand({ type: "go_cue", data: { id } });
}

export async function deleteCue(id: string) {
  sendCommand({ type: "delete_cue", data: { id } });
}

export async function setEffect(effect: EffectState) {
  sendCommand({ type: "set_effect", data: { effect } });
}

export async function clearEffect() {
  sendCommand({ type: "clear_effect" });
}

export async function setConfig(config: Record<string, unknown>) {
  sendCommand({ type: "set_config", data: { config } });
}

