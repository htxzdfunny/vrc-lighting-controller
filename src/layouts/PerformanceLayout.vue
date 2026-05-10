<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from "vue";
import { useLightingStore } from "../stores/lighting";
import { usePerformanceKeyBindings } from "../composables/useKeyboardBindings";
import LightFaderColumn from "../components/perf/LightFaderColumn.vue";
import MasterAngleColumn from "../components/perf/MasterAngleColumn.vue";
import RgbMasterFaders from "../components/perf/RgbMasterFaders.vue";
import PalettePad from "../components/perf/PalettePad.vue";
import LightToggleRow from "../components/perf/LightToggleRow.vue";
import StateSnapshotsBar from "../components/perf/StateSnapshotsBar.vue";
import SettingsModal from "../components/SettingsModal.vue";

defineEmits<{ "switch-layout": [target: string] }>();

const store = useLightingStore();
usePerformanceKeyBindings();

const settingOpen = ref(false);
const rootRef = ref<HTMLElement | null>(null);
const sizeMode = ref<"wide" | "medium" | "narrow" | "xs">("wide");
let resizeObserver: ResizeObserver | null = null;

function pickSizeMode(w: number): "wide" | "medium" | "narrow" | "xs" {
  if (w < 600) return "xs";
  if (w < 900) return "narrow";
  if (w < 1280) return "medium";
  return "wide";
}

const lightCount = computed(() => Math.min(10, store.fixtures.length));

async function toggleOutputWindow() {
  await store.setOutputWindowVisible(!(store.config as any).output_window_visible);
}

const ndiStateText = computed(() => {
  const raw = (store.output_status as any)?.ndi?.state ?? "disabled";
  if (raw === "active") return "运行";
  if (raw === "error") return "异常";
  return "关闭";
});
const spoutStateText = computed(() => {
  const raw = (store.output_status as any)?.spout?.state ?? "disabled";
  if (raw === "active") return "运行";
  if (raw === "error") return "异常";
  return "关闭";
});

onMounted(() => {
  store.initConnection();
  if (rootRef.value) {
    resizeObserver = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width ?? window.innerWidth;
      sizeMode.value = pickSizeMode(w);
    });
    resizeObserver.observe(rootRef.value);
    sizeMode.value = pickSizeMode(rootRef.value.offsetWidth);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
});
</script>

<template>
  <div ref="rootRef" class="perf-layout" :class="`mode-${sizeMode}`">
    <header class="perf-header">
      <h1 class="logo">VRC 灯光控制系统</h1>
      <div class="status-bar">
        <span class="status-indicator" :class="{ online: store.connected }">
          {{ store.connected ? "已连接" : "离线" }}
        </span>
        <span class="status-indicator hide-narrow">WS {{ store.wsClientCount }}</span>
        <span class="status-indicator hide-narrow">NDI {{ ndiStateText }}</span>
        <span class="status-indicator hide-narrow">Spout {{ spoutStateText }}</span>
      </div>
      <div class="header-actions">
        <button
          class="icon-btn"
          :class="{ active: store.editMode }"
          @click="store.toggleEditMode()"
          title="编辑模式：调整颜色和分配预设"
        >
          {{ store.editMode ? "退出编辑" : "编辑模式" }}
        </button>
        <button class="icon-btn" @click="toggleOutputWindow">输出窗</button>
        <button class="icon-btn" @click="$emit('switch-layout', 'console')">
          备用UI
        </button>
        <button class="icon-btn" @click="settingOpen = true">设置</button>
      </div>
    </header>

    <main class="perf-body">
      <section class="panel angles">
        <header class="panel-head">
          <span class="panel-title">灯光角度</span>
          <span class="panel-hint">推子=R通道(Tilt) · 旋钮=G通道(Pan)</span>
        </header>
        <div class="angles-row">
          <MasterAngleColumn class="master-stick" />
          <div class="lights-track">
            <LightFaderColumn
              v-for="i in lightCount"
              :key="i - 1"
              :fixture-id="i - 1"
            />
          </div>
        </div>
      </section>

      <section class="panel palette">
        <header class="panel-head">
          <span class="panel-title">颜色预设</span>
          <span class="panel-hint">
            {{
              store.editMode
                ? "编辑模式：调 RGB 后点格子保存，× 清除"
                : "点击格子或按 QWERT / ASDFG 应用"
            }}
          </span>
        </header>
        <div class="palette-row">
          <RgbMasterFaders class="rgb-side" />
          <PalettePad class="pad-side" />
        </div>
      </section>

      <section class="panel toggles">
        <LightToggleRow />
      </section>

      <section class="panel snapshots-row">
        <StateSnapshotsBar />
      </section>
    </main>

    <SettingsModal v-model:open="settingOpen" />
  </div>
