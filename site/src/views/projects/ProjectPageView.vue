<template>
  <main v-if="project">
    <div class="primaryInfo">
      <h1>{{ project.name }}</h1>
      <RouterLink
        class="openBrowse"
        :to="{
          name: 'repository',
          params: {
            id: project.repository_id,
          },
        }">
        Open Repository
      </RouterLink>
      <RouterLink
        class="openBrowse"
        :to="{
          name: 'Browse',
          params: {
            id: project.repository_id,
            catchAll: project.storage_path,
          },
        }">
        Open Browse
      </RouterLink>
      <CopyURL :code="url"> </CopyURL>
      <div v-if="repositoryType">
        <RepositoryIcon
          :name="repositoryType.name"
          v-for="icon in repositoryType.icons"
          :key="icon.name"
          :icon="icon" />
      </div>
    </div>
    <div class="content">
      <div v-if="repositoryHandler && repositoryHandler.fullProjectComponent">
        <component
          :is="repositoryHandler.fullProjectComponent.component"
          :project="project"
          :repository="repository" />
      </div>
      <div v-else-if="repositoryHandler && repositoryHandler.projectComponent">
        <component
          :is="repositoryHandler.projectComponent.component"
          :project="project"
          :repository="repository" />
      </div>
      <div v-else-if="repositoryHandler">
        <p>Project type not supported</p>
      </div>
      <div v-else>
        <p>This repository has not been defined in the frontend</p>
      </div>
    </div>
  </main>
  <ErrorOnRequest
    v-else-if="error"
    :error="error"
    :errorCode="errorCode" />

  <main v-else-if="!project">
    <p>Project Not Found</p>
  </main>
</template>

<script setup lang="ts">
import CopyURL from "@/components/core/code/CopyCode.vue";
import ErrorOnRequest from "@/components/ErrorOnRequest.vue";
import RepositoryIcon from "@/components/nr/repository/RepositoryIcon.vue";

import router from "@/router";
import { useProjectStore } from "@/stores/project_store";
import { repositoriesStore } from "@/stores/repositories";
import type { Project } from "@/types/project";
import {
  createRepositoryRoute,
  findRepositoryType,
  type FrontendRepositoryType,
  type RepositoryWithStorageName,
} from "@/types/repository";
import { computed, ref, watch } from "vue";
const projectId = router.currentRoute.value.params.projectId as string;
const repositoryId = ref<string | undefined>(undefined);
const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const project = ref<Project | undefined>(undefined);
const error = ref<string | null>(null);
const errorCode = ref<number | undefined>(undefined);
const repoStore = repositoriesStore();
const projectStore = useProjectStore();
const repositoryHandler = ref<FrontendRepositoryType | undefined>(undefined);
const repositoryType = computed(() => {
  if (repository.value) {
    return findRepositoryType(repository.value.repository_type);
  }
  return undefined;
});
const url = computed(() => {
  if (!repository.value) {
    return "";
  }
  return createRepositoryRoute(repository.value);
});

async function fetchProject() {
  await projectStore.getProjectById(projectId).then((response) => {
    project.value = response;
    if (project.value) {
      repositoryId.value = project.value.repository_id;
      console.debug(`Project ${projectId} is in repository ${repositoryId.value}`);
    }
  });
}
watch(repositoryId, () => {
  if (repositoryId.value) {
    repoStore.getRepositoryById(repositoryId.value).then((response) => {
      repository.value = response;
      if (repository.value) {
        repositoryHandler.value = findRepositoryType(repository.value.repository_type);
      }
    });
  }
});
fetchProject();
</script>
<style scoped lang="scss">
@import "@/assets/styles/theme.scss";
main {
  margin: 0 auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.primaryInfo {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  flex-wrap: wrap;

  align-items: center;
  border-bottom: 1px solid gray;
  h1 {
    margin: 0;
  }
}
.content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
main[data-has-page="true"] {
  margin: 0 10%;
}
#page {
  flex-grow: 3;
  width: 100%;
  border-bottom: 1px solid gray;
  background-color: $background;
}
@media screen and (max-width: 1200px) {
  main[data-has-page="true"] {
    margin: 0 0;
  }
  #page {
    border-bottom: none;
  }
}
.openBrowse {
  display: block;
  padding: 0.5rem;
  border: 1px solid gray;
  border-radius: 0.5rem;
  background-color: $primary-30;
  color: white;
  text-decoration: none;
  text-align: end;
}
</style>
