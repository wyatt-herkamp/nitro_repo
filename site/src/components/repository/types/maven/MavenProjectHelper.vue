<template>
  <div class="mavenProject">
    <h1>{{ project.name }}</h1>
    <div class="info">
      <div class="codeBlock">
        <h2>Project Info</h2>
        <CodeMenu :snippets="snippets" />
      </div>
      <div class="details">
        <KeyAndValue :label="'Group Id'" :value="project.scope || 'undefined'" />
        <KeyAndValue :label="'Artifact Id'" :value="project.name || 'undefined'" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Project } from '@/types/project'
import type { RepositoryWithStorageName } from '@/types/repository'
import type { PropType } from 'vue'
import { createProjectSnippets } from './MavenRepositoryHelpers'
import CodeMenu from '@/components/code/CodeMenu.vue'
import KeyAndValue from '@/components/form/KeyAndValue.vue'

const props = defineProps({
  project: {
    type: Object as PropType<Project>,
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
const snippets = createProjectSnippets(props.project)
</script>

<style lang="scss" scoped>
.mavenProject {
  margin: 0 auto;
}
.details {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}
.info {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap-reverse;
}
.codeBlock {
  flex-grow: 1;
  max-width: 50%;
}
</style>
