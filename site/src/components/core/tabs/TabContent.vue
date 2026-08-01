<template>
  <div
    v-if="tabData.isTabActive(tabId)"
    :id="`panel-${tabId}`"
    role="tabpanel"
    :aria-labelledby="`tab-${tabId}`">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { inject } from "vue";
import type { TabData } from "./tabs";

defineProps({
  tabId: {
    type: String,
    required: true,
  },
});

const tabData = inject("tabData") as TabData;
</script>
