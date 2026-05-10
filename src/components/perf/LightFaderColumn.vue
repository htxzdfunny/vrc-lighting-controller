<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../../stores/lighting";
import VerticalFader from "./VerticalFader.vue";
import RotaryKnob from "./RotaryKnob.vue";

const props = defineProps<{ fixtureId: number }>();

const store = useLightingStore();

const fixture = computed(() => store.fixtures.find((f) => f.id === props.fixtureId) ?? null);

// fader value [0,1] ↔ tilt -180..180
const faderValue = computed({
  get: () => {
    const t = fixture.value?.tilt ?? -180;
    return Math.max(0, Math.min(1, (t + 180) / 360));
  },
  set: (v: number) => {
    if (!fixture.value) return;
    const tilt = v * 360 - 180;
    store.setFixturePosition(props.fixtureId, fixture.value.pan, tilt);
  },
});

// knob value [0,1] ↔ pan -180..180
const knobValue = computed({
  get: () => {
    const p = fixture.value?.pan ?? -180;
    return Math.max(0, Math.min(1, (p + 180) / 360));
  },
  set: (v: number) => {
    if (!fixture.value) return;
    const pan = v * 360 - 180;
    store.setFixturePosition(props.fixtureId, pan, fixture.value.tilt);
  },
});

const syncFader = computed(() => fixture.value?.sync_master_fader ?? true);
const syncKnob = computed(() => fixture.value?.sync_master_knob ?? true);

function toggleSync() {
  if (!fixture.value) return;
  const nextFader = !syncFader.value || !syncKnob.value;
  store.setFixtureSync(props.fixtureId, nextFader, nextFader);
}

const isSyncing = computed(() => syncFader.value && syncKnob.value);
const labelNumber = computed(() => props.fixtureId + 1);
</script>

<template>
  <div class="light-col">
    <div class="fader-area">
      <VerticalFader v-model="faderValue" />
    </div>
    <button
      class="sync-btn"
      :class="{ active: isSyncing }"
      :title="isSyncing ? '跟随总控' : '独立'"
      @click="toggleSync"
    >
      sync
    </button>
    <div class="knob-area">
      <RotaryKnob v-model="knobValue" />
    </div>
    <span class="num">{{ labelNumber === 10 ? 0 : labelNumber }}</span>
  </div>
</template>

<style scoped>
.light-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex: 1 1 0;
  min-width: 0;
  height: 100%;
  min-height: 0;
}

.fader-area {
  flex: 1;
  min-height: 0;
  width: 100%;
  display: flex;
  justify-content: center;
}

.sync-btn {
  font-size: 0.62rem;
  letter-spacing: 0.05em;
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid #2a3e63;
  background: #0d1730;
  color: #6b7a99;
  cursor: pointer;
  flex-shrink: 0;
}

.sync-btn.active {
  background: #21406f;
  color: #b6cdf2;
  border-color: #4575aa;
}

.knob-area {
  flex-shrink: 0;
}

.num {
  font-size: clamp(0.7rem, 1.1vw, 0.85rem);
  color: #c8d3eb;
  font-weight: 600;
  flex-shrink: 0;
}
</style>
