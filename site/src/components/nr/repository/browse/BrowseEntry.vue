<template>
  <BrowseFile
    v-if="props.file.type === 'File'"
    :file="props.file.value as RawFile"
    :currentPath="props.currentPath"
    :repository="props.repository" />
  <BrowseFolder
    v-else-if="props.file.type === 'Directory'"
    :file="props.file.value as RawDirectory"
    :currentPath="props.currentPath"
    :repository="props.repository" />
</template>

<script setup lang="ts">
import { apiURL } from '@/config'
import router from '@/router'
import { fixCurrentPath, type RawBrowseFile, type RawDirectory, type RawFile } from '@/types/browse'
import { createRepositoryRoute, type RepositoryWithStorageName } from '@/types/repository'
import { computed, type PropType } from 'vue'
import BrowseFolder from './BrowseFolder.vue'
import BrowseFile from './BrowseFile.vue'

const props = defineProps({
  file: {
    type: Object as PropType<RawBrowseFile>,
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
console.log(props.file)
</script>
