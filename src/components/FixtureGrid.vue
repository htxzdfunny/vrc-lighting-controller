<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../stores/lighting";

const store = useLightingStore();

const fixtures = computed(() =>
  store.fixtures.slice(0, store.config.fixture_count)
);

const selectedIds = computed(() => new Set(store.selected_fixture_ids));

function toggleFixture(id: number, event: MouseEvent | TouchEvent) {
  const isMulti =
    event instanceof MouseEvent
      ? event.ctrlKey || event.metaKey
      : false;

  if (isMulti) {
    const current = [...store.selected_fixture_ids];
    const idx = current.indexOf(id);
    if (idx >= 0) {
      current.splice(idx, 1);
    } else {
      current.push(id);
    }
    store.selectFixtures(current);
  } else {
    store.selectFixtures([id]);
  }
}

function selectAll() {
  const ids = fixtures.value.map((f) => f.id);
  store.selectFixtures(ids);
}
</script>

<template>
  <div class="fixture-grid">
    <div class="grid-header">
      <span class="grid-title">灯具</span>
      <button class="select-all-btn" @click="selectAll">全选</button>
    </div>
    <div class="grid-items">
      <button
        v-for="fixture in fixtures"
        :key="fixture.id"
        class="fixture-btn"
        :class="{ selected: selectedIds.has(fixture.id) }"
        @click="toggleFixture(fixture.id, $event)"
        @touchend.prevent="toggleFixture(fixture.id, $event)"
      >
        <span class="fixture-id">{{ fixture.id + 1 }}</span>
        <span
          class="fixture-color-dot"
          :style="{
            backgroundColor: `rgb(${Math.round(fixture.color.r * 255)},${Math.round(fixture.color.g * 255)},${Math.round(fixture.color.b * 255)})`,
          }"
        />
      </button>
    </div>
  </div>
</template>

<style scoped>
.fixture-grid {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.5rem;
}

.grid-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.grid-title {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #888;
}

.select-all-btn {
  background: #0f3460;
  color: #e0e0e0;
  border: none;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.7rem;
  cursor: pointer;
  touch-action: manipulation;
}

.select-all-btn:active {
  background: #e94560;
}

.grid-items {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(48px, 1fr));
  gap: 4px;
}

.fixture-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  min-height: 48px;
  min-width: 48px;
  background: #16213e;
  border: 2px solid transparent;
  border-radius: 6px;
  color: #ccc;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  touch-action: manipulation;
  transition: border-color 0.15s, background 0.15s;
}

.fixture-btn.selected {
  border-color: #e94560;
  background: #1a2a50;
  color: #fff;
}

.fixture-btn:active {
  background: #1a2a50;
}

.fixture-color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.2);
}
</style>
