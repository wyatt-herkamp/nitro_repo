<template>
  <main class="container">
    <template v-if="repository">
      <div class="page-header">
        <div class="page-header-text">
          <h1>{{ repository.name }}</h1>
          <BrowseHeader :repository="repository" />
        </div>
        <div class="page-header-actions">
          <NBadge variant="accent">{{ repository.repository_type }}</NBadge>
          <NBadge :variant="repository.visibility === 'Public' ? 'success' : 'neutral'">
            {{ repository.visibility }}
          </NBadge>
        </div>
      </div>

      <BrowseList
        :totalFiles="numberOfFiles"
        :files="files ?? []"
        :currentPath="catchAll"
        :repository="repository" />

      <NEmptyState
        v-if="files && files.length === 0 && numberOfFiles === 0"
        title="This directory is empty"
        icon="folder" />

      <BrowseProject
        v-if="projectResolution"
        :projectResolution="projectResolution"
        :repository="repository" />
    </template>

    <div
      v-else-if="loadError"
      class="loadError">
      <font-awesome-icon icon="triangle-exclamation" />
      <span>{{ loadError }}</span>
    </div>

    <div
      v-else
      class="loading">
      <SpinnerElement size="lg" />
    </div>
  </main>
</template>

<script setup lang="ts">
import BrowseHeader from "@/components/nr/repository/browse/BrowseHeader.vue";
import BrowseList from "@/components/nr/repository/browse/BrowseList.vue";
import BrowseProject from "@/components/nr/repository/project/BrowseProject.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import NEmptyState from "@/components/core/ui/NEmptyState.vue";
import SpinnerElement from "@/components/spinner/SpinnerElement.vue";
import { websocketPath } from "@/config";
import router from "@/router";
import { useRepositoryStore } from "@/stores/repositories";
import type { ProjectResolution, RawBrowseFile, WSBrowseResponse } from "@/types/browse";
import { type RepositoryWithStorageName } from "@/types/repository";
import { onBeforeUnmount, ref, watch } from "vue";

const repoStore = useRepositoryStore();
const repositoryId = ref(router.currentRoute.value.params.id as string);
const catchAll = ref((router.currentRoute.value.params.catchAll as string) ?? "");

const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const files = ref<Array<RawBrowseFile> | undefined>(undefined);
const projectResolution = ref<ProjectResolution | undefined>(undefined);
const numberOfFiles = ref(0);
const loadError = ref<string | undefined>(undefined);

const websocket = new WebSocket(websocketPath(`api/repository/browse-ws/${repositoryId.value}`));

onBeforeUnmount(() => {
  websocket.close();
});

websocket.onopen = () => {
  changeDirectory(catchAll.value);
};

websocket.onmessage = (event) => {
  const message: WSBrowseResponse = JSON.parse(event.data);
  if (message.type === "DirectoryItem") {
    files.value = [...(files.value ?? []), message.data];
  } else if (message.type === "OpenedDirectory") {
    numberOfFiles.value = message.data.number_of_files;
    files.value = [];
    projectResolution.value = message.data.project_resolution;
  }
};

async function loadRepository() {
  try {
    repository.value = await repoStore.getRepositoryById(repositoryId.value);
    if (repository.value === undefined) {
      loadError.value = "That repository does not exist, or you cannot see it.";
    }
  } catch {
    loadError.value = "Could not load this repository.";
  }
}

loadRepository();

function changeDirectory(path: string) {
  websocket.send(JSON.stringify({ type: "ListDirectory", data: path }));
}

watch(
  () => router.currentRoute.value.params.catchAll,
  () => {
    catchAll.value = (router.currentRoute.value.params.catchAll as string) ?? "";
    files.value = undefined;
    numberOfFiles.value = 0;
    projectResolution.value = undefined;
    changeDirectory(catchAll.value);
  },
);
</script>

<style scoped lang="scss">
.loading,
.loadError {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-16) var(--space-4);
  color: var(--text-muted);
}

.loadError {
  color: var(--danger);
}
</style>
