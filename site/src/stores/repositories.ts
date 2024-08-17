import { type StorageItem } from './../types/storage'
import { defineStore } from 'pinia'
import { type Ref, ref } from 'vue'
import http from '@/http'
import type { ConfigDescription, RepositoryTypeDescription } from '@/types/repository'
import type { RootSchema } from 'nitro-jsf'
export const repositoriesStore = defineStore(
  'repositories',
  () => {
    const configSchemas: Ref<Map<string, RootSchema>> = ref(new Map())
    const defaultConfigs: Ref<Map<string, any>> = ref(new Map())
    const repositoryTypes: Ref<RepositoryTypeDescription[]> = ref([])
    const configDescriptions: Ref<Map<string, ConfigDescription>> = ref(new Map())
    const storages: Ref<StorageItem[]> = ref([])

    async function getStorages(resetCache: boolean = true): Promise<StorageItem[]> {
      if (resetCache || storages.value.length === 0) {
        return await http
          .get<StorageItem[]>('/api/storage/list')
          .then((response) => {
            storages.value = response.data
            return response.data
          })
          .catch(() => {
            return []
          })
      }
      return storages.value
    }
    async function getRepositoryTypes(
      resetCache: boolean = true
    ): Promise<RepositoryTypeDescription[]> {
      if (resetCache || repositoryTypes.value.length === 0) {
        return await http
          .get<RepositoryTypeDescription[]>('/api/repository/types')
          .then((response) => {
            repositoryTypes.value = response.data
            return response.data
          })
          .catch(() => {
            return []
          })
      }
      return repositoryTypes.value
    }
    async function getConfigDescription(
      type: string,
      ignoreCache: boolean = true
    ): Promise<ConfigDescription | undefined> {
      if (configDescriptions.value.has(type) && !ignoreCache) {
        return configDescriptions.value.get(type)
      }
      return await http
        .get<ConfigDescription>(`/api/repository/config/${type}/description`)
        .then((response) => {
          configDescriptions.value.set(type, response.data)
          return response.data
        })
        .catch(() => {
          return undefined
        })
    }
    async function getConfigSchema(
      type: string,
      ignoreCache: boolean = true
    ): Promise<RootSchema | undefined> {
      if (configSchemas.value.has(type) && !ignoreCache) {
        return configSchemas.value.get(type)
      }
      return await http
        .get<RootSchema>(`/api/repository/config/${type}/schema`)
        .then((response) => {
          configSchemas.value.set(type, response.data)
          return response.data
        })
        .catch(() => {
          return undefined
        })
    }
    async function getDefaultConfig(type: string, ignoreCache: boolean = true) {
      if (defaultConfigs.value.has(type) && !ignoreCache) {
        return defaultConfigs.value.get(type)
      }
      return await http
        .get<any>(`/api/repository/config/${type}/default`)
        .then((response) => {
          defaultConfigs.value.set(type, response.data)
          return response.data
        })
        .catch(() => {
          return undefined
        })
    }
    return {
      repositoryTypes,
      defaultConfigs,
      configSchemas,
      configDescriptions,
      getRepositoryTypes,
      getConfigDescription,
      getConfigSchema,
      getDefaultConfig,
      getStorages
    }
  },
  {
    persist: false
  }
)
