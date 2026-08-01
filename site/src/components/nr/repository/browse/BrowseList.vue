<template>
  <div id="browseList">
    <BrowseEntry
      v-for="file in sortedFiles"
      :key="file.value.name"
      :file="file"
      :currentPath="currentPath"
      :repository="repository" />
    <SkeletonEntry
      v-for="i in skeletons"
      :key="i"
      :index="i" />
  </div>
</template>
<script setup lang="ts">
import { type RawBrowseFile } from "@/types/browse";
import type { RepositoryWithStorageName } from "@/types/repository";

import BrowseEntry from "./BrowseEntry.vue";
import { computed, type PropType } from "vue";
import SkeletonEntry from "./SkeletonEntry.vue";

const props = defineProps({
  files: {
    type: Array as PropType<RawBrowseFile[]>,
    required: true,
  },
  totalFiles: {
    type: Number,
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
const skeletons = computed(() => {
  const skeletonsArray = [];
  for (let i = props.files.length; i < props.totalFiles; i++) {
    skeletonsArray.push(i);
  }
  return skeletonsArray;
});
const sortedFiles = computed(() => {
  // A copy: `Array.prototype.sort` mutates, and sorting the prop array in place mutates the parent's
  // state from inside a computed.
  return [...props.files].sort((a, b) => {
    if (a.type === "Directory" && b.type === "File") {
      return -1;
    } else if (a.type === "File" && b.type === "Directory") {
      return 1;
    } else {
      return a.value.name.localeCompare(b.value.name);
    }
  });
});
</script>
<style lang="scss" scoped>
// Rows sit flush against each other, separated by their own hairline borders — a file listing, not
// a stack of cards.
#browseList {
  display: flex;
  flex-direction: column;
  background-color: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
}
</style>
