<template>
  <div v-if="projectResolution.project_id">
    <div v-if="project">
      <div v-if="projectHandler">
        <component
          :is="projectHandler.projectComponent.component"
          :project="project"
          :version="version"
          :repository="repository" />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { useProjectStore } from "@/stores/project_store";
import type { ProjectResolution } from "@/types/browse";
import type { Project } from "@/types/project";
import { findRepositoryType, type RepositoryWithStorageName } from "@/types/repository";
import { computed, ref, type PropType, type Ref } from "vue";

const props = defineProps({
  projectResolution: {
    type: Object as PropType<ProjectResolution>,
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
const projectStore = useProjectStore();
const project: Ref<Project | undefined> = ref(undefined);
if (props.projectResolution.project_id) {
  projectStore.getProjectById(props.projectResolution.project_id).then((response) => {
    project.value = response;
  });
}
const projectHandler = computed(() => {
  return findRepositoryType(props.repository.repository_type);
});
</script>
