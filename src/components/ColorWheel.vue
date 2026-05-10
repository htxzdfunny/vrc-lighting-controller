<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from "vue";

const props = defineProps<{
  r: number;
  g: number;
  b: number;
}>();

const emit = defineEmits<{
  "update:r": [value: number];
  "update:g": [value: number];
  "update:b": [value: number];
}>();

function rgbToHsv(r: number, g: number, b: number): [number, number, number] {
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;
  let h = 0;
  const s = max === 0 ? 0 : d / max;
  const v = max;
  if (d !== 0) {
    if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
    else if (max === g) h = ((b - r) / d + 2) / 6;
    else h = ((r - g) / d + 4) / 6;
  }
  return [h, s, v];
}

function hsvToRgb(h: number, s: number, v: number): [number, number, number] {
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);
  let r: number, g: number, b: number;
  switch (i % 6) {
    case 0: r = v; g = t; b = p; break;
    case 1: r = q; g = v; b = p; break;
    case 2: r = p; g = v; b = t; break;
    case 3: r = p; g = q; b = v; break;
    case 4: r = t; g = p; b = v; break;
    default: r = v; g = p; b = q; break;
  }
  return [r, g, b];
}

const canvasRef = ref<HTMLCanvasElement | null>(null);
const containerRef = ref<HTMLDivElement | null>(null);
const canvasSize = ref(300);

const hsv = ref<[number, number, number]>(rgbToHsv(props.r, props.g, props.b));

watch(
  () => [props.r, props.g, props.b] as const,
  ([r, g, b]) => {
    const [cr, cg, cb] = hsvToRgb(...hsv.value);
    // Avoid feedback loop: only sync if props diverge meaningfully from internal state
    if (
      Math.abs(cr - r) > 0.005 ||
      Math.abs(cg - g) > 0.005 ||
      Math.abs(cb - b) > 0.005
    ) {
      hsv.value = rgbToHsv(r, g, b);
    }
  }
);

function emitRgb() {
  const [r, g, b] = hsvToRgb(...hsv.value);
  emit("update:r", r);
  emit("update:g", g);
  emit("update:b", b);
}

const PADDING = 4;
const RING_RATIO = 0.13;
const GAP = 4;

function geo() {
  const s = canvasSize.value;
  const cx = s / 2;
  const cy = s / 2;
  const outerR = s / 2 - PADDING;
  const ringW = Math.max(18, outerR * RING_RATIO);
  const innerR = outerR - ringW;
  const sqR = innerR - GAP;
  const sqSide = sqR * Math.SQRT2;
  const sqHalf = sqSide / 2;
  return { cx, cy, outerR, ringW, innerR, sqR, sqSide, sqHalf };
}

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const s = canvasSize.value;
  canvas.width = s * dpr;
  canvas.height = s * dpr;
  canvas.style.width = `${s}px`;
  canvas.style.height = `${s}px`;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

  const { cx, cy, outerR, ringW, sqSide, sqHalf } = geo();
  const [h, sat, val] = hsv.value;

  ctx.clearRect(0, 0, s, s);

  const steps = 360;
  for (let i = 0; i < steps; i++) {
    const a0 = (i / steps) * Math.PI * 2 - Math.PI / 2;
    const a1 = ((i + 1.5) / steps) * Math.PI * 2 - Math.PI / 2;
    const [hr, hg, hb] = hsvToRgb(i / steps, 1, 1);
    ctx.beginPath();
    ctx.arc(cx, cy, outerR - ringW / 2, a0, a1);
    ctx.strokeStyle = `rgb(${hr * 255},${hg * 255},${hb * 255})`;
    ctx.lineWidth = ringW;
    ctx.stroke();
  }

  const sqX = cx - sqHalf;
  const sqY = cy - sqHalf;
  const [pr, pg, pb] = hsvToRgb(h, 1, 1);

  const gradH = ctx.createLinearGradient(sqX, sqY, sqX + sqSide, sqY);
  gradH.addColorStop(0, "#fff");
  gradH.addColorStop(1, `rgb(${pr * 255},${pg * 255},${pb * 255})`);
  ctx.fillStyle = gradH;
  ctx.fillRect(sqX, sqY, sqSide, sqSide);

  const gradV = ctx.createLinearGradient(sqX, sqY, sqX, sqY + sqSide);
  gradV.addColorStop(0, "rgba(0,0,0,0)");
  gradV.addColorStop(1, "#000");
  ctx.fillStyle = gradV;
  ctx.fillRect(sqX, sqY, sqSide, sqSide);

  ctx.strokeStyle = "rgba(255,255,255,0.1)";
  ctx.lineWidth = 1;
  ctx.strokeRect(sqX, sqY, sqSide, sqSide);

  const hueAngle = h * Math.PI * 2 - Math.PI / 2;
  const hueIx = cx + Math.cos(hueAngle) * (outerR - ringW / 2);
  const hueIy = cy + Math.sin(hueAngle) * (outerR - ringW / 2);
  ctx.beginPath();
  ctx.arc(hueIx, hueIy, ringW / 2 + 2, 0, Math.PI * 2);
  ctx.strokeStyle = "#fff";
  ctx.lineWidth = 2.5;
  ctx.stroke();
  ctx.beginPath();
  ctx.arc(hueIx, hueIy, ringW / 2, 0, Math.PI * 2);
  ctx.strokeStyle = "rgba(0,0,0,0.3)";
  ctx.lineWidth = 1;
  ctx.stroke();

  const svIx = sqX + sat * sqSide;
  const svIy = sqY + (1 - val) * sqSide;
  const outerStroke = val > 0.5 && sat < 0.5 ? "#000" : "#fff";
  const innerStroke = val > 0.5 && sat < 0.5 ? "#fff" : "#000";
  ctx.beginPath();
  ctx.arc(svIx, svIy, 8, 0, Math.PI * 2);
  ctx.strokeStyle = outerStroke;
  ctx.lineWidth = 2.5;
  ctx.stroke();
  ctx.beginPath();
  ctx.arc(svIx, svIy, 6, 0, Math.PI * 2);
  ctx.strokeStyle = innerStroke;
  ctx.lineWidth = 1;
  ctx.stroke();

  const prevR = Math.max(10, Math.min(18, sqSide * 0.08));
  const [cr, cg, cb] = hsvToRgb(h, sat, val);
  ctx.beginPath();
  ctx.arc(cx, cy, prevR, 0, Math.PI * 2);
  ctx.fillStyle = `rgb(${cr * 255},${cg * 255},${cb * 255})`;
  ctx.fill();
  ctx.strokeStyle = "rgba(255,255,255,0.5)";
  ctx.lineWidth = 1.5;
  ctx.stroke();
}

