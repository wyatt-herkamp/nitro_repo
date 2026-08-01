<template>
  <main class="container container-narrow">
    <div class="page-header">
      <div class="page-header-text">
        <NBreadcrumb
          :items="[{ label: 'Storages', to: { name: 'StorageList' } }, { label: 'New' }]" />
        <h1>New storage</h1>
      </div>
    </div>

    <form
      class="stack"
      @submit.prevent="createStorage">
      <NCard title="Storage">
        <div class="grid">
          <div class="field">
            <label for="storageName">Name</label>
            <input
              id="storageName"
              v-model="name"
              class="mono"
              required
              autocomplete="off"
              spellcheck="false" />
          </div>

          <div class="field">
            <label for="storageType">Type</label>
            <select
              id="storageType"
              v-model="storageType"
              required>
              <option
                value=""
                disabled>
                Choose a backend
              </option>
              <option
                v-for="type in storageTypes"
                :key="type.value"
                :value="type.value">
                {{ type.label }}
              </option>
            </select>
          </div>
        </div>

        <p
          v-if="selected"
          class="typeDescription">
          {{ selected.description }}
        </p>
      </NCard>

      <NCard
        v-if="selected"
        :title="selected.title">
        <component
          :is="selected.component"
          v-model="settings" />
      </NCard>

      <p
        v-if="error"
        class="field-error">
        {{ error }}
      </p>

      <div
        v-if="selected"
        class="row">
        <NButton
          type="submit"
          variant="primary"
          :loading="creating">
          Create storage
        </NButton>
        <NButton
          variant="ghost"
          :to="{ name: 'StorageList' }"
          >Cancel</NButton
        >
      </div>
    </form>
  </main>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import { notify } from "@kyvg/vue3-notification";
import NCard from "@/components/core/ui/NCard.vue";
import NButton from "@/components/core/ui/NButton.vue";
import NBreadcrumb from "@/components/core/ui/NBreadcrumb.vue";
import { getStorageType, storageTypes } from "@/components/nr/storage/storageTypes";
import http from "@/http";
import router from "@/router";

const name = ref("");
const storageType = ref("");
const settings = ref<Record<string, unknown>>({});
const error = ref("");
const creating = ref(false);

const selected = computed(() =>
  storageType.value === "" ? undefined : getStorageType(storageType.value),
);

// Switching backend has to reset the settings: the shapes have nothing in common, and carrying an
// S3 bucket name into a filesystem config would send it to the server as-is.
watch(storageType, () => {
  settings.value = selected.value?.defaultConfig() ?? {};
  error.value = "";
});

async function createStorage() {
  creating.value = true;
  error.value = "";
  try {
    const response = await http.post(`/api/storage/new/${storageType.value}`, {
      name: name.value,
      config: { type: storageType.value, settings: settings.value },
    });
    notify({ type: "success", title: "Storage created" });
    router.push({ name: "ViewStorage", params: { id: response.data.id } });
  } catch (caught: unknown) {
    // The old handler was an empty `if (error.response.status === 400) {}`, so an invalid config was
    // indistinguishable from a click that did nothing — and it threw on a network error, where
    // there is no `response` at all.
    const response = (caught as { response?: { status?: number; data?: unknown } })?.response;
    if (response === undefined) {
      error.value = "Could not reach the server.";
    } else if (response.status === 400) {
      error.value =
        typeof response.data === "string"
          ? response.data
          : "The server rejected this configuration.";
    } else {
      error.value = "Could not create the storage.";
    }
  } finally {
    creating.value = false;
  }
}
</script>

<style scoped lang="scss">
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: var(--space-4);
}

.typeDescription {
  margin-top: var(--space-4);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border);
  font-size: var(--text-sm);
  color: var(--text-muted);
}
</style>
