<template>
  <main v-if="!error && repositories.length >= 1">
    <PublicRepositoryList :repositories="repositories" />
  </main>
  <ErrorOnRequest v-else-if="error" :error="error" />
</template>

<script setup lang="ts">
import ErrorOnRequest from '@/components/ErrorOnRequest.vue'
import PublicRepositoryList from '@/components/nr/repository/PublicRepositoryList.vue'
import { repositoriesStore } from '@/stores/repositories'
import type { RepositoryWithStorageName } from '@/types/repository'
import { ref } from 'vue'

const repositories = ref<RepositoryWithStorageName[]>([])
const error = ref<string | null>(null)
const repoStore = repositoriesStore()
async function getRepositories() {
  await repoStore.getRepositories().then((response) => {
    repositories.value = response
    console.log(repositories.value)
  })
}
getRepositories()
</script>
