<template>
  <nav class="flex flex-wrap bg-slate-800 p-6 m-1 roun">
    <a @click="changeViewValue('General')">
      <div
        class="
          mx-2
          py-1.5
          rounded-lg
          bg-slate-700
          font-bold
          px-6
          m-1
          text-white
          cursor-pointer
        "
      >
        General
      </div>
    </a>

    <a @click="changeViewValue('Password')">
      <div
        class="
          mx-2
          py-1.5
          rounded-lg
          bg-slate-700
          font-bold
          px-6
          m-1
          text-white
          cursor-pointer
        "
      >
        Password
      </div>
    </a>
    <div
      class="
        mx-2
        py-1.5
        rounded-lg
        bg-slate-700
        font-bold
        px-6
        m-1
        text-white
        cursor-pointer
      "
    >
      {{ user }}
    </div>
  </nav>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";

import { apiURL } from "@/http-common";

export default defineComponent({
  props: {
    user: {
      type: String,
      required: true,
    },
  },
  setup() {
    const url = apiURL;
    return { url };
  },
  methods: {
    changeViewValue(value: string) {
      console.log(value);
      this.$emit("changeView", value);
    },
  },
});
</script>
<style scoped>
.end {
  margin-left: auto;
}
</style>
