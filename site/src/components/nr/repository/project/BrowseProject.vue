<template>
  <div v-if="projectResolution.project_id">
    <div v-if="project">
      <div v-if="projectHandler && projectHandler.projectComponent">
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
import type { Project, ProjectVersion } from "@/types/project";
import { findRepositoryType, type RepositoryWithStorageName } from "@/types/repository";
import { computed, ref, type PropType, type Ref } from "vue";

const props = defineProps({
  projectResolution: {
    type: Object as PropType<ProjectResolution>,
    required: true,
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});
const projectStore = useProjectStore();
const project: Ref<Project | undefined> = ref(undefined);
// Browsing a version directory resolves a version id, and the snippets shown alongside its files
// should name that version. It was never fetched, so they all fell back to "latest".
const version: Ref<ProjectVersion | undefined> = ref(undefined);
if (props.projectResolution.project_id) {
  projectStore.getProjectById(props.projectResolution.project_id).then((response) => {
    project.value = response;
  });
}
if (props.projectResolution.version_id) {
  projectStore.getVersionById(props.projectResolution.version_id).then((response) => {
    version.value = response;
  });
}
const projectHandler = computed(() => {
  return findRepositoryType(props.repository.repository_type);
});
</script>
