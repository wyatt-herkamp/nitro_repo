<template>
  <div v-if="!error && repositories.length >= 1">
    <RepositoryListInner :repositories="repositories" />
  </div>
</template>

<script setup lang="ts">
import http from '@/http'
import { ref } from 'vue'

import type { RepositoryWithStorageName } from '@/types/repository'
import RepositoryListInner from './RepositoryListInner.vue'

const repositories = ref<RepositoryWithStorageName[]>([])
const error = ref<string | null>(null)

async function fetchUsers() {
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
fetchUsers()
</script>
