<template>
  <div v-if="tabData.isTabActive(tabId)">
    <slot />
  </div>
</template>

<script setup lang="ts">
defineProps({
  tabId: {
    type: String,
    required: true,
  },
});

import { inject } from "vue";
import type { TabData } from "./tabs";
const tabData = inject("tabData") as TabData;
</script>