</template>

<style scoped>
.perf-layout {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: #1a1a2e;
  overflow: hidden;
  user-select: none;
  -webkit-user-select: none;
}

/* Header */
.perf-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.4rem 0.8rem;
  background: #16213e;
  border-bottom: 1px solid #0f3460;
  flex-shrink: 0;
  gap: 0.6rem;
  flex-wrap: wrap;
}

.logo {
  font-size: clamp(0.9rem, 1.6vw, 1.1rem);
  font-weight: 700;
  color: #e94560;
  letter-spacing: 0.05em;
  margin: 0;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.status-indicator {
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  background: #0f3460;
  font-size: 0.72rem;
  color: #888;
}

.status-indicator.online {
  background: #2d6a4f;
  color: #b7e4c7;
}

.header-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.icon-btn {
  border: 1px solid #355a8b;
  border-radius: 6px;
  background: #103d6a;
  color: #dfe8ff;
  min-height: 32px;
  padding: 0 10px;
  cursor: pointer;
  touch-action: manipulation;
  font-size: 0.78rem;
}

.icon-btn.active {
  background: #e94560;
  border-color: #ff5e7d;
  color: #fff;
}

/* Body */
.perf-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr) auto auto;
  grid-template-areas:
    "angles palette"
    "toggles toggles"
    "snapshots snapshots";
  gap: 8px;
  padding: 8px;
  overflow: hidden;
}

.panel {
  background: #111a35;
  border: 1px solid #21406f;
  border-radius: 8px;
  padding: 6px 8px;
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
}

.panel.angles {
  grid-area: angles;
}

.panel.palette {
  grid-area: palette;
}

.panel.toggles {
  grid-area: toggles;
  padding: 0;
  background: transparent;
  border: none;
}

.panel.snapshots-row {
  grid-area: snapshots;
  padding: 0;
  background: transparent;
  border: none;
}

.panel-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 0.5rem;
  flex-shrink: 0;
  margin-bottom: 6px;
}

.panel-title {
  font-size: clamp(0.78rem, 1.2vw, 0.9rem);
  font-weight: 600;
  color: #dfe8ff;
}

.panel-hint {
  font-size: clamp(0.62rem, 1vw, 0.72rem);
  color: #6b7a99;
}

/* Angles row */
.angles-row {
  display: flex;
  flex: 1;
  min-height: 0;
  gap: 6px;
  align-items: stretch;
}

.master-stick {
  flex-shrink: 0;
}

.lights-track {
  flex: 1;
  min-width: 0;
  display: flex;
  gap: clamp(2px, 0.5vw, 8px);
  overflow: hidden;
  align-items: stretch;
}

/* Palette row */
.palette-row {
  display: grid;
  grid-template-columns: minmax(120px, 0.6fr) minmax(0, 1fr);
  gap: 10px;
  flex: 1;
  min-height: 0;
}

.rgb-side {
  min-height: 0;
}

.pad-side {
  min-height: 0;
}

/* ==== Responsive overrides ==== */

/* medium: same grid, denser controls */
.perf-layout.mode-medium .lights-track {
  gap: 4px;
}

/* narrow: stack panels vertically */
.perf-layout.mode-narrow .perf-body,
.perf-layout.mode-xs .perf-body {
  grid-template-columns: 1fr;
  grid-template-rows: minmax(180px, 0.9fr) auto auto auto;
  grid-template-areas:
    "angles"
    "palette"
    "toggles"
    "snapshots";
}

.perf-layout.mode-narrow .master-stick,
.perf-layout.mode-xs .master-stick {
  position: sticky;
  left: 0;
  z-index: 1;
  background: #111a35;
}

.perf-layout.mode-narrow .palette-row {
  grid-template-columns: minmax(100px, 0.5fr) minmax(0, 1fr);
}

.perf-layout.mode-xs .palette-row {
  grid-template-columns: 1fr;
  grid-auto-rows: auto;
}

.perf-layout.mode-xs .panel.angles,
.perf-layout.mode-xs .panel.palette {
  min-height: 220px;
}

.perf-layout.mode-xs .status-bar .hide-narrow,
.perf-layout.mode-narrow .status-bar .hide-narrow {
  display: none;
}

@media (pointer: coarse) {
  .icon-btn {
    min-height: 40px;
    padding: 0 12px;
  }
}
</style>
