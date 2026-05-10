<script setup lang="ts">
import { ref } from "vue";
import { useLightingStore } from "../stores/lighting";

const store = useLightingStore();

const newCueName = ref("");
const fadeTime = ref(1.0);

function saveCue() {
  const name = newCueName.value.trim() || `场景 ${store.cues.length + 1}`;
  store.saveCue(name, fadeTime.value);
  newCueName.value = "";
}

function goCue(id: string) {
  store.goCue(id);
}

function deleteCue(id: string) {
  store.deleteCue(id);
}
</script>

<template>
  <div class="cue-list">
    <div class="cue-controls">
      <input
        v-model="newCueName"
        class="cue-name-input"
        placeholder="场景名称..."
      />
      <div class="fade-control">
        <label>淡入</label>
        <input
          v-model.number="fadeTime"
          type="number"
          min="0"
          max="30"
          step="0.5"
          class="fade-input"
        />
        <span class="fade-unit">s</span>
      </div>
      <button class="save-btn" @click="saveCue">保存</button>
    </div>

    <div class="cue-items">
      <div
        v-for="cue in store.cues"
        :key="cue.id"
        class="cue-item"
        :class="{ active: store.active_cue_id === cue.id }"
      >
        <button
          class="go-btn"
          @click="goCue(cue.id)"
          @touchend.prevent="goCue(cue.id)"
        >
          触发
        </button>
        <div class="cue-info">
          <span class="cue-name">{{ cue.name }}</span>
          <span class="cue-fade">{{ cue.fade_time }}s 淡入</span>
        </div>
        <button
          class="delete-btn"
          @click="deleteCue(cue.id)"
          @touchend.prevent="deleteCue(cue.id)"
        >×</button>
      </div>
      <div v-if="store.cues.length === 0" class="empty-hint">
        还没有已保存场景，调整灯光后点击“保存”。
      </div>
    </div>
  </div>
</template>

<style scoped>
.cue-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.5rem;
}

.cue-controls {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  flex-wrap: wrap;
}

.cue-name-input {
  flex: 1;
  min-width: 120px;
  background: #0f3460;
  border: 1px solid #233c6e;
  border-radius: 4px;
  color: #e0e0e0;
  padding: 0.4rem 0.6rem;
  font-size: 0.85rem;
}

.fade-control {
  display: flex;
  align-items: center;
  gap: 4px;
}

.fade-control label {
  font-size: 0.75rem;
  color: #888;
}

.fade-input {
  width: 50px;
  background: #0f3460;
  border: 1px solid #233c6e;
  border-radius: 4px;
  color: #e0e0e0;
  padding: 0.3rem;
  font-size: 0.8rem;
  text-align: center;
}

.fade-unit {
  font-size: 0.75rem;
  color: #888;
}

.save-btn {
  background: #e94560;
  color: #fff;
  border: none;
  padding: 0.4rem 1rem;
  border-radius: 4px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  touch-action: manipulation;
  min-height: 40px;
}

.save-btn:active {
  opacity: 0.8;
}

.cue-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.cue-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.3rem;
  background: #16213e;
  border-radius: 6px;
  border: 2px solid transparent;
  transition: border-color 0.15s;
}

.cue-item.active {
  border-color: #e94560;
}

.go-btn {
  min-width: 48px;
  min-height: 48px;
  background: #0f3460;
  color: #4cc9f0;
  border: none;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 700;
  cursor: pointer;
  touch-action: manipulation;
}

.go-btn:active {
  background: #4cc9f0;
  color: #000;
}

.cue-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.cue-name {
  font-size: 0.85rem;
}

.cue-fade {
  font-size: 0.7rem;
  color: #888;
}

.delete-btn {
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  color: #666;
  font-size: 1.2rem;
  cursor: pointer;
  border-radius: 4px;
}

.delete-btn:hover {
  color: #e94560;
  background: rgba(233, 69, 96, 0.1);
}

.empty-hint {
  font-size: 0.8rem;
  color: #555;
  text-align: center;
  padding: 2rem 1rem;
}
</style>
