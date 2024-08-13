<template>
  <main v-if="repository">
    <div class="tabs">
      <div class="tabs-header">
        <div class="tab" :data-active="currentConfig === 'main'" @click="currentConfig = 'main'">
          Main
        </div>
        <div
          class="tab"
          v-for="configType in configTypes"
          :key="configType"
          :data-active="currentConfig === configType"
          @click="currentConfig = configType"
        >
          {{ getConfigTitleOrFallback(configType) }}
        </div>
      </div>
      <div class="tabs-content">
        <div
          class="tab-content"
          v-for="configType in configComponents"
          :key="configType.configName"
          :data-active="currentConfig === configType.configName"
        >
          <component class="config" :is="configType.component" v-bind="configType.props" />
        </div>
        <div class="tab-content" :data-active="currentConfig === 'main'">
          <BasicRepositoryInfo :repository="repository" />
        </div>
      </div>
    </div>
  </main>
</template>
<script setup lang="ts">
import BasicRepositoryInfo from '@/components/admin/repository/BasicRepositoryInfo.vue'
import FallBackEditor from '@/components/admin/repository/configs/FallBackEditor.vue'
import http from '@/http'
import router from '@/router'
import { repositoriesStore } from '@/stores/repositories'
import {
  getConfigType,
  type ConfigDescription,
  type RepositoryWithStorageName
} from '@/types/repository'
import { storageTypes, type StorageItem } from '@/types/storage'
import { computed, ref, watch } from 'vue'
const repositoryTypesStore = repositoriesStore()
const repositoryId = router.currentRoute.value.params.id as string
const currentConfig = ref<string | undefined>('main')
watch(currentConfig, (newValue) => {
  console.log('New Value ' + newValue)
})
const repository = ref<RepositoryWithStorageName | undefined>(undefined)
const configDescriptions = ref<Map<string, ConfigDescription>>(new Map())
const configTypes = ref<string[]>([])
function getConfigTitleOrFallback(config: string) {
  return configDescriptions.value.get(config)?.name || config
}
watch(configTypes, async () => {
  for (const config of configTypes.value) {
    await repositoryTypesStore.getConfigDescription(config).then((response) => {
      if (response) {
        configDescriptions.value.set(config, response)
      }
    })
  }
})
const configComponents = computed(() => {
  const configs = configTypes.value.map((config) => {
    const component = getConfigType(config)
    if (component) {
      return {
        component: component.component,
        configName: config,
        props: {
          repository: repositoryId
        }
      }
    } else {
      return {
        component: FallBackEditor,
        configName: config,
        props: {
          settingName: config,
          repository: repositoryId
        }
      }
    }
  })
  console.log(configs)
  return configs
})

async function getRepository() {
  await http.get(`/api/repository/${repositoryId}`).then((response) => {
    repository.value = response.data
  })
  await http.get(`/api/repository/${repositoryId}/configs`).then((response) => {
    configTypes.value = response.data
  })
}
getRepository()
</script>
<style scoped lang="scss">
@import '@/assets/styles/theme';
.tabs {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 90vh;

  background-color: $background-30;
}
@media screen and (max-width: 800px) {
  .tabs-header {
    flex-direction: column;
  }
}
.tabs-header {
  display: flex;
  gap: 1rem;
  width: 100%;
  background-color: $primary-30;
}
.tab {
  padding: 1rem;
  cursor: pointer;
  border-radius: 0.5rem 0.5rem 0 0;
  border: 1px solid $primary-50;
  &:hover {
    background-color: $accent;
    color: white;
  }
}
.tab[data-active='true'] {
  background-color: $accent;
  color: white;
  cursor: default;
}
.tab-content {
  display: flex;
  width: 100%;
  height: 100%;
  margin: auto 0;
  border: 1px solid $primary-50;
  padding: 1rem;
}
.tab-content[data-active='false'] {
  display: none;
}
.tabs-content[data-active='true'] {
  display: block;
}
.config {
  width: 100%;
  height: 100%;
  margin: auto 0;
}
#repository {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  margin: 0 auto;
}
</style>
