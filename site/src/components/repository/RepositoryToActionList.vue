<template>
  <div v-auto-animate id="repositoryEntries">
    <div id="header" class="row">
      <div class="col">Repository</div>
      <div class="col">Read</div>
      <div class="col">Write</div>
      <div class="col">Edit</div>
      <div class="col action">Action</div>
    </div>
    <div class="row item" v-for="entry in repositoryEntries" :key="entry.repositoryId">
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
          @click="removeRepositoryScopeEntry(entry.repositoryId)">
          Remove
        </button>
      </div>
    </div>
    <div class="row item" id="create">
      <div class="col" id="repoDropDown">
        <RepositoryDropdown v-model="newRepositoryScopeEntry.repositoryId" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newRepositoryScopeEntry.actions.can_read" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newRepositoryScopeEntry.actions.can_write" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newRepositoryScopeEntry.actions.can_edit" />
      </div>
      <div class="col">
        <button
          type="button"
          class="actionButton"
          @click="addRepositoryScopeEntry"
          :disabled="!isNewEntryValid">
          Add
        </button>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import BaseSwitch from '@/components/form/BaseSwitch.vue'
import RepositoryDropdown from '@/components/form/dropdown/RepositoryDropdown.vue'
import { repositoriesStore } from '@/stores/repositories'
import { RepositoryActionsType } from '@/types/user'
import { type NewAuthTokenRepositoryScope } from '@/types/user/token'
import { notify } from '@kyvg/vue3-notification'
import { computed, ref } from 'vue'

const repositoryStore = repositoriesStore()
function getRepositoryName(repositoryId: string) {
  const repository = repositoryStore.getRepositoryFromCache(repositoryId)
  return repository ? repository.name : repositoryId
}
const repositoryEntries = defineModel<Array<NewAuthTokenRepositoryScope>>({
  required: true
})
const newRepositoryScopeEntry = ref<NewAuthTokenRepositoryScope>({
  repositoryId: '',
  actions: new RepositoryActionsType([])
})
const isNewEntryValid = computed(() => {
  if (
    !newRepositoryScopeEntry.value.repositoryId ||
    newRepositoryScopeEntry.value.repositoryId == ''
  ) {
    return false
  }

  if (newRepositoryScopeEntry.value.actions.asArray().length === 0) {
    return false
  }
  return true
})

function addRepositoryScopeEntry() {
  for (const repository of repositoryEntries.value) {
    if (repository.repositoryId === newRepositoryScopeEntry.value.repositoryId) {
      repository.actions = newRepositoryScopeEntry.value.actions
      notify({
        type: 'success',
        title: 'Repository Already Exists',
        text: 'Values have been updated.'
      })
      newRepositoryScopeEntry.value = {
        repositoryId: '',
        actions: new RepositoryActionsType([])
      }
      return
    }
  }
  repositoryEntries.value.push({
    repositoryId: newRepositoryScopeEntry.value.repositoryId,
    actions: newRepositoryScopeEntry.value.actions
  })
  newRepositoryScopeEntry.value = {
    repositoryId: '',
    actions: new RepositoryActionsType([])
  }
}

function removeRepositoryScopeEntry(id: string) {
  repositoryEntries.value = repositoryEntries.value.filter((entry) => entry.repositoryId !== id)
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme';

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
