<template>
  <div class="codeMenu">
    <div class="selector">
      <slot name="header"></slot>
      <nav class="flex flex-wrap p-6 m-1">
        <div
          v-for="code in codes"
          :key="code.name"
          :class="currentTab == code.name ? 'active item' : 'item'"
          @click="currentTab = code.name"
        >
          {{ code.name }}
        </div>
      </nav>
    </div>
    <div class="codeBox">
      <div v-for="entry in codes" :key="entry.name">
        <CodeViewComp v-if="entry.name === currentTab" :snippetInfo="entry" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { apiURL } from "@/http-common";
import CodeCard from "@/components/common/code/CodeCard.vue";
import { SnippetInfo } from "@/api/CodeGenGeneral";

export default defineComponent({
  components: { CodeCard },
  props: {
    codes: {
      type: Object as () => SnippetInfo[],
      required: true,
    },
  },
  setup(props) {
    const url = apiURL;
    console.log(props.codes.length);
    const currentTab = ref(props.codes[0].name);

    return { url, currentTab };
  },
});
</script>

<style>
.selector {
}
.active {
  @apply text-yellow-50 !important;
  @apply border-slate-900 !important;
}

.item {
  @apply text-white;
  @apply py-4;
  @apply px-7;
  @apply flex-grow;
  @apply text-center;
  @apply border-b-2;
  @apply cursor-pointer;
  @apply border-transparent;
}
.codeMenu {
  @apply bg-slate-800;
}
.codeBox {
  @apply relative;
  @apply h-24;
  @apply overflow-hidden;
}
.nitroEditor {
}
.prism-editor__textarea:focus {
  outline: none;
}
</style>
