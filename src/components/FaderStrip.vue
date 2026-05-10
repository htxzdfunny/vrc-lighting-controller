<script setup lang="ts">
import { ref, computed } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: number;
    label: string;
    min?: number;
    max?: number;
    showValue?: boolean;
  }>(),
  {
    min: 0,
    max: 1,
    showValue: true,
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: number];
}>();

const trackRef = ref<HTMLElement | null>(null);
const dragging = ref(false);

const ratio = computed(() => {
  if (props.max === props.min) return 0;
  return (props.modelValue - props.min) / (props.max - props.min);
});

const displayValue = computed(() => {
  if (props.max <= 1 && props.min >= 0) {
    return Math.round(ratio.value * 100) + "%";
  }
  return props.modelValue.toFixed(1);
});

function clamp(v: number, lo: number, hi: number) {
  return Math.min(hi, Math.max(lo, v));
}

function valueFromY(clientY: number) {
  const el = trackRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const pct = 1 - (clientY - rect.top) / rect.height;
  const val = props.min + clamp(pct, 0, 1) * (props.max - props.min);
  emit("update:modelValue", Math.round(val * 1000) / 1000);
}

function onPointerDown(e: PointerEvent) {
  dragging.value = true;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  valueFromY(e.clientY);
}

function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  valueFromY(e.clientY);
}

function onPointerUp() {
  dragging.value = false;
}

function onTouchStart(e: TouchEvent) {
  e.preventDefault();
}
</script>

<template>
  <div class="fader-strip">
    <span v-if="showValue" class="fader-value">{{ displayValue }}</span>
    <div
      ref="trackRef"
      class="fader-track"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @touchstart="onTouchStart"
    >
      <div class="fader-fill" :style="{ height: ratio * 100 + '%' }" />
      <div class="fader-handle" :style="{ bottom: ratio * 100 + '%' }" />
    </div>
    <span class="fader-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.fader-strip {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  min-height: 150px;
  height: 100%;
  user-select: none;
  -webkit-user-select: none;
  touch-action: none;
}

.fader-value {
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  color: #e94560;
  font-weight: 600;
  flex-shrink: 0;
  min-height: 1.2em;
}

.fader-track {
  position: relative;
  width: 64px;
  flex: 1;
  min-height: 0;
  background: #16213e;
  border-radius: 6px;
  overflow: hidden;
  cursor: pointer;
}

.fader-fill {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  background: #e94560;
  border-radius: 0 0 6px 6px;
  pointer-events: none;
  transition: height 0.04s linear;
}

.fader-handle {
  position: absolute;
  left: 4px;
  right: 4px;
  height: 6px;
  background: #fff;
  border-radius: 3px;
  transform: translateY(50%);
  pointer-events: none;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.5);
  transition: bottom 0.04s linear;
}

.fader-label {
  font-size: 0.7rem;
  color: rgba(255, 255, 255, 0.7);
  text-align: center;
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 72px;
}
</style>
