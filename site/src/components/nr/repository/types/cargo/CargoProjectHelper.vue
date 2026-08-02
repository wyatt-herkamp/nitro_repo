<template>
  <div class="cargoProject">
    <div class="projectHeader">
      <h1>{{ project.name }}</h1>
      <RouterLink
        class="openProject"
        :to="{ name: 'ProjectPageView', params: { projectId: project.id } }"
        >Open Project</RouterLink
      >
    </div>
    <div class="info">
      <div class="codeBlock">
        <h2>Add</h2>
        <CodeMenu
          defaultTab="cargo-add"
          :snippets="snippets" />
      </div>
      <div class="details">
        <CopyCode :code="project.project_key">Crate</CopyCode>
        <CopyCode
          v-if="project.latest_release"
          :code="project.latest_release"
          >Latest</CopyCode
        >
        <CopyCode
          v-if="project.latest_pre_release"
          :code="project.latest_pre_release"
          >Latest Pre-Release</CopyCode
        >
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Project, ProjectVersion } from "@/types/project";
import type { RepositoryWithStorageName } from "@/types/repository";
import { computed, type PropType } from "vue";
import { createProjectSnippets } from "./CargoRepositoryHelpers";
import CodeMenu from "@/components/core/code/CodeMenu.vue";
import CopyCode from "@/components/core/code/CopyCode.vue";
import { RouterLink } from "vue-router";

const props = defineProps({
  project: {
    type: Object as PropType<Project>,
    required: true,
  },
  version: {
    type: Object as PropType<ProjectVersion>,
    required: false,
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});

// Cargo has no `latest` tag to fall back on the way npm does, so an unknown version becomes the
// wildcard requirement — which resolves to the newest match rather than failing.
const version = computed(() => {
  return (
    props.version?.version ??
    props.project.latest_release ??
    props.project.latest_pre_release ??
    "*"
  );
});
const snippets = computed(() => createProjectSnippets(props.project, version.value));
</script>

<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";

.cargoProject {
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
.projectHeader {
  display: flex;
  align-items: center;
  margin-bottom: 1rem;
  .openProject {
    margin-left: 1rem;
    display: block;
    padding: 0.5rem;
    border: 1px solid gray;
    border-radius: 0.5rem;
    background-color: $primary-30;
    color: white;
    text-decoration: none;
    text-align: end;
  }
}
</style>
