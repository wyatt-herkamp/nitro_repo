<template>
  <div class="mavenProject">
    <div class="info">
      <div class="codeBlock">
        <h2>Project Info</h2>
        <CodeMenu
          defaultTab="maven"
          :snippets="snippets" />
      </div>
      <div class="details">
        <CopyCode :code="project.scope || 'undefined'">Group Id</CopyCode>
        <CopyCode :code="project.name || 'undefined'">Artifact Id</CopyCode>
        <CopyCode
          v-if="project.latest_pre_release"
          :code="project.latest_pre_release"
          >Latest Pre-Release</CopyCode
        >
        <CopyCode
          v-if="project.latest_release"
          :code="project.latest_release"
          >Latest Release</CopyCode
        >
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Project, ProjectVersion } from "@/types/project";
import type { RepositoryWithStorageName } from "@/types/repository";
import { computed, type PropType } from "vue";
import { createProjectSnippets } from "./MavenRepositoryHelpers";
import CodeMenu from "@/components/core/code/CodeMenu.vue";
import CopyCode from "@/components/core/code/CopyCode.vue";

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
const version = computed(() => {
  if (props.version) {
    console.debug("Using version from props");
    return props.version.version;
  } else if (props.project.latest_release) {
    console.debug("Using latest release");
    return props.project.latest_release;
  } else if (props.project.latest_pre_release) {
    console.debug("Using latest pre-release");
    return props.project.latest_pre_release;
  } else {
    return "latest";
  }
});
const snippets = createProjectSnippets(props.project, version.value);
</script>

<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";

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
