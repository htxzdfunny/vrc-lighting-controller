<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../../stores/lighting";

const store = useLightingStore();

const PALETTE_KEYS = ["Q", "W", "E", "R", "T", "A", "S", "D", "F", "G"];

const slots = computed(() => {
  const palette = store.color_palette;
  return PALETTE_KEYS.map((key, idx) => {
    const c = palette[idx]?.color ?? null;
    return {
      idx,
      key,
      color: c,
      bg: c
        ? `rgb(${Math.round(c.r * 255)}, ${Math.round(c.g * 255)}, ${Math.round(c.b * 255)})`
        : "#000",
      empty: !c,
    };
  });
});

function onSlotClick(idx: number) {
  if (store.editMode) {
    store.savePaletteFromWorking(idx);
  } else {
    store.applyPaletteSlot(idx);
  }
}

function onClearClick(idx: number, e: Event) {
  e.stopPropagation();
  store.clearPaletteSlot(idx);
}
</script>

<template>
  <div class="palette-pad">
    <div class="palette-grid">
      <div
        v-for="slot in slots"
        :key="slot.idx"
        class="slot"
        :class="{ empty: slot.empty, edit: store.editMode }"
        :style="{ background: slot.bg }"
        @click="onSlotClick(slot.idx)"
      >
        <span class="slot-key">{{ slot.key }}</span>
        <button
          v-if="store.editMode && !slot.empty"
          class="clear-btn"
          aria-label="清除该预设"
          @click="onClearClick(slot.idx, $event)"
        >
          ×
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.palette-pad {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  min-height: 0;
}

.palette-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  grid-auto-rows: 1fr;
  gap: clamp(4px, 0.8vw, 10px);
  flex: 1;
  min-height: 0;
}

.slot {
  position: relative;
  border-radius: 6px;
  border: 1px solid #2a3e63;
  cursor: pointer;
  display: flex;
  align-items: flex-end;
  justify-content: flex-start;
  padding: 4px 6px;
  min-height: 38px;
  transition: transform 0.08s ease;
  user-select: none;
  -webkit-user-select: none;
}

.slot:active {
  transform: scale(0.96);
}

.slot.empty {
  background: #0a1228 !important;
  border-style: dashed;
  border-color: #2a3e63;
}

.slot.edit {
  outline: 2px solid #e94560;
  outline-offset: -2px;
}

.slot-key {
  font-size: clamp(0.7rem, 1.2vw, 0.95rem);
  color: rgba(255, 255, 255, 0.85);
  font-weight: 700;
  text-shadow: 0 0 3px rgba(0, 0, 0, 0.8);
  letter-spacing: 0.05em;
}

.empty .slot-key {
  color: #4a5a7c;
  text-shadow: none;
}

.clear-btn {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.4);
  background: rgba(0, 0, 0, 0.7);
  color: #fff;
  font-size: 0.95rem;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

@media (pointer: coarse) {
  .clear-btn {
    width: 28px;
    height: 28px;
    font-size: 1.05rem;
  }
}
</style>
