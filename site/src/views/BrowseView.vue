<template>
  <main v-if="repository">
    <BrowseHeader :repository="repository" />
    <div v-if="files">
      <div class="browse">
        <BrowseList
          :totalFiles="numberOfFiles"
          :files="files"
          :currentPath="catchAll"
          :repository="repository" />
      </div>
      <div v-if="projectResolution">
        <BrowseProject
          :projectResolution="projectResolution"
          :repository="repository" />
      </div>
    </div>
    <div v-else>
      <p>Loading...</p>
    </div>
  </main>
</template>
<script setup lang="ts">
import BrowseHeader from "@/components/nr/repository/browse/BrowseHeader.vue";
import BrowseList from "@/components/nr/repository/browse/BrowseList.vue";
import BrowseProject from "@/components/nr/repository/project/BrowseProject.vue";
import { websocketPath } from "@/config";

import router from "@/router";
import { useRepositoryStore } from "@/stores/repositories";
import type { ProjectResolution, RawBrowseFile, WSBrowseResponse } from "@/types/browse";
import { type RepositoryWithStorageName } from "@/types/repository";
import { onBeforeUnmount, ref, watch } from "vue";
const repoStore = useRepositoryStore();
const repositoryId = ref(router.currentRoute.value.params.id as string);
const catchAll = ref(router.currentRoute.value.params.catchAll as string);
console.log(`Browsing repository ${repositoryId.value} with catchAll ${catchAll.value}`);

const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const websocket = new WebSocket(websocketPath(`api/repository/browse-ws/${repositoryId.value}`));
onBeforeUnmount(() => {
  console.log("Closing websocket");
  websocket.close();
});
websocket.onopen = () => {
  console.log("Websocket opened");
  changeDirectory(catchAll.value);
};
websocket.onmessage = (event) => {
  const message: WSBrowseResponse = JSON.parse(event.data);
  console.log(`Received message`, message);
  if (message.type === "DirectoryItem") {
    if (files.value === undefined) {
      files.value = [];
    }
    console.log("Adding file", message.data);
    files.value.push(message.data);
  } else if (message.type === "OpenedDirectory") {
    console.log("Opened Directory", message.data);
    numberOfFiles.value = message.data.number_of_files;
    files.value = [];
    projectResolution.value = message.data.project_resolution;
  } else {
    console.log(`Unknown message type`, message);
  }
};
const files = ref<RawBrowseFile[] | undefined>(undefined);
const projectResolution = ref<ProjectResolution | undefined>(undefined);
async function loadRepository() {
  console.log(`Loading repository ${repositoryId.value}`);
  repoStore.getRepositoryById(repositoryId.value).then((response) => {
    repository.value = response;
    console.log("Loaded Repository" + response);
  });
}
const numberOfFiles = ref(0);

loadRepository();

function changeDirectory(path: string) {
  console.log(`Changing directory to ${path}`);
  websocket.send(JSON.stringify({ type: "ListDirectory", data: path }));
}
watch(
  () => router.currentRoute.value.params.catchAll,
  () => {
    console.log("CatchAll changed");
    catchAll.value = router.currentRoute.value.params.catchAll as string;
    files.value = undefined;
    projectResolution.value = undefined;
    changeDirectory(catchAll.value);
  },
);
</script>
