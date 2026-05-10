<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";

const props = defineProps<{
  panValue: number;
  tiltValue: number;
}>();

const emit = defineEmits<{
  "update:panValue": [value: number];
  "update:tiltValue": [value: number];
}>();

const padRef = ref<HTMLElement | null>(null);
const isDragging = ref(false);

const RANGE = 180;

const crosshairX = computed(() => ((props.panValue + RANGE) / (RANGE * 2)) * 100);
const crosshairY = computed(() => ((RANGE - props.tiltValue) / (RANGE * 2)) * 100);

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

function updateFromPosition(clientX: number, clientY: number) {
  const el = padRef.value;
  if (!el) return;

  const rect = el.getBoundingClientRect();
  const xRatio = clamp((clientX - rect.left) / rect.width, 0, 1);
  const yRatio = clamp((clientY - rect.top) / rect.height, 0, 1);

  const pan = Math.round(xRatio * RANGE * 2 - RANGE);
  // Y axis inverted: top = +180, bottom = -180
  const tilt = Math.round(RANGE - yRatio * RANGE * 2);

  emit("update:panValue", clamp(pan, -RANGE, RANGE));
  emit("update:tiltValue", clamp(tilt, -RANGE, RANGE));
}

function onPointerDown(e: PointerEvent) {
  isDragging.value = true;
  padRef.value?.setPointerCapture(e.pointerId);
  updateFromPosition(e.clientX, e.clientY);
}

function onPointerMove(e: PointerEvent) {
  if (!isDragging.value) return;
  updateFromPosition(e.clientX, e.clientY);
}

function onPointerUp() {
  isDragging.value = false;
}

function onTouchStart(e: TouchEvent) {
  e.preventDefault();
}

onMounted(() => {
  const el = padRef.value;
  if (!el) return;
  el.addEventListener("touchstart", onTouchStart, { passive: false });
});

onBeforeUnmount(() => {
  const el = padRef.value;
  if (!el) return;
  el.removeEventListener("touchstart", onTouchStart);
});
</script>

<template>
  <div class="xy-pad-wrapper">
    <div
      ref="padRef"
      class="xy-pad"
      :class="{ dragging: isDragging }"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
    >
      <svg class="grid-svg" viewBox="0 0 100 100" preserveAspectRatio="none">
        <defs>
          <pattern id="minor-grid" width="10" height="10" patternUnits="userSpaceOnUse">
            <path d="M 10 0 L 0 0 0 10" fill="none" stroke="rgba(255,255,255,0.04)" stroke-width="0.3" />
          </pattern>
          <pattern id="major-grid" width="25" height="25" patternUnits="userSpaceOnUse">
            <rect width="25" height="25" fill="url(#minor-grid)" />
            <path d="M 25 0 L 0 0 0 25" fill="none" stroke="rgba(255,255,255,0.08)" stroke-width="0.3" />
          </pattern>
        </defs>
        <rect width="100" height="100" fill="url(#major-grid)" />

        <line x1="50" y1="0" x2="50" y2="100" stroke="rgba(255,255,255,0.15)" stroke-width="0.4" />
        <line x1="0" y1="50" x2="100" y2="50" stroke="rgba(255,255,255,0.15)" stroke-width="0.4" />
      </svg>

      <div class="corner-label top-left">-180°, +180°</div>
      <div class="corner-label top-right">+180°, +180°</div>
      <div class="corner-label bottom-left">-180°, -180°</div>
      <div class="corner-label bottom-right">+180°, -180°</div>

      <div class="axis-label axis-x">水平 →</div>
      <div class="axis-label axis-y">↑ 垂直</div>

      <div
        class="crosshair"
        :style="{ left: crosshairX + '%', top: crosshairY + '%' }"
      >
        <div class="crosshair-h" />
        <div class="crosshair-v" />
        <div class="crosshair-dot" />
      </div>

      <div class="value-display">
        <span class="value-label">水平</span>
        <span class="value-number">{{ panValue }}°</span>
        <span class="value-sep">|</span>
        <span class="value-label">垂直</span>
        <span class="value-number">{{ tiltValue }}°</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.xy-pad-wrapper {
  width: 100%;
  aspect-ratio: 1 / 1;
  container-type: inline-size;
}

.xy-pad {
  position: relative;
  width: 100%;
  height: 100%;
  background: #1a1a2e;
  border: 1px solid rgba(233, 69, 96, 0.3);
  border-radius: 8px;
  overflow: hidden;
  cursor: crosshair;
  touch-action: none;
  user-select: none;
  -webkit-user-select: none;
}

.xy-pad.dragging {
  border-color: #e94560;
}

.grid-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.corner-label {
  position: absolute;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.2);
  font-family: monospace;
  pointer-events: none;
  padding: 4px 6px;
}

.top-left { top: 0; left: 0; }
.top-right { top: 0; right: 0; }
.bottom-left { bottom: 0; left: 0; }
.bottom-right { bottom: 0; right: 0; }

.axis-label {
  position: absolute;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.25);
  font-family: monospace;
  pointer-events: none;
}

.axis-x {
  bottom: 50%;
  right: 6px;
  transform: translateY(50%);
}

.axis-y {
  top: 6px;
  left: 50%;
  transform: translateX(-50%);
}

.crosshair {
  position: absolute;
  pointer-events: none;
  transform: translate(-50%, -50%);
  z-index: 2;
}

.crosshair-h,
.crosshair-v {
  position: absolute;
  background: #e94560;
}

.crosshair-h {
  width: 24px;
  height: 1px;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  box-shadow: 0 0 6px rgba(233, 69, 96, 0.6);
}

.crosshair-v {
  width: 1px;
  height: 24px;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  box-shadow: 0 0 6px rgba(233, 69, 96, 0.6);
}

.crosshair-dot {
  position: absolute;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #e94560;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  box-shadow: 0 0 10px rgba(233, 69, 96, 0.8), 0 0 20px rgba(233, 69, 96, 0.4);
}

.value-display {
  position: absolute;
  bottom: 8px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  padding: 4px 12px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 13px;
  pointer-events: none;
  z-index: 3;
  white-space: nowrap;
}

.value-label {
  color: rgba(255, 255, 255, 0.5);
}

.value-number {
  color: #e94560;
  font-weight: 600;
  min-width: 3.5em;
  text-align: right;
}

.value-sep {
  color: rgba(255, 255, 255, 0.15);
}
</style>
