<template>
  <div v-if="page.page_type == PageType.Markdown">
    <vue-markdown :source="page.content" />
  </div>
</template>
<script setup lang="ts">
import { defineProps, type PropType } from 'vue'
import { PageType, type RepositoryPage, type RepositoryWithStorageName } from '@/types/repository'
import VueMarkdown from 'vue-markdown-render'

const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true
  },
  page: {
    type: Object as PropType<RepositoryPage>,
    required: true
  }
})
</script>
