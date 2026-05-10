<script setup lang="ts">
import { computed } from "vue";
import { useLightingStore } from "../../stores/lighting";
import VerticalFader from "./VerticalFader.vue";

const store = useLightingStore();

const rValue = computed({
  get: () => store.workingColor.r,
  set: (v: number) => {
    store.workingColor = { ...store.workingColor, r: v };
    if (!store.editMode) {
      store.setSelectedColor(
        store.workingColor.r,
        store.workingColor.g,
        store.workingColor.b
      );
    }
  },
});
const gValue = computed({
  get: () => store.workingColor.g,
  set: (v: number) => {
    store.workingColor = { ...store.workingColor, g: v };
    if (!store.editMode) {
      store.setSelectedColor(
        store.workingColor.r,
        store.workingColor.g,
        store.workingColor.b
      );
    }
  },
});
const bValue = computed({
  get: () => store.workingColor.b,
  set: (v: number) => {
    store.workingColor = { ...store.workingColor, b: v };
    if (!store.editMode) {
      store.setSelectedColor(
        store.workingColor.r,
        store.workingColor.g,
        store.workingColor.b
      );
    }
  },
});
const mValue = computed({
  get: () => store.workingMaster,
  set: (v: number) => {
    store.workingMaster = v;
    if (!store.editMode) {
      store.setSelectedDimmer(v);
    }
  },
});

const previewColor = computed(() => {
  const c = store.getEffectiveColor();
  return `rgb(${Math.round(c.r * 255)}, ${Math.round(c.g * 255)}, ${Math.round(c.b * 255)})`;
});
</script>

<template>
  <div class="rgb-col">
    <div class="faders-row">
      <VerticalFader
        v-model="rValue"
        label="R"
        gradient="linear-gradient(to top, #200 0%, #f33 100%)"
      />
      <VerticalFader
        v-model="gValue"
        label="G"
        gradient="linear-gradient(to top, #020 0%, #2f3 100%)"
      />
      <VerticalFader
        v-model="bValue"
        label="B"
        gradient="linear-gradient(to top, #002 0%, #38f 100%)"
      />
      <VerticalFader
        v-model="mValue"
        label="M"
        gradient="linear-gradient(to top, #111 0%, #ddd 100%)"
      />
    </div>
    <div class="preview">
      <span class="preview-label">输出颜色</span>
      <div class="preview-swatch" :style="{ background: previewColor }" />
    </div>
  </div>
</template>

<style scoped>
.rgb-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  min-height: 0;
}

.faders-row {
  display: flex;
  gap: clamp(6px, 1vw, 14px);
  flex: 1;
  min-height: 0;
  padding: 0 4px;
}

.preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.preview-label {
  font-size: clamp(0.62rem, 1vw, 0.74rem);
  color: #b8c4dc;
}

.preview-swatch {
  width: clamp(46px, 6vw, 70px);
  height: clamp(46px, 6vw, 70px);
  border-radius: 6px;
  border: 1px solid #355a8b;
  box-shadow: inset 0 0 0 1px #0a1228;
}
</style>
