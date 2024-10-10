<template>
  <div
    @click="click"
    class="browseItem"
    data-type="folder">
    <div class="itemAndName">
      <font-awesome-icon icon="fa-solid fa-folder" />
      {{ props.file.name }}
    </div>
  </div>
</template>

<script setup lang="ts">
import router from "@/router";
import { fixCurrentPath, type RawDirectory } from "@/types/browse";
import { type RepositoryWithStorageName } from "@/types/repository";
import { type PropType } from "vue";
import "./browse.scss";

const props = defineProps({
  file: {
    type: Object as PropType<RawDirectory>,
    required: true,
  },
  currentPath: {
    type: String,
    required: true,
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});
const fixedPath = fixCurrentPath(props.currentPath);
const browseRoute = `/browse/${props.repository.id}/${fixedPath}/${props.file.name}`;

function click() {
  router.push(browseRoute);
}
</script>
