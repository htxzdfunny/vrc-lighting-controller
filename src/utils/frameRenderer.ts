import type { LightingState, Color } from "../types/lighting";

export interface RenderOptions {
  width?: number;
  height?: number;
  gap?: number;
  blockWidth?: number;
}

function clamp(v: number, min: number, max: number) {
  return Math.max(min, Math.min(max, v));
}

function encodeAngle(angle: number) {
  return Math.round(clamp((angle + 180) / 360, 0, 1) * 255);
}

// Port of Rust hsl_to_rgb(h, s=1.0, l=0.5) from engine/effect.rs
function hslToRgb(hDeg: number): Color {
  const h = ((hDeg % 360) + 360) % 360;
  const s = 1.0;
  const l = 0.5;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  let r = 0, g = 0, b = 0;
  if (h < 60)       { r = c; g = x; b = 0; }
  else if (h < 120) { r = x; g = c; b = 0; }
  else if (h < 180) { r = 0; g = c; b = x; }
  else if (h < 240) { r = 0; g = x; b = c; }
  else if (h < 300) { r = x; g = 0; b = c; }
  else              { r = c; g = 0; b = x; }
  return { r: r + m, g: g + m, b: b + m };
}

// Port of Rust simple_hash from engine/effect.rs using BigInt for 64-bit wrap
function simpleHash(a: number, b: number): number {
  const mask = BigInt("0xFFFFFFFFFFFFFFFF");
  let h = (BigInt(a) * BigInt("6364136223846793005") + BigInt(b)) & mask;
  h = (h ^ (h >> BigInt(33))) & mask;
  h = (h * BigInt("0xff51afd7ed558ccd")) & mask;
  h = (h ^ (h >> BigInt(33))) & mask;
  return Number(h) / Number(mask);
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

function applyEffectColor(
  color: Color,
  state: LightingState,
  fixtureIndex: number,
  total: number,
  nowSecs: number
): Color {
  const effect = state.effect;
  if (effect.effect_type === "none") return color;

  const speed = effect.speed;
  const phasePerFixture =
    total > 1
      ? effect.phase_offset * fixtureIndex / (total - 1)
      : 0;
  const t = nowSecs * speed + phasePerFixture;
  const intensity = effect.intensity;

  if (effect.effect_type === "color_cycle") {
    const hue = ((t * 360) % 360 + 360) % 360;
    const { r, g, b } = hslToRgb(hue);
    return {
      r: lerp(color.r, r, intensity),
      g: lerp(color.g, g, intensity),
      b: lerp(color.b, b, intensity),
    };
  }

  if (effect.effect_type === "scan") {
    const wave = (Math.sin(t * Math.PI * 2) * 0.5 + 0.5) * intensity;
    return { r: color.r * wave, g: color.g * wave, b: color.b * wave };
  }

  if (effect.effect_type === "pulse") {
    const pulse = Math.abs(Math.sin(t * Math.PI * 2)) * intensity;
    const k = 1 - intensity + pulse;
    return { r: color.r * k, g: color.g * k, b: color.b * k };
  }

  if (effect.effect_type === "wave") {
    const wave = Math.sin(t * Math.PI * 2) * 0.5 + 0.5;
    const k = lerp(1 - intensity, 1, wave);
    return { r: color.r * k, g: color.g * k, b: color.b * k };
  }

  // Random: port of Rust simple_hash(floor(t*7), fixture_index)
  const seed = Math.floor(t * 7);
  const brightness = simpleHash(seed, fixtureIndex) * intensity;
  const k = 1 - intensity + brightness;
  return { r: color.r * k, g: color.g * k, b: color.b * k };
}

// Returns per-fixture block heights that exactly fill `height` with no wasted pixels.
// Mirrors the Rust BlockLayout::for_fixture logic.
export function computeBlockMetrics(
  fixtureCount: number,
  height: number,
  gap = 2,
  blockWidth = 42
) {
  const safeCount = Math.max(1, fixtureCount);
  const usable = Math.max(0, height - safeCount * gap);
  const baseH = Math.max(1, Math.floor(usable / safeCount));
  const extra = usable % safeCount;
  // blockHeights[i] = baseH + (i < extra ? 1 : 0)
  const blockHeights = Array.from({ length: safeCount }, (_, i) =>
    baseH + (i < extra ? 1 : 0)
  );
  return { gap, blockWidth, blockHeight: baseH, blockHeights };
}

export function renderFrameToCanvas(
  canvas: HTMLCanvasElement,
  state: LightingState,
  nowMillis = Date.now(),
  options: RenderOptions = {}
) {
  const width = options.width ?? 90;
  const height = options.height ?? 720;
  const gap = options.gap ?? 2;
  const blockWidth = options.blockWidth ?? 42;

  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const imageData = ctx.createImageData(width, height);
  const data = imageData.data;
  data.fill(0);

  const fixtureCount = Math.min(
    Math.max(1, state.config.fixture_count),
    state.fixtures.length
  );
  const { blockHeights } = computeBlockMetrics(fixtureCount, height, gap, blockWidth);
  const nowSecs = nowMillis / 1000;

  let yOffset = gap;
  for (let i = 0; i < fixtureCount; i++) {
    const fixture = state.fixtures[i];
    const blockHeight = blockHeights[i];
    const y = yOffset;
    yOffset += blockHeight + gap;

    const aR = encodeAngle(fixture.tilt);
    const aG = encodeAngle(fixture.pan);
    fillRect(data, width, gap, y, blockWidth, blockHeight, aR, aG, 0, 255);

    const effectiveColor = applyEffectColor(fixture.color, state, i, fixtureCount, nowSecs);
    const strobeVisible =
      !fixture.strobe_on ||
      fixture.strobe_speed <= 0 ||
      Math.sin(nowSecs * fixture.strobe_speed * Math.PI * 2) > 0;
    const isOn = fixture.is_on !== false;
    const k = clamp(fixture.dimmer, 0, 1);
    const visible = isOn && strobeVisible;
    const bR = visible ? Math.round(clamp(effectiveColor.r * k, 0, 1) * 255) : 0;
    const bG = visible ? Math.round(clamp(effectiveColor.g * k, 0, 1) * 255) : 0;
    const bB = visible ? Math.round(clamp(effectiveColor.b * k, 0, 1) * 255) : 0;
    fillRect(
      data,
      width,
      gap + blockWidth + gap,
      y,
      blockWidth,
      blockHeight,
      bR,
      bG,
      bB,
      255
    );
  }

  ctx.putImageData(imageData, 0, 0);
}

function fillRect(
  data: Uint8ClampedArray,
  canvasWidth: number,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
  g: number,
  b: number,
  a: number
) {
  const x0 = Math.max(0, x);
  const y0 = Math.max(0, y);
  const x1 = x0 + Math.max(0, w);
  const y1 = y0 + Math.max(0, h);
  for (let py = y0; py < y1; py++) {
    for (let px = x0; px < x1; px++) {
      const idx = (py * canvasWidth + px) * 4;
      data[idx] = r;
      data[idx + 1] = g;
      data[idx + 2] = b;
      data[idx + 3] = a;
    }
  }
}
