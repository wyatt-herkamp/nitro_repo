<template>
  <main v-if="repository">
    <BrowseHeader :repository="repository" />
    <div v-if="lastResponse">
      <div class="browse">
        <BrowseList :files="lastResponse.files" :currentPath="catchAll" :repository="repository" />
      </div>
      <div v-if="lastResponse.project">
        <BrowseProject :project="new Project(lastResponse.project)" :repository="repository" />
      </div>
    </div>
  </main>
</template>
<script setup lang="ts">
import BrowseHeader from '@/components/repository/browse/BrowseHeader.vue'
import BrowseList from '@/components/repository/browse/BrowseList.vue'
import BrowseProject from '@/components/repository/project/BrowseProject.vue'
import http from '@/http'
import router from '@/router'
import { repositoriesStore } from '@/stores/repositories'
import type { RawBrowseResponse } from '@/types/browse'
import { Project } from '@/types/project'
import { type RepositoryWithStorageName } from '@/types/repository'
import { ref, watch } from 'vue'
const repoStore = repositoriesStore()
const repositoryId = ref(router.currentRoute.value.params.id as string)
const catchAll = ref(router.currentRoute.value.params.catchAll as string)
console.log(`Browsing repository ${repositoryId.value} with catchAll ${catchAll.value}`)

const repository = ref<RepositoryWithStorageName | undefined>(undefined)
const lastResponse = ref<RawBrowseResponse | undefined>(undefined)
async function loadRepository() {
  console.log(`Loading repository ${repositoryId.value}`)
  await repoStore.getRepositoryById(repositoryId.value).then((response) => {
    repository.value = response
    console.log('Loaded Repository' + response)
  })
}

async function browse() {
  const apiRoute = `/api/repository/browse/${repositoryId.value}/${catchAll.value}`
  console.log(
    `Browsing repository ${repositoryId.value} with catchAll ${catchAll.value} using ${apiRoute}`
  )
  http.get<RawBrowseResponse>(apiRoute).then((response) => {
    lastResponse.value = response.data
    console.log('Loaded Browse Response' + response.data)
  })
}
loadRepository()

browse()
watch(
  () => router.currentRoute.value.params.catchAll,
  () => {
    console.log('CatchAll changed')
    catchAll.value = router.currentRoute.value.params.catchAll as string
    browse()
  }
)
</script>
