<template>
  <main>
    <div v-if="!error && repositories.length >= 1">
      <RepositoryListInner :repositories="repositories" />
    </div>
  </main>
</template>

<script setup lang="ts">
import http from '@/http'
import { ref } from 'vue'

import type { RepositoryWithStorageName } from '@/types/repository'
import RepositoryListInner from '@/components/admin/repository/RepositoryListInner.vue'

const repositories = ref<RepositoryWithStorageName[]>([])
const error = ref<string | null>(null)

async function fetchRepositories() {
  await http
    .get<RepositoryWithStorageName[]>('/api/repository/list')
    .then((response) => {
      repositories.value = response.data
    })
    .catch((error) => {
      console.error(error)
      error.value = 'Failed to fetch repositories'
    })
}
fetchRepositories()
</script>
