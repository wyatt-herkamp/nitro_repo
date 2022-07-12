<template>
  <div v-show="tab === name">
    <slot />
  </div>
</template>

<script lang="js">
import { inject } from "vue";
export default {
  name: "SimpleTab",
  props: {
    icon: {
      required: false,
      type: String,
    },
    name: {
      required: true,
      type: String,
    },
  },
  setup() {
    const tab = inject("tab");

    return { tab };
  },
}
</script>
