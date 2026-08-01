<template>
  <main class="container">
    <template v-if="repository">
      <div class="page-header">
        <div class="page-header-text">
          <NBreadcrumb :items="crumbs" />
          <h1>{{ repository.name }}</h1>
        </div>
        <div class="page-header-actions">
          <NButton
            icon="folder"
            :to="{ name: 'Browse', params: { id: repository.id } }">
            Browse
          </NButton>
        </div>
      </div>

      <TabsElement default-tab="main">
        <template #header>
          <TabElement id="main">Main</TabElement>
          <TabElement
            v-for="configType in configTypes"
            :id="configType"
            :key="configType">
            {{ getConfigTitleOrFallback(configType) }}
          </TabElement>
        </template>
        <template #content>
          <TabContent tabId="main">
            <BasicRepositoryInfo
              :repository="repository"
              @updated="onUpdated" />
          </TabContent>
          <TabContent
            v-for="configType in configComponents"
            :key="configType.configName"
            :tabId="configType.configName">
            <component
              :is="configType.component"
              v-bind="configType.props" />
          </TabContent>
        </template>
      </TabsElement>
    </template>

    <div
      v-else-if="loadError"
      class="loadError">
      {{ loadError }}
    </div>

    <div
      v-else
      class="loading">
      <SpinnerElement size="lg" />
    </div>
  </main>
</template>

<script setup lang="ts">
import BasicRepositoryInfo from "@/components/admin/repository/BasicRepositoryInfo.vue";
import FallBackEditor from "@/components/admin/repository/configs/FallBackEditor.vue";
import TabContent from "@/components/core/tabs/TabContent.vue";
import TabElement from "@/components/core/tabs/TabElement.vue";
import TabsElement from "@/components/core/tabs/TabsElement.vue";
import NBreadcrumb, { type Crumb } from "@/components/core/ui/NBreadcrumb.vue";
import NButton from "@/components/core/ui/NButton.vue";
import SpinnerElement from "@/components/spinner/SpinnerElement.vue";
import http from "@/http";
import router from "@/router";
import { useRepositoryStore } from "@/stores/repositories";
import {
  getConfigType,
  type ConfigDescription,
  type RepositoryWithStorageName,
} from "@/types/repository";
import { computed, ref, watch } from "vue";

const repositoryTypesStore = useRepositoryStore();
const repositoryId = router.currentRoute.value.params.id as string;

const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const configDescriptions = ref<Map<string, ConfigDescription>>(new Map());
const configTypes = ref<Array<string>>([]);
const loadError = ref<string | undefined>(undefined);

const crumbs = computed<Array<Crumb>>(() => [
  { label: "Repositories", to: { name: "RepositoriesList" } },
  { label: repository.value?.name ?? "" },
]);

function getConfigTitleOrFallback(config: string) {
  return configDescriptions.value.get(config)?.name || config;
}

watch(configTypes, async () => {
  for (const config of configTypes.value) {
    const description = await repositoryTypesStore.getConfigDescription(config);
    if (description) {
      configDescriptions.value.set(config, description);
    }
  }
});

const configComponents = computed(() =>
  configTypes.value.map((config) => {
    const configType = getConfigType(config);
    return configType
      ? {
          component: configType.component,
          configName: config,
          props: { repository: repositoryId },
        }
      : {
          component: FallBackEditor,
          configName: config,
          props: { settingName: config, repository: repositoryId },
        };
  }),
);

function onUpdated(updated: RepositoryWithStorageName) {
  repository.value = updated;
}

async function load() {
  try {
    const [details, configs] = await Promise.all([
      http.get<RepositoryWithStorageName>(`/api/repository/${repositoryId}`),
      http.get<Array<string>>(`/api/repository/${repositoryId}/configs`),
    ]);
    repository.value = details.data;
    configTypes.value = configs.data;
  } catch {
    loadError.value = "Could not load this repository.";
  }
}
load();
</script>

<style scoped lang="scss">
.loading,
.loadError {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-16) var(--space-4);
  color: var(--text-muted);
}

.loadError {
  color: var(--danger);
}
</style>
