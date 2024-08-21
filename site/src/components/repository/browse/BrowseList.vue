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
import { fixCurrentPath, type RawBrowseFile } from '@/types/browse'
import type { RepositoryWithStorageName } from '@/types/repository'
import { computed, type PropType } from 'vue'
import BrowseFile from './BrowseEntry.vue'
import BrowseEntry from './BrowseEntry.vue'

const props = defineProps({
  files: {
    type: Array as PropType<RawBrowseFile[]>,
    required: true
  },
  currentPath: {
    type: String,
    required: true
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true
  }
})

function joinLink(file: RawBrowseFile) {
  const newPath = fixCurrentPath(props.currentPath)
  console.log(
    `Repository ID: ${props.repository.id}, Current Path: ${props.currentPath} (Fixed: ${newPath}), File Name: ${file.value.name}`
  )

  return `/browse/${props.repository.id}/${newPath}/${file.value.name}`
}
</script>
<style lang="scss" scoped>
@import '@/assets/styles/theme.scss';
#browseList {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
</style>
