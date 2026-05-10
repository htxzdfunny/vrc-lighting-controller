<script setup lang="ts">
import { ref, computed } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: number;
    label?: string;
    gradient?: string;
    accent?: string;
    disabled?: boolean;
  }>(),
  {
    label: "",
    gradient: "linear-gradient(to top, #000 0%, #e94560 100%)",
    accent: "#fff",
    disabled: false,
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: number];
}>();

const trackEl = ref<HTMLElement | null>(null);
let dragging = false;

function pointerToValue(e: PointerEvent, rect: DOMRect): number {
  const y = e.clientY - rect.top;
  let v = 1 - y / rect.height;
  if (!Number.isFinite(v)) v = 0;
  return Math.max(0, Math.min(1, v));
}

function onPointerDown(e: PointerEvent) {
  if (props.disabled || !trackEl.value) return;
  e.preventDefault();
  dragging = true;
  trackEl.value.setPointerCapture(e.pointerId);
  const rect = trackEl.value.getBoundingClientRect();
  emit("update:modelValue", pointerToValue(e, rect));
}

function onPointerMove(e: PointerEvent) {
  if (!dragging || !trackEl.value) return;
  const rect = trackEl.value.getBoundingClientRect();
  emit("update:modelValue", pointerToValue(e, rect));
}

function onPointerUp(e: PointerEvent) {
  if (!dragging) return;
  dragging = false;
  trackEl.value?.releasePointerCapture(e.pointerId);
}

function onWheel(e: WheelEvent) {
  if (props.disabled) return;
  e.preventDefault();
  const step = e.shiftKey ? 0.002 : 0.02;
  let next = props.modelValue + (e.deltaY < 0 ? step : -step);
  next = Math.max(0, Math.min(1, next));
  emit("update:modelValue", next);
}

function onDoubleClick() {
  if (props.disabled) return;
  emit("update:modelValue", 0);
}

const handleY = computed(() => {
  const v = Math.max(0, Math.min(1, props.modelValue));
  return `${(1 - v) * 100}%`;
});
</script>

<template>
  <div class="fader-wrap">
    <div
      ref="trackEl"
      class="fader-track"
      :class="{ disabled }"
      :style="{ background: gradient }"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @wheel="onWheel"
      @dblclick="onDoubleClick"
    >
      <div
        class="handle"
        :style="{ top: handleY, borderColor: accent }"
      />
    </div>
    <span v-if="label" class="fader-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.fader-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  user-select: none;
  -webkit-user-select: none;
  height: 100%;
  min-height: 0;
}

.fader-track {
  position: relative;
  width: clamp(14px, 1.5vw, 22px);
  flex: 1;
  min-height: 80px;
  border-radius: 4px;
  border: 1px solid #2a3e63;
  cursor: ns-resize;
  touch-action: none;
  overflow: visible;
}

.fader-track.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.handle {
  position: absolute;
  left: -8px;
  right: -8px;
  height: 12px;
  margin-top: -6px;
  background: #d6dff0;
  border: 2px solid #fff;
  border-radius: 4px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
  pointer-events: none;
}

.fader-label {
  font-size: clamp(0.65rem, 1.1vw, 0.78rem);
  color: #b8c4dc;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}

@media (pointer: coarse) {
  .fader-track {
    width: clamp(20px, 3vw, 28px);
  }
  .handle {
    left: -10px;
    right: -10px;
    height: 16px;
    margin-top: -8px;
  }
}
</style>
