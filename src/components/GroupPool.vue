<script setup lang="ts">
import { ref } from "vue";
import { useLightingStore } from "../stores/lighting";

const store = useLightingStore();
const showCreateDialog = ref(false);
const newGroupName = ref("");

function selectGroup(fixtureIds: number[]) {
  store.selectFixtures(fixtureIds);
}

function createGroup() {
  if (!newGroupName.value.trim()) return;
  store.createGroup(newGroupName.value.trim(), [...store.selected_fixture_ids]);
  newGroupName.value = "";
  showCreateDialog.value = false;
}

function removeGroup(id: string) {
  store.deleteGroup(id);
}
</script>

<template>
  <div class="group-pool">
    <div class="pool-header">
      <span class="pool-title">编组</span>
      <button class="add-btn" @click="showCreateDialog = !showCreateDialog">+</button>
    </div>

    <div v-if="showCreateDialog" class="create-dialog">
      <input
        v-model="newGroupName"
        class="group-name-input"
        placeholder="输入编组名称..."
        @keyup.enter="createGroup"
      />
      <button class="create-btn" @click="createGroup">创建</button>
    </div>

    <div class="pool-items">
      <div
        v-for="group in store.groups"
        :key="group.id"
        class="group-item"
        @click="selectGroup(group.fixture_ids)"
        @touchend.prevent="selectGroup(group.fixture_ids)"
      >
        <span class="group-name">{{ group.name }}</span>
        <span class="group-count">{{ group.fixture_ids.length }}</span>
        <button
          class="remove-btn"
          @click.stop="removeGroup(group.id)"
          @touchend.stop.prevent="removeGroup(group.id)"
        >×</button>
      </div>
      <div v-if="store.groups.length === 0" class="empty-hint">
        暂无编组
      </div>
    </div>
  </div>
</template>

<style scoped>
.group-pool {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.5rem;
}

.pool-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.pool-title {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #888;
}

.add-btn {
  width: 28px;
  height: 28px;
  background: #0f3460;
  color: #e94560;
  border: none;
  border-radius: 4px;
  font-size: 1.1rem;
  cursor: pointer;
  touch-action: manipulation;
}

.create-dialog {
  display: flex;
  gap: 4px;
}

.group-name-input {
  flex: 1;
  background: #0f3460;
  border: 1px solid #233c6e;
  border-radius: 4px;
  color: #e0e0e0;
  padding: 0.3rem 0.5rem;
  font-size: 0.8rem;
}

.create-btn {
  background: #e94560;
  color: #fff;
  border: none;
  padding: 0.3rem 0.6rem;
  border-radius: 4px;
  font-size: 0.8rem;
  cursor: pointer;
}

.pool-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.group-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  background: #16213e;
  border-radius: 6px;
  cursor: pointer;
  min-height: 40px;
  touch-action: manipulation;
}

.group-item:active {
  background: #1a2a50;
}

.group-name {
  flex: 1;
  font-size: 0.85rem;
}

.group-count {
  font-size: 0.7rem;
  color: #888;
  background: #0f3460;
  padding: 0.1rem 0.4rem;
  border-radius: 3px;
}

.remove-btn {
  width: 24px;
  height: 24px;
  background: none;
  border: none;
  color: #666;
  font-size: 1rem;
  cursor: pointer;
  border-radius: 3px;
}

.remove-btn:hover {
  color: #e94560;
  background: rgba(233, 69, 96, 0.1);
}

.empty-hint {
  font-size: 0.75rem;
  color: #555;
  text-align: center;
  padding: 1rem;
}
</style>
