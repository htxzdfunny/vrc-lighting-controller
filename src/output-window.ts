import { invoke } from "@tauri-apps/api/core";
import type { LightingState } from "./types/lighting";
import { renderFrameToCanvas } from "./utils/frameRenderer";

const canvas = document.getElementById("outputCanvas") as HTMLCanvasElement | null;
let running = true;

async function loop() {
  if (!running || !canvas) return;
  try {
    const state = await invoke<LightingState>("get_state");
    renderFrameToCanvas(canvas, state, undefined, { width: 90, height: 720, gap: 2, blockWidth: 42 });
  } catch {
    // keep loop alive
  }
  requestAnimationFrame(loop);
}

window.addEventListener("beforeunload", () => {
  running = false;
});

requestAnimationFrame(loop);
