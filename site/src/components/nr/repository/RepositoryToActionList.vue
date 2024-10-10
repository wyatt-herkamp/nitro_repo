<template>
  <div
    v-auto-animate
    id="repositoryEntries">
    <div
      id="header"
      class="row">
      <div class="col">Repository</div>
      <div class="col">Read</div>
      <div class="col">Write</div>
      <div class="col">Edit</div>
      <div class="col action">Action</div>
    </div>
    <div
      class="row item"
      v-for="entry in repositoryEntries"
      :key="entry.repositoryId">
      <div class="col">{{ getRepositoryName(entry.repositoryId) }}</div>
      <div class="col">
        <BaseSwitch v-model="entry.actions.can_read" />
      </div>
      <div class="col">
        <BaseSwitch v-model="entry.actions.can_write" />
      </div>
      <div class="col">
        <BaseSwitch v-model="entry.actions.can_edit" />
      </div>
      <div class="col">
        <button
          type="button"
          class="actionButton"
          @click="removeEntry(entry.repositoryId)">
          Remove
        </button>
      </div>
    </div>
    <div
      class="row item"
      id="create">
      <div
        class="col"
        id="repoDropDown">
        <RepositoryDropdown v-model="newEntry.repositoryId" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newEntry.actions.can_read" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newEntry.actions.can_write" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newEntry.actions.can_edit" />
      </div>
      <div class="col">
        <button
          type="button"
          class="actionButton"
          @click="addEntry"
          :disabled="!isNewEntryValid">
          Add
        </button>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import BaseSwitch from "@/components/form/BaseSwitch.vue";
import RepositoryDropdown from "@/components/form/dropdown/RepositoryDropdown.vue";
import { repositoriesStore } from "@/stores/repositories";
import { RepositoryActionsType } from "@/types/user";
import { type NewAuthTokenRepositoryScope } from "@/types/user/token";
import { notify } from "@kyvg/vue3-notification";
import { computed, ref } from "vue";

const repositoryStore = repositoriesStore();
function getRepositoryName(repositoryId: string) {
  const repository = repositoryStore.getRepositoryFromCache(repositoryId);
  return repository ? repository.name : repositoryId;
}
const repositoryEntries = defineModel<Array<NewAuthTokenRepositoryScope>>({
  required: true,
});
const newEntry = ref<NewAuthTokenRepositoryScope>({
  repositoryId: "",
  actions: new RepositoryActionsType([]),
});
const isNewEntryValid = computed(() => {
  if (!newEntry.value.repositoryId || newEntry.value.repositoryId == "") {
    return false;
  }

  if (newEntry.value.actions.asArray().length === 0) {
    return false;
  }
  return true;
});

function addEntry() {
  for (const repository of repositoryEntries.value) {
    if (repository.repositoryId === newEntry.value.repositoryId) {
      repository.actions = newEntry.value.actions;
      notify({
        type: "success",
        title: "Repository Already Exists",
        text: "Values have been updated.",
      });
      newEntry.value = {
        repositoryId: "",
        actions: new RepositoryActionsType([]),
      };
      return;
    }
  }
  repositoryEntries.value.push({
    repositoryId: newEntry.value.repositoryId,
    actions: newEntry.value.actions,
  });
  newEntry.value = {
    repositoryId: "",
    actions: new RepositoryActionsType([]),
  };
}

function removeEntry(id: string) {
  repositoryEntries.value = repositoryEntries.value.filter((entry) => entry.repositoryId !== id);
}
</script>

<style scoped lang="scss">
@import "@/assets/styles/theme";

#repositoryEntries {
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 1fr 1fr;
  }
  .actionButton {
    background-color: $primary;
    color: white;
    border: none;
    padding: 0.5rem;
    border-radius: 0.5rem;
    cursor: pointer;
    &:disabled {
      background-color: $primary-50;
      cursor: not-allowed;
    }
  }
  #header {
    border-bottom: 1px solid $primary-50;
    padding: 1rem 0rem;
    .col {
      font-weight: bold;
    }
  }
  .row {
    padding: 1rem;
    padding-top: 0.5rem;
  }
}
</style>
