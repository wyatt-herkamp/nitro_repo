<template>
  <main v-if="repository">
    <TabsElement>
      <template #header>
        <TabElement id="main"> Main </TabElement>
        <TabElement
          :id="configType"
          v-for="configType in configTypes"
          :key="configType">
          {{ getConfigTitleOrFallback(configType) }}
        </TabElement>
      </template>
      <template #content>
        <TabContent tabId="main">
          <BasicRepositoryInfo :repository="repository" />
        </TabContent>
        <TabContent
          class="tab-content"
          v-for="configType in configComponents"
          :tabId="configType.configName"
          :key="configType.configName">
          <component
            class="config"
            :is="configType.component"
            v-bind="configType.props" />
        </TabContent>
      </template>
    </TabsElement>
  </main>
</template>
<script setup lang="ts">
import BasicRepositoryInfo from "@/components/admin/repository/BasicRepositoryInfo.vue";
import FallBackEditor from "@/components/admin/repository/configs/FallBackEditor.vue";
import TabContent from "@/components/core/tabs/TabContent.vue";
import TabElement from "@/components/core/tabs/TabElement.vue";
import TabsElement from "@/components/core/tabs/TabsElement.vue";
import http from "@/http";
import router from "@/router";
import { repositoriesStore } from "@/stores/repositories";
import {
  getConfigType,
  type ConfigDescription,
  type RepositoryWithStorageName,
} from "@/types/repository";
import { computed, ref, watch } from "vue";
const repositoryTypesStore = repositoriesStore();
const repositoryId = router.currentRoute.value.params.id as string;

const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const configDescriptions = ref<Map<string, ConfigDescription>>(new Map());
const configTypes = ref<string[]>([]);
function getConfigTitleOrFallback(config: string) {
  return configDescriptions.value.get(config)?.name || config;
}
watch(configTypes, async () => {
  for (const config of configTypes.value) {
    await repositoryTypesStore.getConfigDescription(config).then((response) => {
      if (response) {
        configDescriptions.value.set(config, response);
      }
    });
  }
});
const configComponents = computed(() => {
  const configs = configTypes.value.map((config) => {
    const component = getConfigType(config);
    if (component) {
      return {
        component: component.component,
        configName: config,
        props: {
          repository: repositoryId,
        },
      };
    } else {
      return {
        component: FallBackEditor,
        configName: config,
        props: {
          settingName: config,
          repository: repositoryId,
        },
      };
    }
  });
  console.log(configs);
  return configs;
});

async function getRepository() {
  await http.get(`/api/repository/${repositoryId}`).then((response) => {
    repository.value = response.data;
  });
  await http.get(`/api/repository/${repositoryId}/configs`).then((response) => {
    configTypes.value = response.data;
  });
}
getRepository();
</script>
<style scoped lang="scss">
@import "@/assets/styles/theme";
</style>
