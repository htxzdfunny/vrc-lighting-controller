<script setup lang="ts">
import { ref, computed } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    label: string;
    step?: number;
  }>(),
  {
    min: -180,
    max: 180,
    step: 1,
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: number];
}>();

const dragging = ref(false);
const lastY = ref(0);

const angleDeg = computed(() => {
  if (props.max === props.min) return -135;
  const ratio = (props.modelValue - props.min) / (props.max - props.min);
  return -135 + ratio * 270;
});

const indicatorStyle = computed(() => ({
  transform: `rotate(${angleDeg.value}deg)`,
}));

function clamp(v: number, lo: number, hi: number) {
  return Math.min(hi, Math.max(lo, v));
}

function applyDelta(dy: number) {
  const range = props.max - props.min;
  const sensitivity = range / 200;
  const rawDelta = -dy * sensitivity;
  const snapped =
    props.step >= 1
      ? Math.round(rawDelta / props.step) * props.step
      : rawDelta;
  if (snapped === 0) return;
  const next = clamp(props.modelValue + snapped, props.min, props.max);
  emit("update:modelValue", Math.round(next * 1000) / 1000);
}

function onPointerDown(e: PointerEvent) {
  dragging.value = true;
  lastY.value = e.clientY;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  const dy = e.clientY - lastY.value;
  lastY.value = e.clientY;
  applyDelta(dy);
}

function onPointerUp() {
  dragging.value = false;
}

function onWheel(e: WheelEvent) {
  e.preventDefault();
  const direction = e.deltaY > 0 ? 1 : -1;
  applyDelta(direction * 3);
}

function onTouchStart(e: TouchEvent) {
  e.preventDefault();
}
</script>

<template>
  <div class="encoder-knob">
    <span class="encoder-label">{{ label }}</span>
    <div
      class="knob-body"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @wheel.passive="false"
      @wheel="onWheel"
      @touchstart="onTouchStart"
    >
      <div class="knob-indicator" :style="indicatorStyle">
        <div class="knob-line" />
      </div>
    </div>
    <span class="encoder-value">{{ modelValue }}</span>
  </div>
</template>

<style scoped>
.encoder-knob {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  user-select: none;
  -webkit-user-select: none;
  touch-action: none;
}

.encoder-label {
  font-size: 0.7rem;
  color: rgba(255, 255, 255, 0.7);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 80px;
}

.knob-body {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: #16213e;
  position: relative;
  cursor: grab;
  box-shadow:
    0 2px 8px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

.knob-body:active {
  cursor: grabbing;
}

.knob-indicator {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.knob-line {
  position: absolute;
  width: 3px;
  height: 45%;
  top: 5%;
  left: 50%;
  transform: translateX(-50%);
  background: #e94560;
  border-radius: 1.5px;
}

.encoder-value {
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  color: #e94560;
  font-weight: 600;
  min-height: 1.2em;
}
</style>
