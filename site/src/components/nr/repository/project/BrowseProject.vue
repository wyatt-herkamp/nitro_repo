<template>
  <div>
    <div v-if="projectHandler">
      <component
        :is="projectHandler.projectComponent.component"
        :project="project"
        :version="version"
        :repository="repository" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { Project } from "@/types/project";
import { findRepositoryType, type RepositoryWithStorageName } from "@/types/repository";
import { computed, type PropType } from "vue";

const props = defineProps({
  project: {
    type: Project,
    required: true,
  },
  version: {
    type: Object,
    required: false,
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});
const projectHandler = computed(() => {
  return findRepositoryType(props.repository.repository_type);
});
</script>
