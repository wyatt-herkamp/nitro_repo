<template>
  <li>
    <router-link
      :to="href"
      :class="[active ? 'active' : 'notActive']"
      class="flex flex-row items-center h-12 text-slate-50 hover:text-slate-100"
    >
      <span
        class="
          inline-flex
          items-center
          justify-center
          h-12
          w-12
          text-lg text-gray-400
        "
      >
        <box-icon :name="icon"></box-icon>
      </span>
      <span
        :class="[active ? 'active' : 'notActive']"
        class="text-sm font-medium"
        >{{ name }}</span
      >
    </router-link>
  </li>
</template>
<script lang="ts">
import { defineComponent, ref } from "vue";

export default defineComponent({
  props: {
    icon: {
      required: true,
      type: String,
    },
    href: {
      required: true,
      type: String,
    },
    name: {
      required: true,
      type: String,
    },
    active: {
      required: true,
      type: Boolean,
    },
  },
  setup() {},
});
</script>
<style scoped>
.active {
  @apply font-bold;
}
.notActive {
  @apply transform;
  @apply hover:translate-x-2;
  @apply transition-transform;
  @apply ease-in;
  @apply duration-200;
}
</style>