<template>
  <li
    class="tab"
    :id="id"
    :data-active="tabData.isTabActive(id)"
    @click="tabData.changeTab(id)">
    <slot />
  </li>
</template>

<script setup lang="ts">
import { inject, defineProps } from "vue";
import type { TabData } from "./tabs";
defineProps({
  id: {
    type: String,
    required: true,
  },
});
const tabData = inject("tabData") as TabData;
</script>
