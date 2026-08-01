<template>
  <div class="tabs">
    <ul
      class="tabsHeader"
      role="tablist"
      :data-jb="justifyBetween">
      <slot name="header" />
    </ul>
    <div class="tabsContent">
      <slot name="content" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { provide, ref, type Ref } from "vue";
import "./tabs.scss";
import type { TabData } from "./tabs";

const props = defineProps({
  defaultTab: {
    type: String,
  },
  justifyBetween: {
    type: Boolean,
    default: true,
  },
});

const currentTab: Ref<string> = ref(props.defaultTab ?? "");

const tabData: TabData = {
  changeTab: (tab: string) => {
    currentTab.value = tab;
  },
  getTab: () => currentTab.value,
  isTabActive: (tab: string) => currentTab.value === tab,
};

provide("tabData", tabData);
</script>
