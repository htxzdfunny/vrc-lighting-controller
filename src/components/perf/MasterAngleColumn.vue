<script setup lang="ts">
import { ref } from "vue";
import { useLightingStore } from "../../stores/lighting";
import VerticalFader from "./VerticalFader.vue";
import RotaryKnob from "./RotaryKnob.vue";

const store = useLightingStore();

const masterFader = ref(0.5);
const masterKnob = ref(0.5);

function onFaderUpdate(v: number) {
  masterFader.value = v;
  store.applyMasterFader(v);
}

function onKnobUpdate(v: number) {
  masterKnob.value = v;
  store.applyMasterKnob(v);
}
</script>

<template>
  <div class="master-col">
    <div class="fader-area">
      <VerticalFader
        :model-value="masterFader"
        gradient="linear-gradient(to top, #1a0a14 0%, #ff2855 100%)"
        @update:model-value="onFaderUpdate"
      />
    </div>
    <span class="master-tag">总控</span>
    <div class="knob-area">
      <RotaryKnob
        :model-value="masterKnob"
        accent="#ff2855"
        @update:model-value="onKnobUpdate"
      />
    </div>
    <span class="hint">M</span>
  </div>
</template>

<style scoped>
.master-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  min-width: clamp(50px, 6vw, 68px);
  padding-right: 8px;
  border-right: 1px dashed #355a8b;
  height: 100%;
  min-height: 0;
}

.fader-area {
  flex: 1;
  min-height: 0;
  width: 100%;
  display: flex;
  justify-content: center;
}

.master-tag {
  font-size: 0.62rem;
  letter-spacing: 0.05em;
  padding: 2px 6px;
  border-radius: 4px;
  background: #ff2855;
  color: #fff;
  flex-shrink: 0;
}

.knob-area {
  flex-shrink: 0;
}

.hint {
  font-size: clamp(0.7rem, 1.1vw, 0.85rem);
  color: #ff5e7d;
  font-weight: 700;
  flex-shrink: 0;
}
</style>
