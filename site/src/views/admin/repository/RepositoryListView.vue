<template>
  <main class="container">
    <div class="page-header">
      <div class="page-header-text">
        <h1>Repositories</h1>
        <p>Every repository on this instance.</p>
      </div>
      <div class="page-header-actions">
        <NButton
          variant="primary"
          icon="plus"
          :to="{ name: 'RepositoryCreate' }">
          New repository
        </NButton>
      </div>
    </div>

    <RepositoryTable
      admin
      :repositories="repositories"
      :loading="loading"
      :error="error" />
  </main>
</template>

<script setup lang="ts">
import http from "@/http";
import { ref } from "vue";
import RepositoryTable from "@/components/nr/repository/RepositoryTable.vue";
import NButton from "@/components/core/ui/NButton.vue";
import type { RepositoryWithStorageName } from "@/types/repository";

const repositories = ref<Array<RepositoryWithStorageName>>([]);
const error = ref<string | undefined>(undefined);
const loading = ref(true);

async function load() {
  try {
    const response = await http.get<Array<RepositoryWithStorageName>>("/api/repository/list");
    repositories.value = response.data;
  } catch {
    // The old handler was `.catch((error) => { error.value = "..." })`, whose parameter shadowed the
    // ref — so the message was assigned onto the caught exception and the page just stayed blank.
    error.value = "Failed to load repositories.";
  } finally {
    loading.value = false;
  }
}
load();
</script>
