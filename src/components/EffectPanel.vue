<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../stores/lighting";
import type { EffectType } from "../stores/lighting";

const store = useLightingStore();

const effects: { type: EffectType; label: string }[] = [
  { type: "color_cycle", label: "颜色循环" },
  { type: "scan", label: "扫描" },
  { type: "pulse", label: "脉冲" },
  { type: "wave", label: "波浪" },
  { type: "random", label: "随机" },
];

const currentEffect = computed(() => store.effect.effect_type);

function toggleEffect(type: EffectType) {
  if (currentEffect.value === type) {
    store.clearEffect();
  } else {
    store.setEffect({
      ...store.effect,
      effect_type: type,
    });
  }
}

function updateSpeed(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  store.setEffect({ ...store.effect, speed: val });
}

function updateIntensity(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  store.setEffect({ ...store.effect, intensity: val });
}

function updatePhaseOffset(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  store.setEffect({ ...store.effect, phase_offset: val });
}

</script>

<template>
  <div class="effect-panel">
    <div class="effect-grid">
      <button
        v-for="fx in effects"
        :key="fx.type"
        class="effect-btn"
        :class="{ active: currentEffect === fx.type }"
        @click="toggleEffect(fx.type)"
        @touchend.prevent="toggleEffect(fx.type)"
      >
        <span class="effect-label">{{ fx.label }}</span>
      </button>
    </div>

    <div class="params">
      <div class="param-row">
        <label>速度</label>
        <input
          type="range"
          min="0.1"
          max="10"
          step="0.1"
          :value="store.effect.speed"
          @input="updateSpeed"
        />
        <span class="param-value">{{ store.effect.speed.toFixed(1) }}</span>
      </div>
      <div class="param-row">
        <label>强度</label>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          :value="store.effect.intensity"
          @input="updateIntensity"
        />
        <span class="param-value">{{ (store.effect.intensity * 100).toFixed(0) }}%</span>
      </div>
      <div class="param-row">
        <label>相位</label>
        <input
          type="range"
          min="0"
          max="6.28"
          step="0.1"
          :value="store.effect.phase_offset"
          @input="updatePhaseOffset"
        />
        <span class="param-value">{{ store.effect.phase_offset.toFixed(1) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.effect-panel {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem;
}

.effect-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 8px;
}

.effect-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  min-height: 64px;
  background: #16213e;
  border: 2px solid transparent;
  border-radius: 8px;
  color: #ccc;
  cursor: pointer;
  touch-action: manipulation;
  transition: all 0.15s;
}

.effect-btn.active {
  border-color: #e94560;
  background: #1a2a50;
  color: #fff;
}

.effect-btn:active {
  transform: scale(0.95);
}

.effect-label {
  font-size: 0.75rem;
}

.params {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.param-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.param-row label {
  width: 70px;
  font-size: 0.8rem;
  color: #888;
  flex-shrink: 0;
}

.param-row input[type="range"] {
  flex: 1;
  height: 32px;
  accent-color: #e94560;
  cursor: pointer;
}

.param-value {
  width: 50px;
  text-align: right;
  font-size: 0.8rem;
  font-variant-numeric: tabular-nums;
  color: #aaa;
}
</style>
