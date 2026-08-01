<template>
  <main class="container">
    <div class="page-header">
      <div class="page-header-text">
        <h1>Repositories</h1>
        <p>Browse the repositories you have access to.</p>
      </div>
    </div>

    <RepositoryTable
      :repositories="repositories"
      :loading="loading"
      :error="error" />
  </main>
</template>

<script setup lang="ts">
/**
 * Serves both `/` and `/page/repositories`. `HomeView.vue` was a byte-identical copy of this file;
 * both routes now render this one.
 */
import { ref } from "vue";
import RepositoryTable from "@/components/nr/repository/RepositoryTable.vue";
import { useRepositoryStore } from "@/stores/repositories";
import type { RepositoryWithStorageName } from "@/types/repository";

const repositories = ref<Array<RepositoryWithStorageName>>([]);
const error = ref<string | undefined>(undefined);
const loading = ref(true);
const repositoryStore = useRepositoryStore();

async function load() {
  try {
    repositories.value = await repositoryStore.getRepositories();
  } catch {
    // Previously this rendered nothing when the request failed and nothing when the list came back
    // empty, so the two were indistinguishable from a page that had not finished loading.
    error.value = "Could not load repositories.";
  } finally {
    loading.value = false;
  }
}
load();
</script>
