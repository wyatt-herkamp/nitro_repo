<template>
  <RouterLink
    class="browseItem"
    data-type="folder"
    :to="browseRoute">
    <span class="itemAndName">
      <font-awesome-icon
        icon="folder"
        class="itemIcon"
        :style="{ color: 'var(--file-folder)' }" />
      <span class="itemName">{{ file.name }}</span>
    </span>

    <span class="itemMeta itemSize">
      {{ file.number_of_files }} {{ file.number_of_files === 1 ? "item" : "items" }}
    </span>
    <span class="itemMeta itemModified" />
  </RouterLink>
</template>

<script setup lang="ts">
import { fixCurrentPath, type RawDirectory } from "@/types/browse";
import { type RepositoryWithStorageName } from "@/types/repository";
import { type PropType } from "vue";
import { RouterLink } from "vue-router";
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
</script>
