<template>
  <div v-if="!error && storages.length >= 1">
    <StorageListInner :storages="storages" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import StorageListInner from './StorageListInner.vue'
import type { StorageItem } from '@/types/storage'
import { repositoriesStore } from '@/stores/repositories'

const storages = ref<StorageItem[]>([])
const error = ref<string | null>(null)
const repositoriesTypesStore = repositoriesStore()
async function fetchUsers() {
  await repositoriesTypesStore
    .getStorages()
    .then((response) => {
      storages.value = response
    })
    .catch((error) => {
      console.error(error)
      error.value = 'Failed to fetch storages'
    })
}
fetchUsers()
</script>
