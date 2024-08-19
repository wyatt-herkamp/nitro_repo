<template>
  <div class="codeMenu">
    <div class="selector">
      <div
        v-for="snippet in snippets"
        class="tab"
        :key="snippet.key"
        @click="currentTab = snippet.key"
        :data-active="currentTab === snippet.key">
        {{ snippet.name }}
      </div>
    </div>
    <div class="codeBlock" v-if="currentSnippet">
      <CodeCard :snippetInfo="currentSnippet" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, type PropType } from 'vue'
import type { CodeSnippet } from './code'
import CodeCard from './CodeCard.vue'

const props = defineProps({
  snippets: {
    type: Array as PropType<CodeSnippet[]>,
    required: true
  }
})
const currentTab = ref<string>(props.snippets[0].key)
const currentSnippet = computed(() => {
  return props.snippets.find((snippet) => snippet.key === currentTab.value)
})
</script>
<style lang="scss" scoped>
@import '@/assets/styles/theme.scss';
.codeMenu {
  width: 100%;
  background-color: $background-50;
}
.selector {
  display: flex;
  gap: 1rem;
  align-items: center;
  padding: 1rem;
  flex-direction: row;
  border-bottom: 1px solid $accent-50;
}
.tab {
  padding: 1rem;
  cursor: pointer;
  border-radius: 0.5rem 0.5rem 0 0;
  border: 1px solid $primary-50;
  &:hover {
    background-color: $accent;
    color: white;
  }
}
</style>
<style lang="scss"></style>
