<template>
  <div id="browseList">
    <BrowseEntry
      v-for="file in props.files"
      :key="file.value.name"
      :file="file"
      :currentPath="currentPath"
      :repository="repository" />
  </div>
</template>
<script setup lang="ts">
import { type RawBrowseFile } from "@/types/browse";
import type { RepositoryWithStorageName } from "@/types/repository";

import BrowseEntry from "./BrowseEntry.vue";
import type { PropType } from "vue";

const props = defineProps({
  files: {
    type: Array as PropType<RawBrowseFile[]>,
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
</script>
<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";
#browseList {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
</style>
