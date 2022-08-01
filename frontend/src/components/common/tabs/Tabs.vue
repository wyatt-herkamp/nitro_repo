<template>
  <div class="bg-secondary md:mt-1 rounded-lg xl:w-1/2 xl:mx-auto">
    <ul class="tabs" v-bind="$attrs">
      <slot />
    </ul>
  </div>
</template>

<script lang="ts">
import { defineComponent, provide, ref } from "vue";
import { TabData } from "./TabData";
export default defineComponent({
  props: {
    modelValue: String,
  },
  emits: ["tabChange", "update:modelValue"],
  setup(props, { emit }) {
    const tab = ref(props.modelValue ? props.modelValue : "Users");
    const handleChange = (new_tab: string): void => {
      emit("update:modelValue", new_tab);
      emit("tabChange", new_tab);
      tab.value = new_tab;
    };
    const data: TabData = {
      currentTab: tab,
      update: handleChange,
    };
    provide("tabData", data);
    return { tab };
  },
});
</script>
<style scoped>
.tabs {
  @apply flex;
  @apply flex-wrap;
  @apply lg:flex-nowrap;
  @apply justify-between;
  @apply m-auto;
}
</style>
