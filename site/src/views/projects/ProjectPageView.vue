<template>
  <main
    v-if="project"
    class="container">
    <div class="page-header">
      <div class="page-header-text">
        <NBreadcrumb :items="crumbs" />
        <h1>{{ project.name }}</h1>
        <div class="chips">
          <NBadge
            v-if="repository"
            variant="accent"
            >{{ repository.repository_type }}</NBadge
          >
          <span
            v-if="project.scope"
            class="mono scope"
            >{{ project.scope }}</span
          >
          <span
            v-if="project.latest_release"
            class="mono scope"
            >latest {{ project.latest_release }}</span
          >
        </div>
      </div>

      <div class="page-header-actions">
        <NButton
          icon="folder"
          :to="{
            name: 'Browse',
            params: { id: project.repository_id, catchAll: project.storage_path },
          }">
          Browse files
        </NButton>
        <NButton
          :to="{
            name: 'repository_page_by_id',
            params: { repositoryId: project.repository_id },
          }">
          Repository
        </NButton>
      </div>
    </div>

    <div
      v-if="repositoryType && repositoryType.icons.length > 0"
      class="icons">
      <RepositoryIcon
        v-for="icon in repositoryType.icons"
        :key="icon.name"
        :name="repositoryType.name"
        :icon="icon" />
    </div>

    <component
      :is="handlerComponent"
      v-if="handlerComponent"
      :project="project"
      :repository="repository" />

    <NEmptyState
      v-else-if="repositoryHandler"
      title="Nothing to show for this project type"
      :description="`The frontend knows about ${repositoryHandler.properName} repositories but has no project view for them yet.`"
      icon="cube" />

    <NEmptyState
      v-else
      title="Unsupported repository type"
      :description="`${repository?.repository_type ?? 'This repository type'} has no frontend definition, so install snippets and version listings cannot be rendered.`"
      icon="triangle-exclamation" />
  </main>

  <ErrorOnRequest
    v-else-if="error"
    :error="error"
    :errorCode="errorCode" />

  <main
    v-else-if="notFound"
    class="container">
    <NEmptyState
      title="Project not found"
      description="It may have been unpublished, or you may not have access to the repository holding it."
      icon="triangle-exclamation">
      <NButton
        variant="primary"
        :to="{ name: 'search' }"
        >Search</NButton
      >
    </NEmptyState>
  </main>

  <main
    v-else
    class="loading">
    <SpinnerElement size="lg" />
  </main>
</template>

<script setup lang="ts">
import ErrorOnRequest from "@/components/ErrorOnRequest.vue";
import RepositoryIcon from "@/components/nr/repository/RepositoryIcon.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import NButton from "@/components/core/ui/NButton.vue";
import NBreadcrumb, { type Crumb } from "@/components/core/ui/NBreadcrumb.vue";
import NEmptyState from "@/components/core/ui/NEmptyState.vue";
import SpinnerElement from "@/components/spinner/SpinnerElement.vue";
import router from "@/router";
import { useProjectStore } from "@/stores/project_store";
import { useRepositoryStore } from "@/stores/repositories";
import type { Project } from "@/types/project";
import {
  findRepositoryType,
  type FrontendRepositoryType,
  type RepositoryWithStorageName,
} from "@/types/repository";
import { computed, ref, watch } from "vue";

const projectId = router.currentRoute.value.params.projectId as string;
const repositoryId = ref<string | undefined>(undefined);
const repository = ref<RepositoryWithStorageName | undefined>(undefined);
const project = ref<Project | undefined>(undefined);
const error = ref<string | null>(null);
const errorCode = ref<number | undefined>(undefined);
// Distinct from "still loading": the page used to render "Project Not Found" the instant it mounted,
// before the request had even been made.
const notFound = ref(false);
const repoStore = useRepositoryStore();
const projectStore = useProjectStore();
const repositoryHandler = ref<FrontendRepositoryType | undefined>(undefined);

const repositoryType = computed(() =>
  repository.value ? findRepositoryType(repository.value.repository_type) : undefined,
);

const handlerComponent = computed(
  () =>
    repositoryHandler.value?.fullProjectComponent?.component ??
    repositoryHandler.value?.projectComponent?.component,
);

const crumbs = computed<Array<Crumb>>(() => {
  const items: Array<Crumb> = [{ label: "Repositories", to: { name: "repositories" } }];
  if (repository.value) {
    items.push({
      label: `${repository.value.storage_name}/${repository.value.name}`,
      to: { name: "repository_page_by_id", params: { repositoryId: repository.value.id } },
    });
  }
  items.push({ label: project.value?.name ?? "" });
  return items;
});

async function fetchProject() {
  const route = router.currentRoute.value;
  if (route.params.projectId) {
    project.value = await projectStore.getProjectById(projectId);
    repositoryId.value = project.value?.repository_id;
  } else if (route.params.storageName && route.params.repositoryName && route.params.projectKey) {
    const resolved = await repoStore.getRepositoryIdByNames(
      route.params.storageName as string,
      route.params.repositoryName as string,
    );
    if (!resolved) {
      notFound.value = true;
      return;
    }
    repositoryId.value = resolved;
    project.value = await projectStore.getProjectByKey(resolved, route.params.projectKey as string);
    repositoryId.value = project.value?.repository_id ?? resolved;
  }

  if (project.value === undefined) {
    notFound.value = true;
  }
}

watch(repositoryId, async () => {
  if (!repositoryId.value) return;
  repository.value = await repoStore.getRepositoryById(repositoryId.value);
  if (repository.value) {
    repositoryHandler.value = findRepositoryType(repository.value.repository_type);
  }
});

fetchProject();
</script>

<style scoped lang="scss">
.chips {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

.scope {
  font-size: var(--text-xs);
  color: var(--text-subtle);
}

.icons {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-bottom: var(--space-4);
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-16) var(--space-4);
}
</style>
