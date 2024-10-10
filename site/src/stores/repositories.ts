import { defineStore } from "pinia";
import { type Ref, ref } from "vue";
import http from "@/http";
import type {
  ConfigDescription,
  RepositoryTypeDescription,
  RepositoryWithStorageName,
} from "@/types/repository";
import type { RootSchema } from "nitro-jsf";
import type { StorageItem } from "@/components/nr/storage/storageTypes";
export const repositoriesStore = defineStore(
  "repositories",
  () => {
    const configSchemas: Ref<Map<string, RootSchema>> = ref(new Map());
    const defaultConfigs: Ref<Map<string, any>> = ref(new Map());
    const repositoryTypes: Ref<RepositoryTypeDescription[]> = ref([]);
    const configDescriptions: Ref<Map<string, ConfigDescription>> = ref(new Map());
    const storages: Ref<StorageItem[]> = ref([]);
    const repositories: Ref<Record<string, RepositoryWithStorageName>> = ref({});
    async function getRepositories(
      resetCache: boolean = true,
    ): Promise<RepositoryWithStorageName[]> {
      if (resetCache || Object.keys(repositories.value).length === 0) {
        return await http
          .get<RepositoryWithStorageName[]>("/api/repository/list")
          .then((response) => {
            repositories.value = {};
            response.data.forEach((repo) => {
              repositories.value[repo.id] = repo;
            });
            return response.data;
          })
          .catch(() => {
            return [];
          });
      }
      return Object.values(repositories.value);
    }
    function getRepositoryFromCache(id: string): RepositoryWithStorageName | undefined {
      return repositories.value[id];
    }
    async function getRepositoryById(
      id: string,
      resetCache: boolean = true,
    ): Promise<RepositoryWithStorageName | undefined> {
      if (resetCache || !repositories.value[id]) {
        await http.get<RepositoryWithStorageName>(`/api/repository/${id}`).then((response) => {
          repositories.value[response.data.id] = response.data;
        });
      }
      return repositories.value[id];
    }
    async function getStorages(resetCache: boolean = true): Promise<StorageItem[]> {
      if (resetCache || storages.value.length === 0) {
        return await http
          .get<StorageItem[]>("/api/storage/list")
          .then((response) => {
            storages.value = response.data;
            return response.data;
          })
          .catch(() => {
            return [];
          });
      }
      return storages.value;
    }
    async function getRepositoryTypes(
      resetCache: boolean = true,
    ): Promise<RepositoryTypeDescription[]> {
      if (resetCache || repositoryTypes.value.length === 0) {
        return await http
          .get<RepositoryTypeDescription[]>("/api/repository/types")
          .then((response) => {
            repositoryTypes.value = response.data;
            return response.data;
          })
          .catch(() => {
            return [];
          });
      }
      return repositoryTypes.value;
    }
    async function getConfigDescription(
      type: string,
      ignoreCache: boolean = true,
    ): Promise<ConfigDescription | undefined> {
      if (configDescriptions.value.has(type) && !ignoreCache) {
        return configDescriptions.value.get(type);
      }
      return await http
        .get<ConfigDescription>(`/api/repository/config/${type}/description`)
        .then((response) => {
          configDescriptions.value.set(type, response.data);
          return response.data;
        })
        .catch(() => {
          return undefined;
        });
    }
    async function getConfigSchema(
      type: string,
      ignoreCache: boolean = true,
    ): Promise<RootSchema | undefined> {
      if (configSchemas.value.has(type) && !ignoreCache) {
        return configSchemas.value.get(type);
      }
      return await http
        .get<RootSchema>(`/api/repository/config/${type}/schema`)
        .then((response) => {
          configSchemas.value.set(type, response.data);
          return response.data;
        })
        .catch(() => {
          return undefined;
        });
    }
    async function getDefaultConfig(type: string, ignoreCache: boolean = true) {
      if (defaultConfigs.value.has(type) && !ignoreCache) {
        return defaultConfigs.value.get(type);
      }
      return await http
        .get<any>(`/api/repository/config/${type}/default`)
        .then((response) => {
          defaultConfigs.value.set(type, response.data);
          return response.data;
        })
        .catch(() => {
          return undefined;
        });
    }
    return {
      repositoryTypes,
      defaultConfigs,
      configSchemas,
      configDescriptions,
      repositories,
      getRepositoryById,
      getRepositoryTypes,
      getConfigDescription,
      getConfigSchema,
      getDefaultConfig,
      getStorages,
      getRepositoryFromCache,
      getRepositories,
    };
  },
  {
    persist: false,
  },
);
