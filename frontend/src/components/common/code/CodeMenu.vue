<template>
  <div class="codeMenu">
    <div class="selector">
      <slot name="header"></slot>
      <nav class="flex flex-wrap mb-1">
        <div
          v-for="code in codes"
          :key="code.name"
          class="item"
          :class="currentTab.name === code.name ? 'active' : ''"
          @click="currentTab = code"
        >
          {{ code.name }}
        </div>
      </nav>
    </div>
    <div class="codeBlock">
      <CodeCard :snippetInfo="currentTab" />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import httpCommon from "@/http-common";
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
    const url = httpCommon.apiURL;
    const currentTab = ref<SnippetInfo>(props.codes[0]);

    return { url, currentTab };
  },
});
</script>

<style>
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
  @apply lg:max-w-md;
  @apply h-56;
}

.codeBlock {
  @apply pl-2;
  @apply my-auto;
  font-family: "Fira Code", monospace;
  font-size: 14px;
  @apply text-white;
  @apply lg:whitespace-pre;
  @apply overflow-x-auto;
  @apply h-1/2;
  @apply select-all;
}
</style>
