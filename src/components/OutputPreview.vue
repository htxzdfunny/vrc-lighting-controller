<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useLightingStore } from "../stores/lighting";
import { renderFrameToCanvas } from "../utils/frameRenderer";
import type { LightingState } from "../types/lighting";

const canvas = ref<HTMLCanvasElement | null>(null);
let animFrame = 0;
let running = true;
const store = useLightingStore();

function renderLoop() {
  if (!running) return;

  const cvs = canvas.value;
  if (cvs) {
    const state: LightingState = {
      fixtures: store.fixtures,
      groups: store.groups,
      selected_fixture_ids: store.selected_fixture_ids,
      cues: store.cues,
      active_cue_id: store.active_cue_id,
      effect: store.effect,
      config: store.config,
      output_status: (store as any).output_status ?? {
        ndi: { state: "disabled", message: null },
        spout: { state: "disabled", message: null },
      },
    };
    renderFrameToCanvas(cvs, state, Date.now(), {
      width: 90,
      height: 720,
      gap: 2,
      blockWidth: 42,
    });
  }

  if (running) {
    animFrame = requestAnimationFrame(renderLoop);
  }
}

onMounted(() => {
  running = true;
  animFrame = requestAnimationFrame(renderLoop);
});

onUnmounted(() => {
  running = false;
  cancelAnimationFrame(animFrame);
});
</script>

<template>
  <div class="output-preview">
    <span class="preview-label">预览</span>
    <div class="preview-container">
      <canvas ref="canvas" width="90" height="720" />
    </div>
  </div>
</template>

<style scoped>
.output-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  gap: 4px;
}

.preview-label {
  font-size: 0.68rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #888;
  text-align: center;
  flex-shrink: 0;
}

.preview-container {
  background: #000;
  border-radius: 6px;
  overflow: hidden;
  flex: 1;
  min-height: 0;
  display: flex;
  justify-content: center;
}

.preview-container canvas {
  display: block;
  height: 100%;
  width: auto;
  image-rendering: pixelated;
}
</style>
