<template>
  <li
    @click="handleChange"
    class="flex flex-row items-center mx-auto h-12 tab min-w-fit"
  >
    <span
      v-show="$slots.icon"
      class="inline-flex items-center justify-center pr-2"
    >
      <slot name="icon" />
    </span>
    <span class="text-sm font-medium mr-2"> <slot /> </span>
  </li>
</template>

<script lang="ts">
import { defineComponent, inject } from "vue";
import { TabData } from "./TabData";
export default defineComponent({
  props: {
    name: {
      required: false,
      default: "",
      type: String,
    },
    disabled: {
      required: false,
      default: false,
      type: Boolean,
    },
  },
  slots: ["icon", "default"],
  setup(props, { emit }) {
    const tab: TabData | undefined = inject("tabData");

    const handleChange = (): void => {
      if (tab && !props.disabled) {
        tab.update(props.name);
        emit("click", props.name);
      }
    };
    return { tab, handleChange };
  },
});
</script>
<style scoped>
.tab {
  @apply text-quaternary;
  @apply hover:cursor-pointer;
  @apply hover:border-b-accent/25;
  @apply border-b-2;
  @apply border-transparent;

  transition: border-bottom-color 0.2s ease-in-out;
}
</style>
