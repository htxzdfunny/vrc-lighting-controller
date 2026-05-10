<script setup lang="ts">
import { ref, computed } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: number;
    label?: string;
    size?: number;
    accent?: string;
    disabled?: boolean;
  }>(),
  {
    label: "",
    size: 0,
    accent: "#e94560",
    disabled: false,
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: number];
}>();

const knobEl = ref<HTMLElement | null>(null);
let dragging = false;
let dragOffset = 0;

function pointerToValue(e: PointerEvent, rect: DOMRect): number {
  const cx = rect.left + rect.width / 2;
  const cy = rect.top + rect.height / 2;
  const dx = e.clientX - cx;
  const dy = e.clientY - cy;
  const theta = Math.atan2(dy, dx);
  let v = (Math.PI / 2 - theta) / (Math.PI * 2);
  v = ((v % 1) + 1) % 1;
  return v;
}

function clamp01Wrap(v: number): number {
  return ((v % 1) + 1) % 1;
}

function onPointerDown(e: PointerEvent) {
  if (props.disabled || !knobEl.value) return;
  e.preventDefault();
  dragging = true;
  knobEl.value.setPointerCapture(e.pointerId);
  const rect = knobEl.value.getBoundingClientRect();
  const pointerV = pointerToValue(e, rect);
  dragOffset = props.modelValue - pointerV;
}

function onPointerMove(e: PointerEvent) {
  if (!dragging || !knobEl.value) return;
  const rect = knobEl.value.getBoundingClientRect();
  const pointerV = pointerToValue(e, rect);
  const next = clamp01Wrap(pointerV + dragOffset);
  emit("update:modelValue", next);
}

function onPointerUp(e: PointerEvent) {
  if (!dragging) return;
  dragging = false;
  knobEl.value?.releasePointerCapture(e.pointerId);
}

function onWheel(e: WheelEvent) {
  if (props.disabled) return;
  e.preventDefault();
  const step = e.shiftKey ? 0.002 : 0.01;
  let next = props.modelValue + (e.deltaY < 0 ? step : -step);
  next = clamp01Wrap(next);
  emit("update:modelValue", next);
}

function onDoubleClick() {
  if (props.disabled) return;
  emit("update:modelValue", 0.5);
}

const needleAngle = computed(() => 180 - props.modelValue * 360);
const sizeStyle = computed(() => {
  if (props.size > 0) {
    return {
      width: `${props.size}px`,
      height: `${props.size}px`,
    };
  }
  return {};
});
</script>

<template>
  <div class="knob-wrap">
    <div
      ref="knobEl"
      class="knob"
      :class="{ disabled }"
      :style="sizeStyle"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @wheel="onWheel"
      @dblclick="onDoubleClick"
    >
      <div class="dial" />
      <div
        class="needle"
        :style="{
          transform: `rotate(${needleAngle}deg)`,
          background: accent,
        }"
      />
      <div class="hub" />
    </div>
    <span v-if="label" class="knob-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.knob-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  user-select: none;
  -webkit-user-select: none;
}

.knob {
  width: clamp(36px, 5.2vw, 56px);
  height: clamp(36px, 5.2vw, 56px);
  position: relative;
  border-radius: 50%;
  background: radial-gradient(circle at 50% 35%, #2a3a5a, #0d1730 70%);
  border: 1px solid #2c466c;
  box-shadow: inset 0 -2px 4px rgba(0, 0, 0, 0.6),
    inset 0 1px 1px rgba(255, 255, 255, 0.05);
  cursor: grab;
  touch-action: none;
}

.knob:active {
  cursor: grabbing;
}

.knob.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.dial {
  position: absolute;
  inset: 4px;
  border-radius: 50%;
  background: #1a2440;
  border: 1px solid #2a3e63;
}

.needle {
  position: absolute;
  width: 3px;
  left: 50%;
  top: 50%;
  height: 45%;
  transform-origin: 50% 0;
  margin-left: -1.5px;
  background: #e94560;
  border-radius: 1.5px;
  pointer-events: none;
}

.hub {
  position: absolute;
  width: 22%;
  height: 22%;
  left: 39%;
  top: 39%;
  border-radius: 50%;
  background: #0a1228;
  border: 1px solid #2a3e63;
  pointer-events: none;
}

.knob-label {
  font-size: clamp(0.65rem, 1.1vw, 0.78rem);
  color: #b8c4dc;
  letter-spacing: 0.04em;
}

@media (pointer: coarse) {
  .knob {
    width: clamp(48px, 8vw, 64px);
    height: clamp(48px, 8vw, 64px);
  }
}
</style>
