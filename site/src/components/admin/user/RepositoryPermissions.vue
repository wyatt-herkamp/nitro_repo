<template>
  <div>
    <div id="listHeader">
      <h2>Repository Permissions</h2>
      <button @click="save" :disabled="!hasChanged">Save</button>
    </div>
    <div id="header" class="row">
      <div class="col">Repository</div>
      <div class="col">Read</div>
      <div class="col">Write</div>
      <div class="col">Edit</div>
      <div class="col action">Action</div>
    </div>
    <div class="row item" v-for="repository in repositoryPermissions" :key="repository.id">
      <div class="col">{{ repository.name }}</div>
      <div class="col">
        <BaseSwitch v-model="repository.permissions.can_read" />
      </div>
      <div class="col">
        <BaseSwitch v-model="repository.permissions.can_write" />
      </div>
      <div class="col">
        <BaseSwitch v-model="repository.permissions.can_edit" />
      </div>
      <div class="col action">
        <button class="actionButton" @click="deleteRepository(repository.id)">Delete</button>
      </div>
    </div>
    <div class="row item" id="create">
      <div class="col" id="repoDropDown">
        <RepositoryDropdown v-model="newEntry.repository" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newEntry.can_read" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newEntry.can_write" />
      </div>
      <div class="col">
        <BaseSwitch v-model="newEntry.can_edit" />
      </div>
      <div class="col">
        <button class="actionButton" @click="addRepository" :disabled="!isNewEntryValid">
          Add
        </button>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, ref, type PropType } from 'vue'
import { compareRepositoryActions } from '@/types/user'
import type { RepositoryActions, User } from '@/types/base'
import BaseSwitch from '@/components/form/BaseSwitch.vue'
import { repositoriesStore } from '@/stores/repositories'
import RepositoryDropdown from '@/components/form/dropdown/RepositoryDropdown.vue'
import { notify } from '@kyvg/vue3-notification'
import http from '@/http'
import { watch } from 'vue'
const props = defineProps({
  user: {
    type: Object as PropType<User>,
    required: true
  }
})
const repoStore = repositoriesStore()

const hasChanged = ref(false)

const isNewEntryValid = computed(() => {
  if (!newEntry.value.repository) {
    return false
  }
  if (newEntry.value.repository.length === 0) {
    return false
  }
  return true
})

const newEntry = ref({
  repository: '',
  can_read: false,
  can_write: false,
  can_edit: false
})
function deleteRepository(repository: string) {
  for (let i = 0; i < repositoryPermissions.value.length; i++) {
    if (repositoryPermissions.value[i].id === repository) {
      repositoryPermissions.value.splice(i, 1)
      return
    }
  }
}
const repositoryPermissions = ref<
  {
    id: string
    name: string
    permissions: RepositoryActions
  }[]
>([])

async function addRepository() {
  if (!isNewEntryValid.value) {
    return
  }
  for (const repository of repositoryPermissions.value) {
    if (repository.id === newEntry.value.repository) {
      repository.permissions.can_read = newEntry.value.can_read
      repository.permissions.can_write = newEntry.value.can_write
      repository.permissions.can_edit = newEntry.value.can_edit
      notify({
        type: 'success',
        title: 'Repository Already Exists',
        text: 'Values have been updated.'
      })
      return
    }
  }
  let repositoryValue = await repoStore.getRepositoryById(newEntry.value.repository)
  if (!repositoryValue) {
    notify({
      type: 'error',
      title: 'Repository Not Found',
      text: 'The repository could not be found.'
    })
    return
  }

  repositoryPermissions.value.push({
    id: newEntry.value.repository,
    name: repositoryValue.name,
    permissions: {
      can_read: newEntry.value.can_read,
      can_write: newEntry.value.can_write,
      can_edit: newEntry.value.can_edit
    }
  })

  newEntry.value.repository = ''
  newEntry.value.can_read = false
  newEntry.value.can_write = false
  newEntry.value.can_edit = false
}

async function load() {
  for (const [repository, permissions] of Object.entries(
    props.user.permissions.repository_permissions
  )) {
    let repositoryValue = await repoStore.getRepositoryById(repository)
    repositoryPermissions.value.push({
      id: repository,
      name: repositoryValue?.name ?? 'Unknown',
      permissions: {
        can_read: permissions.can_read,
        can_write: permissions.can_write,
        can_edit: permissions.can_edit
      }
    })
  }
}
load()
watch(
  repositoryPermissions,
  () => {
    if (
      repositoryPermissions.value.length !==
      Object.keys(props.user.permissions.repository_permissions).length
    ) {
      console.debug('Length has changed')
      hasChanged.value = true
    }
    for (const repository of repositoryPermissions.value) {
      let originalRepositoryPermissions =
        props.user.permissions.repository_permissions[repository.id]
      if (!originalRepositoryPermissions) {
        console.debug(`Repository ${repository.id} has changed`)
        hasChanged.value = true
        return
      }
      console.debug(
        `Comparing ${JSON.stringify(originalRepositoryPermissions)} to ${JSON.stringify(
          repository.permissions
        )}`
      )
      if (!compareRepositoryActions(originalRepositoryPermissions, repository.permissions)) {
        console.debug(`Repository ${repository.id} has changed`)
        hasChanged.value = true
        return
      }
    }

    hasChanged.value = false
    console.log('No changes')
  },
  { deep: true }
)
async function save() {
  let repositoryPermissionsValue: Record<string, RepositoryActions> = {}
  for (const repository of repositoryPermissions.value) {
    repositoryPermissionsValue[repository.id] = repository.permissions
  }
  const newPermissions = {
    repository_permissions: repositoryPermissionsValue
  }
  console.log(`Saving: ${JSON.stringify(newPermissions)}`)
  await http
    .put(`/api/user-management/update/${props.user.id}/permissions`, newPermissions)
    .then(() => {
      notify({
        type: 'success',
        title: 'Permissions Saved',
        text: 'Permissions have been saved.'
      })
    })
    .catch((error) => {
      let text = 'An error occurred while saving permissions.'
      if (error.response.data) {
        text = error.response.data
      }
      notify({
        type: 'error',
        title: 'Error Saving Permissions',
        text: text
      })
    })
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme';
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
  .col {
    font-weight: bold;
  }
  border-bottom: 1px solid $primary-50;
  padding: 1rem 0rem;
}
.row {
  padding: 1rem;
  padding-top: 0.5rem;
}
#create {
  margin-top: 1rem;
  border-top: 1px solid $primary-50;
  #repoDropDown {
    margin-right: 1rem;
  }
}
#listHeader {
  display: flex;
  justify-content: space-between;
  margin-bottom: 1rem;
  button {
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
}
</style>
