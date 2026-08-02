<template>
  <div class="dockerProject">
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
        <h2>Pull</h2>
        <CodeMenu
          defaultTab="pull"
          :snippets="snippets" />
      </div>
      <div class="details">
        <CopyCode :code="project.project_key">Image</CopyCode>
        <CopyCode
          v-if="project.latest_release"
          :code="project.latest_release"
          >Latest Tag</CopyCode
        >
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Project, ProjectVersion } from "@/types/project";
import type { RepositoryHostname, RepositoryWithStorageName } from "@/types/repository";
import { computed, ref, type PropType } from "vue";
import { createProjectSnippets } from "./DockerRepositoryHelpers";
import CodeMenu from "@/components/core/code/CodeMenu.vue";
import CopyCode from "@/components/core/code/CopyCode.vue";
import { RouterLink } from "vue-router";
import http from "@/http";

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

const hostname = ref<string | undefined>(undefined);
http
  .get<RepositoryHostname[]>(`/api/repository/${props.repository.id}/hostnames`)
  .then((response) => {
    hostname.value = response.data[0]?.hostname;
  })
  .catch(() => {
    hostname.value = undefined;
  });

// A Docker tag *is* the version, and `latest` is the conventional default rather than something the
// registry resolves — so it is only a fallback when nothing more specific is known.
const tag = computed(() => {
  return props.version?.version ?? props.project.latest_release ?? "latest";
});
const snippets = computed(() =>
  createProjectSnippets(props.repository, props.project, tag.value, hostname.value),
);
</script>

<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";

.dockerProject {
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
