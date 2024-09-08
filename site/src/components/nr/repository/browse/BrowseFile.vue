<template>
  <div @click="click" class="browseItem" data-type="file">
    <div class="itemAndName">
      <font-awesome-icon :icon="fileIcon" />
      {{ props.file.name }}
    </div>

    <div>
      {{ props.file.modified }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { apiURL } from '@/config'
import router from '@/router'
import { fixCurrentPath, type RawBrowseFile, type RawFile } from '@/types/browse'
import { createRepositoryRoute, type RepositoryWithStorageName } from '@/types/repository'
import { computed, type PropType } from 'vue'
import './browse.scss'
const props = defineProps({
  file: {
    type: Object as PropType<RawFile>,
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

const fixedPath = fixCurrentPath(props.currentPath)
const repositoryURL = createRepositoryRoute(props.repository, `${fixedPath}/${props.file.name}`)

const fileIcon = computed(() => {
  // TODO: More icons
  return 'fa-solid fa-file'
})
function click() {
  window.open(repositoryURL, '_blank')
}
</script>
