import { onBeforeUnmount, onMounted } from "vue";
import { useLightingStore } from "../stores/lighting";

const PALETTE_MAP: Record<string, number> = {
  q: 0,
  w: 1,
  e: 2,
  r: 3,
  t: 4,
  a: 5,
  s: 6,
  d: 7,
  f: 8,
  g: 9,
};

const SNAPSHOT_MAP: Record<string, number> = {
  z: 0,
  x: 1,
  c: 2,
  v: 3,
  b: 4,
  n: 5,
  m: 6,
};

function isInputFocused(): boolean {
  const ae = document.activeElement as HTMLElement | null;
  if (!ae) return false;
  if (ae.isContentEditable) return true;
  const tag = ae.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}

function fixtureIndexFromKey(key: string): number | null {
  if (key === "0") return 9;
  if (key.length === 1 && key >= "1" && key <= "9") return Number(key) - 1;
  return null;
}

export function usePerformanceKeyBindings() {
  const store = useLightingStore();
  const heldKeys = new Set<string>();

  function onKeyDown(e: KeyboardEvent) {
    if (isInputFocused()) return;
    if (e.repeat) return;

    const key = e.key.toLowerCase();
    const shift = e.shiftKey;

    // 1~0 灯具开关（选区由复选框管理，不受编辑模式影响）
    const fIdx = fixtureIndexFromKey(key);
    if (fIdx !== null) {
      e.preventDefault();
      heldKeys.add(key);
      if (store.button_mode) {
        store.setFixtureOn(fIdx, true);
      } else {
        store.toggleFixtureOn(fIdx);
      }
      return;
    }

    // QWERT / ASDFG → 颜色预设
    if (key in PALETTE_MAP) {
      e.preventDefault();
      const idx = PALETTE_MAP[key];
      if (shift) {
        store.clearPaletteSlot(idx);
      } else if (store.editMode) {
        store.savePaletteFromWorking(idx);
      } else {
        store.applyPaletteSlot(idx);
      }
      return;
    }

    // ZXCVBNM → 状态快照
    if (key in SNAPSHOT_MAP) {
      e.preventDefault();
      const idx = SNAPSHOT_MAP[key];
      if (shift && store.editMode) {
        store.saveSnapshot(idx);
      } else if (shift) {
        store.clearSnapshot(idx);
      } else {
        store.recallSnapshot(idx);
      }
      return;
    }
  }

  function onKeyUp(e: KeyboardEvent) {
    if (isInputFocused()) return;
    const key = e.key.toLowerCase();
    const fIdx = fixtureIndexFromKey(key);
    if (fIdx !== null) {
      heldKeys.delete(key);
      if (store.button_mode) {
        store.setFixtureOn(fIdx, false);
      }
    }
  }

  function onWindowBlur() {
    // 释放所有按下的灯键，避免 button_mode 下的"卡住"
    if (!store.button_mode) {
      heldKeys.clear();
      return;
    }
    for (const k of heldKeys) {
      const fIdx = fixtureIndexFromKey(k);
      if (fIdx !== null) store.setFixtureOn(fIdx, false);
    }
    heldKeys.clear();
  }

  onMounted(() => {
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    window.addEventListener("blur", onWindowBlur);
  });

  onBeforeUnmount(() => {
    window.removeEventListener("keydown", onKeyDown);
    window.removeEventListener("keyup", onKeyUp);
    window.removeEventListener("blur", onWindowBlur);
  });
}
