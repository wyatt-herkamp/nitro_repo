<template>
  <div>
    <h1>//TODO</h1>
  </div>
</template>
<script setup lang="ts">
import type { Project, ProjectVersion } from '@/types/project'
import type { RepositoryWithStorageName } from '@/types/repository'
import type { PropType } from 'vue'

const props = defineProps({
  project: {
    type: Object as PropType<Project>,
    required: true
  },
  version: {
    type: Object as PropType<ProjectVersion>,
    required: false
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true
  }
})
</script>
