<template>
  <main class="container">
    <div class="page-header">
      <div class="page-header-text">
        <h1>Storages</h1>
        <p>Where repositories put their files.</p>
      </div>
      <div class="page-header-actions">
        <NButton
          variant="primary"
          icon="plus"
          :to="{ name: 'StorageCreate' }">
          New storage
        </NButton>
      </div>
    </div>

    <StorageTable
      :storages="storages"
      :loading="loading"
      :error="error" />
  </main>
</template>

<script setup lang="ts">
import { ref } from "vue";
import StorageTable from "@/components/nr/storage/StorageTable.vue";
import NButton from "@/components/core/ui/NButton.vue";
import { useRepositoryStore } from "@/stores/repositories";
import type { StorageItem } from "@/components/nr/storage/storageTypes";

const storages = ref<Array<StorageItem>>([]);
const error = ref<string | undefined>(undefined);
const loading = ref(true);
const repositoryStore = useRepositoryStore();

async function load() {
  try {
    storages.value = await repositoryStore.getStorages();
  } catch {
    error.value = "Failed to load storages.";
  } finally {
    loading.value = false;
  }
}
load();
</script>
