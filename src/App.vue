<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import ConsoleLayout from "./layouts/ConsoleLayout.vue";
import PerformanceLayout from "./layouts/PerformanceLayout.vue";

type LayoutId = "perf" | "console";
const STORAGE_KEY = "vrc-layout-preference";

function loadInitialLayout(): LayoutId {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "console") return "console";
    if (v === "perf") return "perf";
  } catch {
    // ignore (private mode or SSR)
  }
  return "perf";
}

const layout = ref<LayoutId>(loadInitialLayout());

function switchTo(target: string) {
  if (target === "console" || target === "perf") {
    layout.value = target;
  }
}

watch(layout, (v) => {
  try {
    localStorage.setItem(STORAGE_KEY, v);
  } catch {
    // ignore
  }
});

onMounted(() => {
  layout.value = loadInitialLayout();
});
</script>

<template>
  <PerformanceLayout
    v-if="layout === 'perf'"
    @switch-layout="switchTo"
  />
  <ConsoleLayout
    v-else
    @switch-layout="switchTo"
  />
</template>
