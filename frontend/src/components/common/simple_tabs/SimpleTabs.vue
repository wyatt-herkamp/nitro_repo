<template>
  <Tabs v-model="tab">
    <Tab
      v-for="tab in tabNames"
      :key="tab.name"
      :name="tab.name"
      @click="handleClick(tab.name)"
    >
      {{ tab.name }}
    </Tab>
  </Tabs>
  <slot></slot>
</template>

<script lang="js">
import { defineComponent, provide, ref } from "vue";
import Tabs from "@/components/common/tabs/Tabs.vue";
import Tab from "@/components/common/tabs/Tab.vue";


export default defineComponent({
  name: "SimpleTabs",
  components: {Tab, Tabs},
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
