<template>
  <main class="container container-narrow">
    <template v-if="storage">
      <div class="page-header">
        <div class="page-header-text">
          <NBreadcrumb
            :items="[
              { label: 'Storages', to: { name: 'StorageList' } },
              { label: storage.name },
            ]" />
          <h1>{{ storage.name }}</h1>
        </div>
        <div class="page-header-actions">
          <NBadge variant="accent">{{ storage.storage_type }}</NBadge>
          <NBadge
            :variant="storage.active ? 'success' : 'neutral'"
            dot>
            {{ storage.active ? "Active" : "Disabled" }}
          </NBadge>
        </div>
      </div>

      <form
        class="stack"
        @submit.prevent="save">
        <NCard title="Storage">
          <div class="field">
            <label for="storageName">Name</label>
            <input
              id="storageName"
              v-model="name"
              class="mono"
              required
              spellcheck="false" />
          </div>
        </NCard>

        <NCard
          v-if="storageType"
          :title="storageType.title">
          <component
            :is="storageType.updateComponent"
            v-model="settings" />
        </NCard>

        <p
          v-if="error"
          class="field-error">
          {{ error }}
        </p>

        <div class="row">
          <NButton
            type="submit"
            variant="primary"
            :loading="saving">
            Save changes
          </NButton>
          <NButton
            :loading="togglingActive"
            @click="setActive(!storage.active)">
            {{ storage.active ? "Disable" : "Enable" }}
          </NButton>
        </div>
      </form>
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
/**
 * Storage settings.
 *
 * This page was read-only: every field carried `disabled` and there was no submit handler, because
 * the server had no update endpoint at all. `PUT /api/storage/{id}` now exists, and offers a
 * configuration change to the running storage before persisting it — so an unreachable bucket fails
 * here rather than leaving a row that will not load on the next restart.
 */
import { computed, ref } from "vue";
import { notify } from "@kyvg/vue3-notification";
import NCard from "@/components/core/ui/NCard.vue";
import NButton from "@/components/core/ui/NButton.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import NBreadcrumb from "@/components/core/ui/NBreadcrumb.vue";
import SpinnerElement from "@/components/spinner/SpinnerElement.vue";
import { getStorageType, type StorageItem } from "@/components/nr/storage/storageTypes";
import http from "@/http";
import router from "@/router";

const storageId = router.currentRoute.value.params.id as string;

const storage = ref<StorageItem | undefined>(undefined);
const name = ref("");
const settings = ref<Record<string, unknown>>({});
const error = ref("");
const loadError = ref<string | undefined>(undefined);
const saving = ref(false);
const togglingActive = ref(false);

const storageType = computed(() =>
  storage.value ? getStorageType(storage.value.storage_type) : undefined,
);

function adopt(value: StorageItem) {
  storage.value = value;
  name.value = value.name;
  settings.value = { ...(value.config.settings as unknown as Record<string, unknown>) };
}

async function load() {
  try {
    const response = await http.get<StorageItem>(`/api/storage/${storageId}`);
    adopt(response.data);
  } catch {
    loadError.value = "Could not load this storage.";
  }
}
load();

async function update(body: Record<string, unknown>) {
  const response = await http.put<StorageItem>(`/api/storage/${storageId}`, body);
  adopt(response.data);
}

async function save() {
  saving.value = true;
  error.value = "";
  try {
    await update({
      name: name.value,
      config: { type: storage.value?.storage_type, settings: settings.value },
    });
    notify({ type: "success", title: "Storage updated" });
  } catch (caught: unknown) {
    const response = (caught as { response?: { status?: number; data?: unknown } })?.response;
    if (response?.status === 409) {
      error.value = "Another storage already has that name.";
    } else if (response?.status === 400) {
      error.value =
        typeof response.data === "string"
          ? response.data
          : "The server rejected this configuration.";
    } else {
      error.value = "Could not update the storage.";
    }
  } finally {
    saving.value = false;
  }
}

async function setActive(active: boolean) {
  togglingActive.value = true;
  try {
    await update({ active });
    notify({ type: "success", title: active ? "Storage enabled" : "Storage disabled" });
  } catch {
    notify({ type: "error", title: "Could not change the storage's status" });
  } finally {
    togglingActive.value = false;
  }
}
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
