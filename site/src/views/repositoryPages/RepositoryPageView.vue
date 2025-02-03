<template>
  <main
    v-if="repository"
    :data-has-page="repositoryPage != undefined">
    <div class="primaryInfo">
      <h1>{{ repository.storage_name }}/{{ repository.name }}</h1>
      <RouterLink
        class="openBrowse"
        :to="{
          name: 'Browse',
          params: { id: repository.id, catchAll: '' },
        }">
        Browse
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
      <div id="page">
        <RepositoryPageViewer
          v-if="repositoryPage"
          :repository="repository"
          :page="repositoryPage" />
      </div>
      <div id="helper">
        <RepositoryHelper :repository="repository" />
      </div>
    </div>
  </main>
  <ErrorOnRequest
    v-else-if="error"
    :error="error"
    :errorCode="errorCode" />
</template>

<script setup lang="ts">
import CopyURL from "@/components/core/code/CopyCode.vue";
import ErrorOnRequest from "@/components/ErrorOnRequest.vue";
import RepositoryHelper from "@/components/nr/repository/RepositoryHelper.vue";
import RepositoryIcon from "@/components/nr/repository/RepositoryIcon.vue";
import RepositoryPageViewer from "@/components/nr/repository/RepositoryPageViewer.vue";
import http from "@/http";

import router from "@/router";
import { useRepositoryStore } from "@/stores/repositories";
import {
  createRepositoryRoute,
  findRepositoryType,
  type RepositoryPage,
  type RepositoryWithStorageName,
} from "@/types/repository";
import { computed, ref } from "vue";
const repoStore = useRepositoryStore();

const repositoryId = ref<string | undefined>(undefined);

const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const repositoryPage = ref<RepositoryPage | undefined>(undefined);
const error = ref<string | null>(null);
const errorCode = ref<number | undefined>(undefined);
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
async function fetchRepository() {
  if (!repositoryId.value) {
    console.error("No repository id");
    return;
  }
  await repoStore.getRepositoryById(repositoryId.value).then((response) => {
    repository.value = response;
    console.log(repository.value);
  });
  await http
    .get<RepositoryPage>(`/api/repository/page/${repositoryId.value}`)
    .then((response) => {
      console.log(response.data);
      repositoryPage.value = response.data;
    })
    .catch((error) => {
      console.error(error);
      errorCode.value = error.response.status;
      error.value = "Failed to fetch repository";
    });
}
console.log(router.currentRoute.value.params);
if (router.currentRoute.value.params.repositoryId) {
  repositoryId.value = router.currentRoute.value.params.repositoryId as string;
  console.debug(`Fetching repository ${repositoryId.value}`);
  fetchRepository();
} else if (
  router.currentRoute.value.params.storageName &&
  router.currentRoute.value.params.repositoryName
) {
  console.debug(
    `Fetching repository by names ${router.currentRoute.value.params.storageName}/${router.currentRoute.value.params.repositoryName}`,
  );
  repoStore
    .getRepositoryIdByNames(
      router.currentRoute.value.params.storageName as string,
      router.currentRoute.value.params.repositoryName as string,
    )
    .then((response) => {
      if (response === null) {
        error.value = "Repository not found";
        return;
      }
      repositoryId.value = response;
      fetchRepository();
    });
}
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
