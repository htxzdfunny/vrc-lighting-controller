<script setup lang="ts">
import { reactive, watch } from "vue";
import { useLightingStore } from "../stores/lighting";

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{ "update:open": [value: boolean] }>();

const store = useLightingStore();

const form = reactive({
  fixture_count: store.config.fixture_count,
  output_fps: store.config.output_fps,
  ndi_enabled: store.config.ndi_enabled,
  ndi_name: store.config.ndi_name,
  spout_enabled: (store.config as any).spout_enabled ?? false,
  spout_name: (store.config as any).spout_name ?? "VRC-Lighting-Spout",
  web_port: store.config.web_port,
  output_window_visible: (store.config as any).output_window_visible ?? true,
});

function syncFormFromConfig(cfg: any) {
  form.fixture_count = cfg.fixture_count;
  form.output_fps = cfg.output_fps;
  form.ndi_enabled = cfg.ndi_enabled;
  form.ndi_name = cfg.ndi_name;
  form.web_port = cfg.web_port;
  form.spout_enabled = cfg.spout_enabled ?? false;
  form.spout_name = cfg.spout_name ?? "VRC-Lighting-Spout";
  form.output_window_visible = cfg.output_window_visible ?? true;
}

watch(
  () => props.open,
  (open) => {
    if (open) syncFormFromConfig(store.config as any);
  },
  { immediate: true }
);

watch(
  () => store.config,
  (cfg) => {
    // Keep form stable while modal is open, so checkbox edits are not
    // immediately overwritten by backend-pushed state snapshots.
    if (!props.open) syncFormFromConfig(cfg as any);
  },
  { deep: true }
);

async function save() {
  await store.setConfig({
    ...store.config,
    ...form,
  } as any);
  emit("update:open", false);
}

async function resetAll() {
  await store.resetState();
}

async function exportConfig() {
  await store.exportState("./lighting-state-export.json");
}

async function importConfig() {
  await store.importState("./lighting-state-export.json");
}
</script>

<template>
  <teleport to="body">
    <div v-if="open" class="overlay" @click.self="emit('update:open', false)">
      <div class="modal">
        <h3>设置</h3>
        <div class="grid">
          <label>灯具数量</label>
          <input v-model.number="form.fixture_count" type="number" min="1" max="32" />

          <label>输出帧率</label>
          <select v-model.number="form.output_fps">
            <option :value="30">30</option>
            <option :value="60">60</option>
          </select>

          <label>NDI 启用</label>
          <input v-model="form.ndi_enabled" type="checkbox" />

          <label>NDI 名称</label>
          <input v-model="form.ndi_name" type="text" />

          <label>Spout2 启用</label>
          <input v-model="form.spout_enabled" type="checkbox" />

          <label>Spout2 名称</label>
          <input v-model="form.spout_name" type="text" />

          <label>Web 端口</label>
          <input v-model.number="form.web_port" type="number" min="1024" max="65535" />

          <label>输出窗口可见</label>
          <input v-model="form.output_window_visible" type="checkbox" />
        </div>
        <div class="actions">
          <button @click="resetAll">重置全部</button>
          <button @click="exportConfig">导出配置</button>
          <button @click="importConfig">导入配置</button>
          <button @click="emit('update:open', false)">取消</button>
          <button class="primary" @click="save">保存</button>
        </div>
      </div>
    </div>
  </teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: grid;
  place-items: center;
  z-index: 1000;
}

.modal {
  width: min(560px, 90vw);
  background: #152444;
  border: 1px solid #21406f;
  border-radius: 10px;
  padding: 1rem;
}

h3 {
  margin: 0 0 0.8rem;
}

.grid {
  display: grid;
  grid-template-columns: 140px 1fr;
  gap: 0.6rem;
  align-items: center;
}

input,
select {
  background: #0f3460;
  border: 1px solid #355a8b;
  color: #fff;
  border-radius: 6px;
  min-height: 34px;
  padding: 0 8px;
}

.actions {
  margin-top: 0.8rem;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

button {
  min-height: 34px;
  border-radius: 6px;
  border: 1px solid #355a8b;
  background: #0f3460;
  color: #fff;
  padding: 0 12px;
}

.primary {
  background: #e94560;
  border-color: #e94560;
}
</style>
