<template>
  <main
    v-if="repository"
    class="container">
    <div class="page-header">
      <div class="page-header-text">
        <NBreadcrumb :items="crumbs" />
        <h1>{{ repository.name }}</h1>
        <div class="chips">
          <NBadge variant="accent">{{ repository.repository_type }}</NBadge>
          <NBadge :variant="repository.visibility === 'Public' ? 'success' : 'neutral'">
            {{ repository.visibility }}
          </NBadge>
          <NBadge
            v-if="!repository.active"
            variant="warning"
            >Disabled</NBadge
          >
        </div>
      </div>

      <div class="page-header-actions">
        <NButton
          icon="folder"
          :to="{ name: 'Browse', params: { id: repository.id, catchAll: '' } }">
          Browse files
        </NButton>
      </div>
    </div>

    <NCard
      title="Repository URL"
      subtitle="Point your client at this.">
      <CopyURL :code="url" />
      <div
        v-if="repositoryType && repositoryType.icons.length > 0"
        class="icons">
        <RepositoryIcon
          v-for="icon in repositoryType.icons"
          :key="icon.name"
          :name="repositoryType.name"
          :icon="icon" />
      </div>
    </NCard>

    <RepositoryPageViewer
      v-if="repositoryPage && repositoryPage.page_type !== 'None'"
      :repository="repository"
      :page="repositoryPage" />

    <RepositoryHelper :repository="repository" />
  </main>

  <ErrorOnRequest
    v-else-if="error"
    :error="error"
    :errorCode="errorCode" />

  <main
    v-else
    class="loading">
    <SpinnerElement size="lg" />
  </main>
</template>

<script setup lang="ts">
import CopyURL from "@/components/core/code/CopyCode.vue";
import ErrorOnRequest from "@/components/ErrorOnRequest.vue";
import RepositoryHelper from "@/components/nr/repository/RepositoryHelper.vue";
import RepositoryIcon from "@/components/nr/repository/RepositoryIcon.vue";
import RepositoryPageViewer from "@/components/nr/repository/RepositoryPageViewer.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import NButton from "@/components/core/ui/NButton.vue";
import NCard from "@/components/core/ui/NCard.vue";
import NBreadcrumb, { type Crumb } from "@/components/core/ui/NBreadcrumb.vue";
import SpinnerElement from "@/components/spinner/SpinnerElement.vue";
import http from "@/http";
import router from "@/router";
import { useRepositoryStore } from "@/stores/repositories";
import {
  createRepositoryRoute,
  findRepositoryType,
  type RepositoryPage,
  type RepositoryWithStorageName,
} from "@/types/repository";
import { computed, ref } from "vue";

const repoStore = useRepositoryStore();

const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const repositoryPage = ref<RepositoryPage | undefined>(undefined);
const error = ref<string | null>(null);
const errorCode = ref<number | undefined>(undefined);

const repositoryType = computed(() =>
  repository.value ? findRepositoryType(repository.value.repository_type) : undefined,
);

const url = computed(() => (repository.value ? createRepositoryRoute(repository.value) : ""));

const crumbs = computed<Array<Crumb>>(() => [
  { label: "Repositories", to: { name: "repositories" } },
  { label: repository.value?.storage_name ?? "" },
  { label: repository.value?.name ?? "" },
]);

async function load(repositoryId: string) {
  try {
    repository.value = await repoStore.getRepositoryById(repositoryId);
    if (repository.value === undefined) {
      error.value = "Repository not found";
      errorCode.value = 404;
      return;
    }
  } catch {
    error.value = "Could not load this repository";
    return;
  }

  // A repository with no page configured is the normal case, not a failure — it just means there is
  // nothing to render above the install snippets.
  try {
    const response = await http.get<RepositoryPage>(`/api/repository/page/${repositoryId}`);
    repositoryPage.value = response.data;
  } catch {
    repositoryPage.value = undefined;
  }
}

async function resolve() {
  const params = router.currentRoute.value.params;
  if (params.repositoryId) {
    await load(params.repositoryId as string);
    return;
  }

  if (params.storageName && params.repositoryName) {
    // `getRepositoryIdByNames` resolves to `undefined` when it cannot find one; this used to check
    // `=== null`, so a miss fell straight through and the page rendered nothing at all.
    const resolved = await repoStore.getRepositoryIdByNames(
      params.storageName as string,
      params.repositoryName as string,
    );
    if (!resolved) {
      error.value = "Repository not found";
      errorCode.value = 404;
      return;
    }
    await load(resolved);
    return;
  }

  error.value = "Repository not found";
  errorCode.value = 404;
}

resolve();
</script>

<style scoped lang="scss">
.chips {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

.icons {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-4);
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-16) var(--space-4);
}
</style>
