<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../../stores/lighting";

const store = useLightingStore();

const NUMBER_KEYS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

const fixtures = computed(() => {
  const count = Math.min(10, store.fixtures.length);
  return store.fixtures.slice(0, count);
});

const hasSelection = computed(() => store.selected_fixture_ids.length > 0);

function fixtureColor(idx: number): string {
  const f = fixtures.value[idx];
  if (!f) return "#000";
  if (f.is_on === false) return "#000";
  const m = Math.max(0, Math.min(1, f.dimmer));
  const r = Math.round(Math.max(0, Math.min(1, f.color.r * m)) * 255);
  const g = Math.round(Math.max(0, Math.min(1, f.color.g * m)) * 255);
  const b = Math.round(Math.max(0, Math.min(1, f.color.b * m)) * 255);
  return `rgb(${r}, ${g}, ${b})`;
}

function isSelected(idx: number) {
  return store.selected_fixture_ids.includes(idx);
}

function isOn(idx: number) {
  return fixtures.value[idx]?.is_on !== false;
}

function onSlotPointerDown(idx: number) {
  if (idx >= fixtures.value.length) return;
  if (store.button_mode) {
    store.setFixtureOn(idx, true);
  } else {
    store.toggleFixtureOn(idx);
  }
}

function onSlotPointerUp(idx: number) {
  if (idx >= fixtures.value.length) return;
  if (store.button_mode) {
    store.setFixtureOn(idx, false);
  }
}

function onCheckboxChange(idx: number, e: Event) {
  if (idx >= fixtures.value.length) return;
  const checked = (e.target as HTMLInputElement).checked;
  const ids = new Set(store.selected_fixture_ids);
  if (checked) ids.add(idx);
  else ids.delete(idx);
  store.selectFixtures(Array.from(ids).sort((a, b) => a - b));
}

function toggleSelectAll() {
  if (hasSelection.value) {
    store.selectFixtures([]);
  } else {
    store.selectFixtures(fixtures.value.map((f) => f.id));
  }
}

function toggleButtonMode() {
  store.setButtonMode(!store.button_mode);
}
</script>

<template>
  <div class="toggle-row">
    <div class="left-controls">
      <button
        class="ctrl-btn"
        :class="{ active: store.editMode }"
        @click="store.toggleEditMode()"
        title="编辑模式：调色板/快照可保存与清除"
      >
        <span class="ctrl-label">编辑</span>
        <span class="ctrl-state">{{ store.editMode ? "ON" : "OFF" }}</span>
      </button>
      <button
        class="ctrl-btn"
        :class="{ active: hasSelection }"
        @click="toggleSelectAll"
        :title="hasSelection ? '清空所有选中' : '选中全部灯具'"
      >
        <span class="ctrl-label">{{ hasSelection ? "清空" : "全选" }}</span>
      </button>
      <button
        class="ctrl-btn flash-btn"
        :class="{ active: store.button_mode }"
        @click="toggleButtonMode"
        title="按钮模式：按住=亮，松开=灭"
      >
        <span class="ctrl-label">闪烁</span>
        <span class="ctrl-state">{{ store.button_mode ? "ON" : "OFF" }}</span>
      </button>
    </div>
    <div class="num-grid">
      <div v-for="(key, idx) in NUMBER_KEYS" :key="key" class="num-slot">
        <label
          class="num-check"
          :class="{ disabled: idx >= fixtures.length }"
          @pointerdown.stop
          @click.stop
        >
          <input
            type="checkbox"
            :checked="isSelected(idx)"
            :disabled="idx >= fixtures.length"
            @change="onCheckboxChange(idx, $event)"
          />
        </label>
        <div
          class="num-cell"
          :class="{
            empty: idx >= fixtures.length,
            selected: isSelected(idx),
            off: !isOn(idx) && idx < fixtures.length,
          }"
          :style="{ background: fixtureColor(idx) }"
          @pointerdown="onSlotPointerDown(idx)"
          @pointerup="onSlotPointerUp(idx)"
          @pointercancel="onSlotPointerUp(idx)"
          @pointerleave="onSlotPointerUp(idx)"
        >
          <span class="num-label">{{ key }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.toggle-row {
  display: flex;
  gap: clamp(6px, 1vw, 14px);
  align-items: stretch;
  padding: 6px 8px;
  background: #0d1730;
  border: 1px solid #21406f;
  border-radius: 8px;
  flex-shrink: 0;
}

.left-controls {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.ctrl-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1px;
  padding: 3px 8px;
  background: #152444;
  border: 1px solid #355a8b;
  border-radius: 4px;
  color: #b8c4dc;
  cursor: pointer;
  font-size: 0.7rem;
  min-width: 48px;
  min-height: 32px;
}

.ctrl-btn.active {
  background: #21406f;
  color: #fff;
  border-color: #4575aa;
}

.ctrl-btn.flash-btn.active {
  background: #e94560;
  border-color: #ff5e7d;
  color: #fff;
}

.ctrl-label {
  font-weight: 600;
  letter-spacing: 0.04em;
}

.ctrl-state {
  font-size: 0.6rem;
  opacity: 0.85;
  letter-spacing: 0.04em;
}

.num-grid {
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  gap: clamp(4px, 0.6vw, 8px);
  flex: 1;
  min-width: 0;
}

.num-slot {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  min-width: 0;
  gap: 3px;
}

.num-check {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 16px;
  cursor: pointer;
  flex-shrink: 0;
}

.num-check.disabled {
  cursor: not-allowed;
  opacity: 0.35;
}

.num-check input {
  width: 14px;
  height: 14px;
  margin: 0;
  cursor: inherit;
  accent-color: #e94560;
}

.num-cell {
  position: relative;
  border-radius: 6px;
  border: 1px solid #2a3e63;
  cursor: pointer;
  flex: 1 1 auto;
  min-height: clamp(36px, 5vh, 48px);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding-bottom: 3px;
  user-select: none;
  -webkit-user-select: none;
  touch-action: none;
  transition: transform 0.08s ease;
}

.num-cell:active {
  transform: scale(0.95);
}

.num-cell.empty {
  background: #050a18 !important;
  opacity: 0.4;
  cursor: not-allowed;
}

.num-cell.off::after {
  content: "";
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  border-radius: inherit;
  pointer-events: none;
}

.num-cell.selected {
  outline: 2px solid #e94560;
  outline-offset: -2px;
}

.num-label {
  font-size: clamp(0.72rem, 1.2vw, 0.95rem);
  color: rgba(255, 255, 255, 0.92);
  font-weight: 700;
  text-shadow: 0 0 3px rgba(0, 0, 0, 0.85);
  position: relative;
  z-index: 1;
}

@media (pointer: coarse) {
  .num-cell {
    min-height: clamp(44px, 6vh, 56px);
  }
  .ctrl-btn {
    min-width: 56px;
    min-height: 40px;
  }
  .num-check {
    height: 22px;
  }
  .num-check input {
    width: 18px;
    height: 18px;
  }
}
</style>
