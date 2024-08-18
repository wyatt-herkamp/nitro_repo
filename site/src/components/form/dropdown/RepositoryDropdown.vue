<template>
  <select id="dropdown" v-model="value">
    <option v-for="option in repositoryEntries" :key="option.value" :value="option.value">
      {{ option.label }}
    </option>
  </select>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { type RepositoryWithStorageName } from '@/types/repository'
import { repositoriesStore } from '@/stores/repositories'

const repositories = ref<RepositoryWithStorageName[]>([])
const repoStore = repositoriesStore()
repoStore.getRepositories().then((repos) => {
  repositories.value = repos
})

const repositoryEntries = computed(() => {
  return repositories.value.map((repo) => ({
    label: `${repo.name} (${repo.storage_name})`,
    value: repo.id
  }))
})

const value = defineModel<string>({
  required: true
})
</script>
