<template>
  <main class="container">
    <div class="page-header">
      <div class="page-header-text">
        <h1>Administration</h1>
        <p>Repositories, storages and accounts for this instance.</p>
      </div>
    </div>

    <div class="cards">
      <RouterLink
        v-for="section in sections"
        :key="section.title"
        :to="section.to"
        class="sectionCard">
        <font-awesome-icon
          :icon="section.icon"
          class="sectionIcon" />
        <span class="sectionTitle">{{ section.title }}</span>
        <span class="sectionCount">{{ section.count }}</span>
        <span class="sectionDescription">{{ section.description }}</span>
      </RouterLink>
    </div>
  </main>
</template>

<script setup lang="ts">
/**
 * This page was an `<h1>Admin Home Page</h1>` and nothing else. Landing on it now says what is on
 * the instance and gets you to the thing you came for.
 */
import { computed, ref } from "vue";
import { RouterLink, type RouteLocationRaw } from "vue-router";
import { useRepositoryStore } from "@/stores/repositories";
import http from "@/http";
import type { UserResponseType } from "@/types/base";

const repositoryStore = useRepositoryStore();
const repositoryCount = ref<number | undefined>(undefined);
const storageCount = ref<number | undefined>(undefined);
const userCount = ref<number | undefined>(undefined);

// A dash rather than a zero while loading: "0 repositories" and "not loaded yet" are different
// things, and only one of them is a reason to go and create something.
const asCount = (value?: number) => (value === undefined ? "—" : String(value));

const sections = computed<
  Array<{
    title: string;
    description: string;
    icon: string;
    count: string;
    to: RouteLocationRaw;
  }>
>(() => [
  {
    title: "Repositories",
    description: "Create, rename and configure repositories.",
    icon: "box-open",
    count: asCount(repositoryCount.value),
    to: { name: "RepositoriesList" },
  },
  {
    title: "Storages",
    description: "Where repositories put their files.",
    icon: "database",
    count: asCount(storageCount.value),
    to: { name: "StorageList" },
  },
  {
    title: "Users",
    description: "Accounts and their permissions.",
    icon: "users",
    count: asCount(userCount.value),
    to: { name: "UsersList" },
  },
]);

async function load() {
  const [repositories, storages] = await Promise.all([
    repositoryStore.getRepositories(),
    repositoryStore.getStorages(),
  ]);
  repositoryCount.value = repositories.length;
  storageCount.value = storages.length;

  // Only user managers may list users, so a refusal here is expected rather than an error to show.
  try {
    const users = await http.get<Array<UserResponseType>>("/api/user-management/list");
    userCount.value = users.data.length;
  } catch {
    userCount.value = undefined;
  }
}
load();
</script>

<style scoped lang="scss">
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(15rem, 1fr));
  gap: var(--space-4);
}

.sectionCard {
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-rows: auto auto;
  align-items: center;
  gap: var(--space-1) var(--space-3);
  padding: var(--space-5);
  color: var(--text);
  background-color: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  transition:
    border-color var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);

  &:hover {
    color: var(--text);
    border-color: var(--accent-border);
    background-color: var(--surface-hover);
  }
}

.sectionIcon {
  grid-row: 1 / span 2;
  font-size: 1.5rem;
  color: var(--accent);
}

.sectionTitle {
  font-weight: var(--weight-medium);
}

.sectionCount {
  grid-column: 2;
  grid-row: 1;
  justify-self: end;
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
}

.sectionDescription {
  grid-column: 2;
  font-size: var(--text-sm);
  color: var(--text-muted);
}
</style>
