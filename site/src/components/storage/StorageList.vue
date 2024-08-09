<template>
  <div v-if="!error && storages.length >= 1">
    <StorageListInner :storages="storages" />
  </div>
</template>

<script setup lang="ts">
import http from '@/http'
import { ref } from 'vue'
import StorageListInner from './StorageListInner.vue'
import type { StorageItem } from '@/types/storage'

const storages = ref<StorageItem[]>([])
const error = ref<string | null>(null)

async function fetchUsers() {
  await http
    .get<StorageItem[]>('/api/storage/list')
    .then((response) => {
      storages.value = response.data
    })
    .catch((error) => {
      console.error(error)
      error.value = 'Failed to fetch users'
    })
}
fetchUsers()
</script>
