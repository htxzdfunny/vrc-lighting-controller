<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from "vue";
import { useLightingStore } from "../stores/lighting";
import { api } from "../api/adapter";
import XYPad from "../components/XYPad.vue";
import ColorWheel from "../components/ColorWheel.vue";
import FaderStrip from "../components/FaderStrip.vue";
import EffectPanel from "../components/EffectPanel.vue";
import CueList from "../components/CueList.vue";
import OutputPreview from "../components/OutputPreview.vue";
import StatusStrip from "../components/StatusStrip.vue";
import SettingsModal from "../components/SettingsModal.vue";

defineEmits<{ "switch-layout": [target: string] }>();

const store = useLightingStore();
const settingOpen = ref(false);
const narrowMode = ref(false);
const narrowPreviewOpen = ref(false);
const rootRef = ref<HTMLElement | null>(null);
const appVersion = ref("");
let resizeObserver: ResizeObserver | null = null;

const fixture = computed(() => store.currentFixture);
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

function onPanUpdate(val: number) {
  if (!fixture.value) return;
  store.setSelectedPosition(val, fixture.value.tilt);
}

function onTiltUpdate(val: number) {
  if (!fixture.value) return;
  store.setSelectedPosition(fixture.value.pan, val);
}

function onColorR(val: number) {
  if (!fixture.value) return;
  store.setSelectedColor(val, fixture.value.color.g, fixture.value.color.b);
}

function onColorG(val: number) {
  if (!fixture.value) return;
  store.setSelectedColor(fixture.value.color.r, val, fixture.value.color.b);
}

function onColorB(val: number) {
  if (!fixture.value) return;
  store.setSelectedColor(fixture.value.color.r, fixture.value.color.g, val);
}

function onDimmerUpdate(val: number) {
  store.setSelectedDimmer(val);
}

function onStrobeSpeedUpdate(val: number) {
  if (!fixture.value) return;
  for (const id of store.selected_fixture_ids) {
    store.setFixtureStrobe(id, val > 0, val);
  }
}

async function toggleOutputWindow() {
  await store.setOutputWindowVisible(!(store.config as any).output_window_visible);
}

onMounted(() => {
  store.initConnection();
  api.getAppVersion().then(v => { appVersion.value = v; });
  if (rootRef.value) {
    resizeObserver = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width ?? window.innerWidth;
      narrowMode.value = w < 900;
    });
    resizeObserver.observe(rootRef.value);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
});
</script>

<template>
  <div ref="rootRef" class="console-layout" :class="{ narrow: narrowMode }">
    <header class="console-header">
      <h1 class="logo">VRC Light Controller <span v-if="appVersion" class="version-tag">{{ appVersion }}</span></h1>
      <div class="status-bar">
        <span class="status-indicator" :class="{ online: store.connected }">
          {{ store.connected ? "已连接" : "离线" }}
        </span>
        <span class="status-indicator hide-narrow">WS {{ store.wsClientCount }}</span>
        <span class="status-indicator hide-narrow">NDI {{ ndiStateText }} {{ store.ndiClientCount }}</span>
        <span class="status-indicator hide-narrow">Spout2 {{ spoutStateText }}</span>
        <button class="icon-btn" @click="toggleOutputWindow">输出窗</button>
        <button v-if="narrowMode" class="icon-btn" @click="narrowPreviewOpen = !narrowPreviewOpen">
          {{ narrowPreviewOpen ? "预览关" : "预览开" }}
        </button>
        <button class="icon-btn" @click="$emit('switch-layout', 'perf')">演出</button>
        <button class="icon-btn" @click="settingOpen = true">设置</button>
      </div>
    </header>

    <StatusStrip />

    <!-- Desktop: 4-col × 2-row grid -->
    <div v-if="!narrowMode" class="console-body">
      <div class="zone-xy">
        <XYPad
          :pan-value="fixture?.pan ?? 0"
          :tilt-value="fixture?.tilt ?? 0"
          @update:pan-value="onPanUpdate"
          @update:tilt-value="onTiltUpdate"
        />
      </div>
      <div class="zone-wheel">
        <ColorWheel
          :r="fixture?.color.r ?? 1"
          :g="fixture?.color.g ?? 1"
          :b="fixture?.color.b ?? 1"
          @update:r="onColorR"
          @update:g="onColorG"
          @update:b="onColorB"
        />
      </div>
      <div class="zone-faders">
        <FaderStrip
          :model-value="fixture?.pan ?? 0"
          :min="-180"
          :max="180"
          label="水平"
          @update:model-value="onPanUpdate"
        />
        <FaderStrip
          :model-value="fixture?.tilt ?? 0"
          :min="-180"
          :max="180"
          label="垂直"
          @update:model-value="onTiltUpdate"
        />
        <FaderStrip
          :model-value="fixture?.dimmer ?? 1"
          :min="0"
          :max="1"
          label="亮度"
          @update:model-value="onDimmerUpdate"
        />
        <FaderStrip
          :model-value="fixture?.strobe_speed ?? 0"
          :min="0"
          :max="30"
          label="频闪"
          @update:model-value="onStrobeSpeedUpdate"
        />
      </div>
      <div class="zone-preview">
        <OutputPreview />
      </div>
      <div class="zone-panels">
        <div class="panel-col">
          <div class="panel-title">效果</div>
          <div class="panel-body"><EffectPanel /></div>
        </div>
        <div class="panel-col">
          <div class="panel-title">场景</div>
          <div class="panel-body"><CueList /></div>
        </div>
      </div>
    </div>

    <!-- Narrow / mobile: single-column stacked -->
    <div v-else class="console-body-narrow">
      <div class="narrow-controls">
        <div class="narrow-xy">
          <XYPad
            :pan-value="fixture?.pan ?? 0"
            :tilt-value="fixture?.tilt ?? 0"
            @update:pan-value="onPanUpdate"
            @update:tilt-value="onTiltUpdate"
          />
        </div>
        <div class="narrow-wheel">
          <ColorWheel
            :r="fixture?.color.r ?? 1"
            :g="fixture?.color.g ?? 1"
            :b="fixture?.color.b ?? 1"
            @update:r="onColorR"
            @update:g="onColorG"
            @update:b="onColorB"
          />
        </div>
      </div>
      <div class="narrow-faders">
        <FaderStrip
          :model-value="fixture?.pan ?? 0"
          :min="-180"
          :max="180"
          label="水平"
          @update:model-value="onPanUpdate"
        />
        <FaderStrip
          :model-value="fixture?.tilt ?? 0"
          :min="-180"
          :max="180"
          label="垂直"
          @update:model-value="onTiltUpdate"
        />
        <FaderStrip
          :model-value="fixture?.dimmer ?? 1"
          :min="0"
          :max="1"
          label="亮度"
          @update:model-value="onDimmerUpdate"
        />
        <FaderStrip
          :model-value="fixture?.strobe_speed ?? 0"
          :min="0"
          :max="30"
          label="频闪"
          @update:model-value="onStrobeSpeedUpdate"
        />
      </div>
      <div class="narrow-panels">
        <details class="panel" open>
          <summary>效果</summary>
          <EffectPanel />
        </details>
        <details class="panel">
          <summary>场景</summary>
          <CueList />
        </details>
      </div>
    </div>

    <!-- Narrow floating preview -->
    <div v-if="narrowMode && narrowPreviewOpen" class="narrow-preview-float">
      <OutputPreview />
    </div>

    <SettingsModal v-model:open="settingOpen" />
  </div>
