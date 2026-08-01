<template>
  <div
    v-if="repository"
    class="stack">
    <NCard
      title="Details"
      subtitle="Renaming changes the URL clients pull from.">
      <form
        class="stack"
        @submit.prevent="save">
        <div class="grid">
          <div class="field">
            <label for="repositoryName">Name</label>
            <input
              id="repositoryName"
              v-model="form.name"
              class="mono"
              required
              spellcheck="false" />
          </div>

          <div class="field">
            <label for="repositoryVisibility">Visibility</label>
            <select
              id="repositoryVisibility"
              v-model="form.visibility">
              <option value="Public">Public — anyone can read</option>
              <option value="Hidden">Hidden — readable, not listed</option>
              <option value="Private">Private — members only</option>
            </select>
          </div>
        </div>

        <dl class="facts">
          <div>
            <dt>Type</dt>
            <dd>
              <NBadge variant="accent">{{ repository.repository_type }}</NBadge>
            </dd>
          </div>
          <div>
            <dt>Storage</dt>
            <dd>{{ repository.storage_name }}</dd>
          </div>
          <div>
            <dt>Repository ID</dt>
            <dd class="mono">{{ repository.id }}</dd>
          </div>
          <div>
            <dt>Created</dt>
            <dd>{{ formatDate(repository.created_at) }}</dd>
          </div>
        </dl>

        <div class="row">
          <NButton
            type="submit"
            variant="primary"
            :loading="saving"
            :disabled="!changed">
            Save changes
          </NButton>
          <NButton
            v-if="changed"
            variant="ghost"
            @click="reset">
            Discard
          </NButton>
        </div>
      </form>
    </NCard>

    <NCard title="Availability">
      <div class="availability">
        <div>
          <NBadge
            :variant="repository.active ? 'success' : 'neutral'"
            dot>
            {{ repository.active ? "Active" : "Disabled" }}
          </NBadge>
          <p class="hint">
            A disabled repository refuses every read and write, but keeps its files and its
            configuration.
          </p>
        </div>
        <NButton
          :loading="togglingActive"
          @click="setActive(!repository.active)">
          {{ repository.active ? "Disable" : "Enable" }}
        </NButton>
      </div>
    </NCard>

    <NCard title="Danger zone">
      <div class="availability">
        <p class="hint">
          Deleting a repository removes it and its database records. This cannot be undone.
        </p>
        <NButton
          variant="danger"
          icon="trash"
          @click="confirmingDelete = true">
          Delete repository
        </NButton>
      </div>
    </NCard>

    <NConfirmDialog
      v-model="confirmingDelete"
      title="Delete this repository?"
      :message="`Everything under ${repository.name} goes with it. This cannot be undone.`"
      confirm-label="Delete repository"
      destructive
      :loading="deleting"
      :confirm-text="repository.name"
      @confirm="deleteRepository" />
  </div>
</template>

<script setup lang="ts">
/**
 * Repository settings.
 *
 * Enable and disable were buttons wired to `notify("This feature is not implemented yet")`, and
 * there was no way to rename a repository or change its visibility at all — even though
 * `PUT /api/repository/{id}` handles all three. Deleting fired straight from the click handler with
 * nothing in between.
 */
import { computed, reactive, ref, watch, type PropType } from "vue";
import { notify } from "@kyvg/vue3-notification";
import http from "@/http";
import router from "@/router";
import NCard from "@/components/core/ui/NCard.vue";
import NButton from "@/components/core/ui/NButton.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import NConfirmDialog from "@/components/core/ui/NConfirmDialog.vue";
import { formatDate } from "@/utils/format";
import type { RepositoryWithStorageName } from "@/types/repository";

const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});

const emit = defineEmits<{ (e: "updated", repository: RepositoryWithStorageName): void }>();

const form = reactive({ name: "", visibility: "" });
const saving = ref(false);
const togglingActive = ref(false);
const deleting = ref(false);
const confirmingDelete = ref(false);

watch(
  () => props.repository,
  (repository) => {
    form.name = repository.name;
    form.visibility = repository.visibility;
  },
  { immediate: true },
);

const changed = computed(
  () => form.name !== props.repository.name || form.visibility !== props.repository.visibility,
);

function reset() {
  form.name = props.repository.name;
  form.visibility = props.repository.visibility;
}

async function update(body: Record<string, unknown>) {
  const response = await http.put<RepositoryWithStorageName>(
    `/api/repository/${props.repository.id}`,
    body,
  );
  emit("updated", response.data);
  return response.data;
}

async function save() {
  saving.value = true;
  try {
    await update({ name: form.name, visibility: form.visibility });
    notify({ type: "success", title: "Repository updated" });
  } catch (error: unknown) {
    // A rename can collide with an existing repository in the same storage, which the server
    // reports as a conflict — worth distinguishing from a generic failure.
    const status = (error as { response?: { status?: number } })?.response?.status;
    notify({
      type: "error",
      title: status === 409 ? "That name is already taken" : "Could not update the repository",
    });
  } finally {
    saving.value = false;
  }
}

async function setActive(active: boolean) {
  togglingActive.value = true;
  try {
    await update({ active });
    notify({ type: "success", title: active ? "Repository enabled" : "Repository disabled" });
  } catch {
    notify({ type: "error", title: "Could not change the repository's status" });
  } finally {
    togglingActive.value = false;
  }
}

async function deleteRepository() {
  deleting.value = true;
  try {
    await http.delete(`/api/repository/${props.repository.id}`);
    notify({ type: "success", title: "Repository deleted" });
    router.push({ name: "RepositoriesList" });
  } catch {
    notify({ type: "error", title: "Could not delete the repository" });
  } finally {
    deleting.value = false;
    confirmingDelete.value = false;
  }
}
</script>

<style scoped lang="scss">
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
  gap: var(--space-4);
}

.facts {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
  gap: var(--space-4);
  margin: 0;
  padding-top: var(--space-4);
  border-top: 1px solid var(--border);

  dt {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-label);
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  dd {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    word-break: break-all;
  }
}

.availability {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  flex-wrap: wrap;
}

.hint {
  margin-top: var(--space-2);
  max-width: 42rem;
  font-size: var(--text-sm);
  color: var(--text-muted);
}
</style>
