<template>
  <div>
    <div v-if="projectHandler">
      <component
        :is="projectHandler.component"
        :project="project"
        :version="version"
        :repository="repository" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { findProjectHandler, Project } from '@/types/project'
import type { RepositoryWithStorageName } from '@/types/repository'
import { computed, type PropType } from 'vue'

const props = defineProps({
  project: {
    type: Project,
    required: true
  },
  version: {
    type: Object,
    required: false
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true
  }
})
const projectHandler = computed(() => {
  return findProjectHandler(props.repository.repository_type)
})
</script>
