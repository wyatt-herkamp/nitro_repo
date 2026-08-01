<template>
  <li role="presentation">
    <button
      :id="`tab-${id}`"
      type="button"
      role="tab"
      class="tab"
      :data-active="tabData.isTabActive(id)"
      :aria-selected="tabData.isTabActive(id)"
      :aria-controls="`panel-${id}`"
      @click="tabData.changeTab(id)">
      <slot />
    </button>
  </li>
</template>

<script setup lang="ts">
/**
 * A real `<button>` inside the list item, rather than a clickable `<li>` — so a tab can be reached
 * by keyboard at all, and is announced as a tab rather than as a list item.
 */
import { inject } from "vue";
import type { TabData } from "./tabs";

defineProps({
  id: {
    type: String,
    required: true,
  },
});

const tabData = inject("tabData") as TabData;
</script>
