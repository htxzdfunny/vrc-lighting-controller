import type { LightingState, EffectState, AppConfig, Color } from "../types/lighting";
import * as tauriApi from "./tauri";
import * as wsApi from "./ws";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export type StateUpdateCallback = (state: LightingState) => void;

const _isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
const backend = _isTauri ? tauriApi : wsApi;

// Holds the Tauri event unlisten function so we can clean it up on destroy.
let _tauriUnlisten: (() => void) | null = null;

function init(onStateUpdate: StateUpdateCallback) {
  if (_isTauri) {
    // Subscribe to backend-pushed "state-changed" events so that changes made
    // by LAN WebSocket clients are immediately reflected in the desktop UI.
    tauriApi.subscribeToStateChanges(onStateUpdate).then((unlisten) => {
      _tauriUnlisten = unlisten;
    });
  } else {
    wsApi.onStateUpdate(onStateUpdate);
    wsApi.connect();
  }
}

function destroy() {
  if (_isTauri) {
    _tauriUnlisten?.();
    _tauriUnlisten = null;
  } else {
    wsApi.disconnect();
  }
}

function onConnectionChange(callback: (connected: boolean) => void) {
  if (!_isTauri) {
    wsApi.onConnectionChange(callback);
  }
}

async function getState(): Promise<LightingState> {
  return backend.getState();
}

async function setFixtureColor(fixtureId: number, r: number, g: number, b: number) {
  return backend.setFixtureColor(fixtureId, r, g, b);
}

async function setFixturePosition(fixtureId: number, pan: number, tilt: number) {
  return backend.setFixturePosition(fixtureId, pan, tilt);
}

async function setFixtureDimmer(fixtureId: number, dimmer: number) {
  return backend.setFixtureDimmer(fixtureId, dimmer);
}

async function setFixtureStrobe(fixtureId: number, strobeOn: boolean, strobeSpeed: number) {
  return backend.setFixtureStrobe(fixtureId, strobeOn, strobeSpeed);
}

async function selectFixtures(ids: number[]) {
  return backend.selectFixtures(ids);
}

async function setSelectedColor(ids: number[], r: number, g: number, b: number) {
  return backend.setSelectedColor(ids, r, g, b);
}

async function setSelectedPosition(ids: number[], pan: number, tilt: number) {
  return backend.setSelectedPosition(ids, pan, tilt);
}

async function setSelectedDimmer(ids: number[], dimmer: number) {
  return backend.setSelectedDimmer(ids, dimmer);
}

async function createGroup(name: string, fixtureIds: number[]) {
  return backend.createGroup(name, fixtureIds);
}

async function deleteGroup(id: string) {
  return backend.deleteGroup(id);
}

async function setFixtureCount(count: number) {
  return backend.setFixtureCount(count);
}

async function saveCue(name: string, fadeTime: number) {
  return backend.saveCue(name, fadeTime);
}

async function goCue(id: string) {
  return backend.goCue(id);
}

async function deleteCue(id: string) {
  return backend.deleteCue(id);
}

async function setEffect(effect: EffectState) {
  return backend.setEffect(effect);
}

async function clearEffect() {
  return backend.clearEffect();
}

async function getOutputFrame(): Promise<number[]> {
  if (_isTauri) {
    return tauriApi.getOutputFrame();
  }
  return [];
}

async function setConfig(config: AppConfig) {
  return backend.setConfig(config as any);
}

async function setOutputWindowVisible(visible: boolean) {
  if (_isTauri) return tauriApi.setOutputWindowVisible(visible);
}

async function resetState() {
  if (_isTauri) return tauriApi.resetState();
}

async function exportState(path: string) {
  if (_isTauri) return tauriApi.exportState(path);
}

async function importState(path: string) {
  if (_isTauri) return tauriApi.importState(path);
}

async function setPaletteSlot(index: number, color: Color | null) {
  return backend.setPaletteSlot(index, color);
}

async function applyPaletteSlot(index: number, fixtureIds: number[]) {
  return backend.applyPaletteSlot(index, fixtureIds);
}

async function saveSnapshot(index: number, label: string) {
  return backend.saveSnapshot(index, label);
}

async function recallSnapshot(index: number) {
  return backend.recallSnapshot(index);
}

async function clearSnapshot(index: number) {
  return backend.clearSnapshot(index);
}

async function setFixtureSync(fixtureId: number, syncFader: boolean, syncKnob: boolean) {
  return backend.setFixtureSync(fixtureId, syncFader, syncKnob);
}

async function setFixtureOn(fixtureId: number, on: boolean) {
  return backend.setFixtureOn(fixtureId, on);
}

async function setButtonMode(enabled: boolean) {
  return backend.setButtonMode(enabled);
}

async function applyMasterFader(value: number) {
  return backend.applyMasterFader(value);
}

async function applyMasterKnob(value: number) {
  return backend.applyMasterKnob(value);
}

async function getAppVersion(): Promise<string> {
  if (_isTauri) return tauriApi.getAppVersion();
  return "Web";
}

export const api = {
  isTauri: _isTauri,
  init,
  destroy,
  onConnectionChange,
  getState,
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
  getOutputFrame,
  setConfig,
  setOutputWindowVisible,
  resetState,
  exportState,
  importState,
  setPaletteSlot,
  applyPaletteSlot,
  saveSnapshot,
  recallSnapshot,
  clearSnapshot,
  setFixtureSync,
  setFixtureOn,
  setButtonMode,
  applyMasterFader,
  applyMasterKnob,
  getAppVersion,
};
