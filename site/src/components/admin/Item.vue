<template>
  <li>
    <router-link
      :to="href"
      class="flex flex-row items-center h-12 transform hover:translate-x-2 transition-transform ease-in duration-200 text-slate-50 hover:text-slate-100"
    >
      <span
        class="inline-flex items-center justify-center h-12 w-12 text-lg text-gray-400"
      >
        <box-icon :name="icon"></box-icon>
      </span>
      <span class="font-bold text-sm font-medium">{{ name }}</span>
    </router-link>
  </li>
</template>
<script>
import {defineComponent} from "vue";

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
  },
  setup() {},
});
</script>
