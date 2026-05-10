<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../stores/lighting";
import FaderStrip from "./FaderStrip.vue";

const store = useLightingStore();

const masterDimmer = computed({
  get: () => {
    const f = store.currentFixture;
    return f ? f.dimmer : 1;
  },
  set: (val: number) => {
    store.setSelectedDimmer(val);
  },
});

const fixtureCount = computed(() => store.config.fixture_count);

const faderFixtures = computed(() =>
  store.fixtures.slice(0, Math.min(fixtureCount.value, 8))
);

function setFixtureDimmer(fixtureId: number, val: number) {
  store.setFixtureDimmer(fixtureId, val);
}
</script>

<template>
  <div class="bottom-faders">
    <div class="faders-scroll">
      <div
        v-for="fixture in faderFixtures"
        :key="fixture.id"
        class="fader-item"
      >
        <FaderStrip
          :model-value="fixture.dimmer"
          :label="String(fixture.id + 1)"
          @update:model-value="(v: number) => setFixtureDimmer(fixture.id, v)"
        />
      </div>
      <div class="fader-item master">
        <FaderStrip
          :model-value="masterDimmer"
          label="主控"
          @update:model-value="(v: number) => masterDimmer = v"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.bottom-faders {
  background: #16213e;
  border-top: 1px solid #0f3460;
  padding: 0.25rem 0.5rem;
  flex-shrink: 0;
  overflow: hidden;
}

.faders-scroll {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  height: 120px;
  align-items: stretch;
  -webkit-overflow-scrolling: touch;
}

.fader-item {
  flex: 0 0 56px;
  min-width: 48px;
}

.fader-item.master {
  border-left: 2px solid #e94560;
  padding-left: 4px;
}
</style>
