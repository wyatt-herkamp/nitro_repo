<template>
  <div class="browsePath">
    <span @click="goToHome()" class="browsePathPlace" id="base">
      {{ props.repository.storage_name }} / {{ props.repository.name }}
    </span>
    <span
      v-for="(path, index) in pathElements"
      :key="index"
      class="browsePathPlace"
      :data-clickable="path.path !== undefined"
      @click="goTo(path.path)">
      {{ path.name }}
    </span>
  </div>
</template>

<script setup lang="ts">
import router from '@/router'
import type { RepositoryWithStorageName } from '@/types/repository'
import { computed, ref, watch, type PropType } from 'vue'

const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true
  }
})
function goTo(path?: string) {
  if (path) {
    router.push(`/browse/${props.repository.id}/${path}`)
  }
}
function goToHome() {
  router.push(`/browse/${props.repository.id}`)
}
interface PathElement {
  name: string
  path?: string
}
const pathElements = ref<PathElement[]>([])
function buildPath() {
  pathElements.value = []
  const pathSplit = (router.currentRoute.value.params.catchAll as string).split('/')
  let path = ''
  for (let element of pathSplit) {
    pathElements.value.push({
      name: '/'
    })
    path += element + '/'
    pathElements.value.push({
      name: element,
      path: path
    })
  }
  console.log(pathElements.value)
}
watch(
  () => router.currentRoute.value.params.catchAll,
  () => {
    buildPath()
  }
)
buildPath()
</script>
<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
.browsePath {
  display: flex;
  gap: 0.5rem;
  padding: 1rem;
  background-color: $background;
  span {
    display: flex;
    align-items: center;
  }
}
.browsePathPlace[data-clickable='true'] {
  cursor: pointer;
  &:hover {
    text-decoration: underline;
  }
}
#base {
  font-weight: bold;
  &:hover {
    cursor: pointer;
  }
}
</style>
