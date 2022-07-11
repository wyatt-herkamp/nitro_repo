<template>
  <div class="bg-secondary md:mt-1 rounded-lg xl:w-1/2 xl:mx-auto">
    <ul class="flex flex-wrap justify-center justify-between w-1/2 m-auto">
      <li
        v-for="tab in tabNames"
        :key="tab.name"
        class="flex flex-row items-center mx-auto h-12 tab"
        @click="handleClick(tab.name)"
      >
        <span
          v-show="tab.icon"
          class="inline-flex items-center justify-center w-12"
        >
          <box-icon :name="tab.icon"></box-icon>
        </span>
        <span class="text-sm px-2 font-medium mr-2">{{ tab.name }}</span>
      </li>
    </ul>
  </div>
  <slot></slot>
</template>

<script lang="js">
import { defineComponent, provide, ref } from "vue";
export default defineComponent({
  props: {

    defaultTab: {
      required: false,
      type: String,
    },
  },
  emits: ["tabChange"],
  setup(props, { slots }) {
    const tabNames = ref(slots.default().map((item) => {
      return {
        name: item.props.name,
        icon: item.props.icon
      };
    }));
    const tab = ref(props.defaultTab ? props.defaultTab : tabNames.value[0].name);

    provide("tab", tab);
    return { tab, tabNames };
  },
  methods:{
    handleClick(value){
      this.tab = value;
      this.$emit("tabChange", value);
    }
  }
});
</script>
<style scoped>
.tab {
  @apply text-quaternary;
  @apply hover:cursor-pointer;
  @apply hover:border-b-accent/25;
  @apply border-b-2;
  @apply border-transparent;
  @apply hover:shadow-accent;
  @apply hover:drop-shadow-sm;
  transition: border-bottom-color 0.2s ease-in-out;
}
</style>