watch(hsv, draw, { deep: true });

type DragTarget = "hue" | "sv" | null;
let dragTarget: DragTarget = null;

function pointerPos(e: PointerEvent) {
  const rect = canvasRef.value!.getBoundingClientRect();
  return { x: e.clientX - rect.left, y: e.clientY - rect.top };
}

function hitTest(x: number, y: number): DragTarget {
  const { cx, cy, outerR, innerR, sqHalf } = geo();
  const dx = x - cx;
  const dy = y - cy;
  const dist = Math.sqrt(dx * dx + dy * dy);
  // Generous touch tolerance on the ring
  const tolerance = 8;
  if (dist >= innerR - tolerance && dist <= outerR + tolerance) return "hue";
  if (Math.abs(dx) <= sqHalf && Math.abs(dy) <= sqHalf) return "sv";
  return null;
}

function updateHue(x: number, y: number) {
  const { cx, cy } = geo();
  let angle = Math.atan2(y - cy, x - cx) + Math.PI / 2;
  if (angle < 0) angle += Math.PI * 2;
  hsv.value = [angle / (Math.PI * 2), hsv.value[1], hsv.value[2]];
  emitRgb();
}

function updateSv(x: number, y: number) {
  const { cx, cy, sqHalf, sqSide } = geo();
  const sqX = cx - sqHalf;
  const sqY = cy - sqHalf;
  const s = Math.max(0, Math.min(1, (x - sqX) / sqSide));
  const v = Math.max(0, Math.min(1, 1 - (y - sqY) / sqSide));
  hsv.value = [hsv.value[0], s, v];
  emitRgb();
}

function onPointerDown(e: PointerEvent) {
  const { x, y } = pointerPos(e);
  dragTarget = hitTest(x, y);
  if (!dragTarget) return;
  canvasRef.value?.setPointerCapture(e.pointerId);
  if (dragTarget === "hue") updateHue(x, y);
  else updateSv(x, y);
}

function onPointerMove(e: PointerEvent) {
  if (!dragTarget) return;
  const { x, y } = pointerPos(e);
  if (dragTarget === "hue") updateHue(x, y);
  else updateSv(x, y);
}

function onPointerUp() {
  dragTarget = null;
}

function onTouchStart(e: TouchEvent) {
  e.preventDefault();
}

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  const container = containerRef.value;
  if (!container) return;

  resizeObserver = new ResizeObserver((entries) => {
    const entry = entries[0];
    if (!entry) return;
    const s = Math.max(
      200,
      Math.floor(
        Math.min(entry.contentRect.width, entry.contentRect.height)
      )
    );
    canvasSize.value = s;
    requestAnimationFrame(draw);
  });
  resizeObserver.observe(container);

  const el = canvasRef.value;
  if (el) el.addEventListener("touchstart", onTouchStart, { passive: false });

  requestAnimationFrame(draw);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  const el = canvasRef.value;
  if (el) el.removeEventListener("touchstart", onTouchStart);
});
</script>

<template>
  <div ref="containerRef" class="color-wheel-container">
    <canvas
      ref="canvasRef"
      class="color-wheel-canvas"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
    />
  </div>
</template>

<style scoped>
.color-wheel-container {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  min-width: 200px;
  min-height: 200px;
  background: #1a1a2e;
}

.color-wheel-canvas {
  touch-action: none;
  cursor: crosshair;
  user-select: none;
  -webkit-user-select: none;
}
</style>
