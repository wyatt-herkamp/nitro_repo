<template>
  <div class="tabs">
    <ul class="tabsHeader" :data-jb="justifyBetween">
      <slot name="header" />
    </ul>
    <div class="tabsContent">
      <slot name="content" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { provide, ref, type Ref } from 'vue'
import './tabs.scss'
const props = defineProps({
  defaultTab: {
    type: String
  },
  justifyBetween: {
    type: Boolean,
    default: true
  }
})
import { useSlots } from 'vue'
import type { TabData } from './tabs'
const backupDefaultSlot = ref('')
const slots = useSlots()
if (slots) {
  console.log('slots.header', slots.header)
  console.log('slots.content', slots.content)
}
const currentTab: Ref<string> = ref(props.defaultTab || backupDefaultSlot.value)
const changeTab = (tab: string) => {
  currentTab.value = tab
}
const getTab = () => {
  return currentTab.value
}
const isTabActive = (tab: string) => {
  return currentTab.value === tab
}
const tabData = {
  changeTab,
  getTab,
  isTabActive
} as TabData

provide('tabData', tabData)
</script>
