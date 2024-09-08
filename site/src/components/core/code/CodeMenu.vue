<template>
  <TabsElement :defaultTab="defaultTab" :justifyBetween="false">
    <template #header>
      <TabElement :id="snippet.key" v-for="snippet in snippets" :key="snippet.key">
        {{ snippet.name }}
      </TabElement>
    </template>
    <template #content>
      <TabContent v-for="snippet in snippets" :tabId="snippet.key" :key="snippet.key">
        <CodeCard :code="snippet" />
      </TabContent>
    </template>
  </TabsElement>
</template>

<script setup lang="ts">
import type { PropType } from 'vue'
import type { CodeSnippet } from './code'

import CodeCard from './CodeCard.vue'
import TabContent from '../tabs/TabContent.vue'
import TabsElement from '../tabs/TabsElement.vue'
import TabElement from '../tabs/TabElement.vue'

defineProps({
  snippets: {
    type: Array as PropType<CodeSnippet[]>,
    required: true
  },
  defaultTab: {
    type: String,
    required: true
  }
})
</script>
<style lang="scss">
@media screen and (max-width: 600px) {
  .tabs {
    max-width: 99%;
  }
}
</style>
