<script setup lang="ts">
import { computed, ref } from "vue";
import { useLightingStore } from "../stores/lighting";
import GroupPool from "./GroupPool.vue";

const store = useLightingStore();
const groupPanelOpen = ref(false);

const fixtures = computed(() => store.fixtures.slice(0, store.config.fixture_count));

function selectOnly(id: number, event: MouseEvent) {
  if (event.ctrlKey || event.metaKey) {
    const set = new Set(store.selected_fixture_ids);
    if (set.has(id)) set.delete(id);
    else set.add(id);
    store.selectFixtures([...set]);
    return;
  }
  store.selectFixtures([id]);
}

function rgbStyle(r: number, g: number, b: number, dimmer: number) {
  const rr = Math.round(r * dimmer * 255);
  const gg = Math.round(g * dimmer * 255);
  const bb = Math.round(b * dimmer * 255);
  return `rgb(${rr}, ${gg}, ${bb})`;
}

function selectAllFixtures() {
  const allIds = fixtures.value.map((fx) => fx.id);
  store.selectFixtures(allIds);
}
</script>

<template>
  <div class="status-strip">
    <button class="action-btn" @click="selectAllFixtures">全选</button>
    <button class="action-btn" :class="{ active: groupPanelOpen }" @click="groupPanelOpen = !groupPanelOpen">
      编组 {{ groupPanelOpen ? "▲" : "▼" }}
    </button>
    <button
      v-for="fx in fixtures"
      :key="fx.id"
      class="pill"
      :class="{ active: store.selected_fixture_ids.includes(fx.id) }"
      @click="selectOnly(fx.id, $event)"
    >
      <span
        class="dot"
        :style="{ backgroundColor: rgbStyle(fx.color.r, fx.color.g, fx.color.b, fx.dimmer) }"
      />
      <span class="id">{{ fx.id + 1 }}</span>
      <span class="coord">{{ fx.pan }}°/{{ fx.tilt }}°</span>
    </button>
  </div>
  <div v-if="groupPanelOpen" class="group-popover">
    <GroupPool />
  </div>
</template>

<style scoped>
.status-strip {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  padding: 0.35rem 0.5rem;
  background: #141d3a;
  border-bottom: 1px solid #0f3460;
}

.action-btn {
  border: 1px solid #355a8b;
  background: #103d6a;
  color: #dfe8ff;
  border-radius: 999px;
  min-height: 32px;
  padding: 0 12px;
  cursor: pointer;
  white-space: nowrap;
}

.action-btn.active {
  border-color: #e94560;
}

.pill {
  border: 1px solid #274875;
  background: #11244a;
  color: #d6deef;
  border-radius: 999px;
  min-height: 32px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  white-space: nowrap;
}

.pill.active {
  border-color: #e94560;
  box-shadow: inset 0 0 0 1px rgba(233, 69, 96, 0.4);
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.35);
}

.id {
  font-weight: 700;
}

.coord {
  opacity: 0.8;
  font-size: 0.75rem;
}

.group-popover {
  border-bottom: 1px solid #0f3460;
  background: #0f1b38;
}
</style>
