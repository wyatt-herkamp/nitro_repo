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
          {{ configType }}
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
          <form id="repository">
            <h2>Repository Info</h2>

            <TwoByFormBox>
              <TextInput v-model="repository.name" required disabled> Name </TextInput>
              <TextInput v-model="repository.repository_type" required disabled>
                Repository Type
              </TextInput>
            </TwoByFormBox>
            <TwoByFormBox>
              <TextInput v-model="repository.storage_name" required disabled>
                Storage Name
              </TextInput>
              <TextInput v-model="repository.storage_id" required disabled>
                Storage Id Type
              </TextInput>
            </TwoByFormBox>
          </form>
        </div>
      </div>
    </div>
  </main>
</template>
<script setup lang="ts">
import FallBackEditor from '@/components/admin/repository/configs/FallBackEditor.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import TwoByFormBox from '@/components/form/TwoByFormBox.vue'
import http from '@/http'
import router from '@/router'
import { getConfigType, type RepositoryWithStorageName } from '@/types/repository'
import { storageTypes, type StorageItem } from '@/types/storage'
import { computed, ref, watch } from 'vue'

const repositoryId = router.currentRoute.value.params.id as string
const currentConfig = ref<string | undefined>('main')
watch(currentConfig, (newValue) => {
  console.log('New Value ' + newValue)
})
const repository = ref<RepositoryWithStorageName | undefined>(undefined)
const configTypes = ref<string[]>([])

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
  justify-content: space-between;
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