</template>

<style scoped>
.console-layout {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: #1a1a2e;
  overflow: hidden;
  user-select: none;
  -webkit-user-select: none;
}

/* ── Header ── */
.console-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.4rem 0.8rem;
  background: #16213e;
  border-bottom: 1px solid #0f3460;
  flex-shrink: 0;
  min-height: 44px;
}

.logo {
  font-size: 1rem;
  font-weight: 700;
  color: #e94560;
  letter-spacing: 0.05em;
}

.version-tag {
  font-size: 0.65rem;
  font-weight: 400;
  color: #6b7a99;
  margin-left: 0.4rem;
  letter-spacing: 0;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.78rem;
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

.icon-btn {
  border: 1px solid #355a8b;
  border-radius: 6px;
  background: #103d6a;
  color: #dfe8ff;
  min-height: 28px;
  padding: 0 8px;
  cursor: pointer;
  touch-action: manipulation;
  font-size: 0.75rem;
}

/* ── Desktop Grid Body ── */
.console-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 1fr 1fr clamp(200px, 20vw, 300px) 100px;
  grid-template-rows: minmax(0, 1.4fr) minmax(0, 1fr);
  grid-template-areas:
    "xy wheel faders preview"
    "panels panels panels preview";
  gap: 6px;
  padding: 6px;
  overflow: hidden;
}

.zone-xy {
  grid-area: xy;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 0;
  overflow: hidden;
}

.zone-wheel {
  grid-area: wheel;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 0;
  overflow: hidden;
}

.zone-faders {
  grid-area: faders;
  display: flex;
  gap: 6px;
  justify-content: center;
  align-items: stretch;
  min-height: 0;
  overflow: hidden;
  padding: 4px;
  background: #141d3a;
  border: 1px solid #274875;
  border-radius: 8px;
}

.zone-preview {
  grid-area: preview;
  min-height: 0;
  overflow: hidden;
  background: #111a35;
  border: 1px solid #274875;
  border-radius: 8px;
}

.zone-panels {
  grid-area: panels;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  min-height: 0;
  overflow: hidden;
}

.panel-col {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  background: #111a35;
  border: 1px solid #274875;
  border-radius: 8px;
}

.panel-title {
  padding: 0.45rem 0.6rem;
  background: #16213e;
  color: #dfe8ff;
  font-weight: 600;
  font-size: 0.82rem;
  border-bottom: 1px solid #274875;
  flex-shrink: 0;
}

.panel-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

/* ── Narrow (< 900px) ── */
.console-body-narrow {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.narrow-controls {
  display: flex;
  gap: 8px;
}

.narrow-xy,
.narrow-wheel {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}

.narrow-faders {
  display: flex;
  gap: 6px;
  justify-content: center;
  align-items: stretch;
  height: 160px;
  padding: 4px;
  background: #141d3a;
  border: 1px solid #274875;
  border-radius: 8px;
}

.narrow-panels {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.panel {
  background: #111a35;
  border: 1px solid #274875;
  border-radius: 8px;
  overflow: hidden;
}

.panel > summary {
  cursor: pointer;
  list-style: none;
  padding: 0.5rem 0.7rem;
  background: #16213e;
  color: #dfe8ff;
  font-weight: 600;
  font-size: 0.82rem;
  border-bottom: 1px solid #274875;
}

.panel > summary::-webkit-details-marker {
  display: none;
}

.narrow-preview-float {
  position: fixed;
  right: 8px;
  bottom: 8px;
  width: 60px;
  height: 480px;
  z-index: 50;
  background: #111a35;
  border: 1px solid #274875;
  border-radius: 8px;
  overflow: hidden;
}

/* ── Responsive narrow hide ── */
.narrow .hide-narrow {
  display: none;
}

.narrow .console-header {
  flex-wrap: wrap;
  gap: 0.3rem;
}
</style>
