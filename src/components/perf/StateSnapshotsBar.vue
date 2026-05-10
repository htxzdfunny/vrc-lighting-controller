<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../../stores/lighting";

const store = useLightingStore();

const SNAPSHOT_KEYS = ["Z", "X", "C", "V", "B", "N", "M"];

const slots = computed(() =>
  SNAPSHOT_KEYS.map((key, idx) => {
    const snap = store.state_snapshots[idx];
    return {
      idx,
      key,
      filled: !!snap,
      label: snap?.label ?? "",
    };
  })
);

function onSlotClick(idx: number) {
  if (store.editMode) {
    store.saveSnapshot(idx);
  } else {
    store.recallSnapshot(idx);
  }
}

function onClearClick(idx: number, e: Event) {
  e.stopPropagation();
  store.clearSnapshot(idx);
}
</script>

<template>
  <div class="snapshots-bar">
    <span class="title">状态快照</span>
    <div class="slots">
      <div
        v-for="slot in slots"
        :key="slot.idx"
        class="slot"
        :class="{ filled: slot.filled, edit: store.editMode }"
        @click="onSlotClick(slot.idx)"
      >
        <span class="slot-key">{{ slot.key }}</span>
        <span v-if="slot.filled" class="slot-dot" />
        <button
          v-if="store.editMode && slot.filled"
          class="clear-btn"
          aria-label="清除该快照"
          @click="onClearClick(slot.idx, $event)"
        >
          ×
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.snapshots-bar {
  display: flex;
  align-items: center;
  gap: clamp(6px, 1vw, 12px);
  padding: 6px 8px;
  background: #0d1730;
  border: 1px solid #21406f;
  border-radius: 8px;
}

.title {
  font-size: clamp(0.7rem, 1.1vw, 0.82rem);
  color: #b8c4dc;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.slots {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: clamp(4px, 0.6vw, 8px);
  flex: 1;
  min-width: 0;
}

.slot {
  position: relative;
  background: #152444;
  border: 1px solid #355a8b;
  border-radius: 6px;
  min-height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  user-select: none;
  -webkit-user-select: none;
  transition: transform 0.08s ease;
}

.slot:active {
  transform: scale(0.95);
}

.slot.filled {
  background: #21406f;
  border-color: #4575aa;
}

.slot.edit {
  outline: 2px dashed #e94560;
  outline-offset: -2px;
}

.slot-key {
  font-size: clamp(0.78rem, 1.2vw, 0.95rem);
  color: #dfe8ff;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.slot-dot {
  position: absolute;
  top: 4px;
  left: 4px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #67ddb1;
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
  .slot {
    min-height: 48px;
  }
  .clear-btn {
    width: 28px;
    height: 28px;
  }
}
</style>
