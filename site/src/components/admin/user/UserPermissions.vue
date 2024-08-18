<template>
  <div>
    <div id="userPermissionsHeader">
      <h1>User Permission</h1>
      <SubmitButton :disabled="!hasChanged" @click="save">Save</SubmitButton>
    </div>
    <div class="staticPermissions">
      <div class="permissionsSection">
        <h2>Primary Permissions</h2>
        <p>General Permissions for Nitro Repo</p>
        <div id="primaryPermissions" class="twoByGrid">
          <SwitchInput id="admin" v-model="userPermissions.admin">
            <template #comment>
              Admins have full control over the system.
              <br />
              <small>All other permissions are ignored</small>
            </template>
            Admin
          </SwitchInput>
          <SwitchInput id="userManager" v-model="userPermissions.user_manager">
            <template #comment>
              Can create, edit, and remove users.
              <br />
              <small>Admins can only be edited by admins</small>
            </template>
            User Manager
          </SwitchInput>
          <SwitchInput id="storageManager" v-model="userPermissions.storage_manager">
            <template #comment> Can create, edit, and remove storages </template>
            Storage Manager
          </SwitchInput>
          <SwitchInput id="repositoryManager" v-model="userPermissions.repository_manager">
            <template #comment>
              Can create, edit, and remove repositories.
              <br />
              <small>They also can see all storages</small>
            </template>
            Repository Manager
          </SwitchInput>
        </div>
      </div>
      <div class="permissionsSection">
        <h2>Default Repository Permissions</h2>
        <p>
          Default Permissions for a Repository. Used if the person does not have a set permissions
        </p>
        <div id="defaultRepository" class="twoByGrid">
          <SwitchInput
            id="defaultRead"
            v-model="userPermissions.default_repository_permissions.can_read">
            <template #comment> Can read artifacts on any repository </template>
            Read
          </SwitchInput>
          <SwitchInput
            id="defaultWrite"
            v-model="userPermissions.default_repository_permissions.can_write">
            <template #comment> Can write artifacts on any repository </template>
            Write
          </SwitchInput>
          <SwitchInput
            id="defaultExecute"
            v-model="userPermissions.default_repository_permissions.can_edit">
            <template #comment> Can edit configuration on any repository </template>
            Edit
          </SwitchInput>
        </div>
      </div>
    </div>
  </div>
</template>
<script lang="ts" setup>
import SubmitButton from '@/components/form/SubmitButton.vue'
import SwitchInput from '@/components/form/SwitchInput.vue'
import http from '@/http'
import type { User } from '@/types/base'
import { UserPermissions } from '@/types/user'
import { notify } from '@kyvg/vue3-notification'
import { computed, ref, watch, type PropType } from 'vue'

const props = defineProps({
  user: {
    type: Object as PropType<User>,
    required: true
  }
})
const hasChanged = computed(() => {
  return !userPermissions.value.equalsRawType(props.user.permissions)
})
const userPermissions = ref<UserPermissions>(new UserPermissions(props.user.permissions))

watch(
  userPermissions,
  (newValue) => {
    console.log(`User Permissions: ${JSON.stringify(userPermissions)}`)
  },
  { deep: true }
)
async function save() {
  console.log('Saving User Permissions')
  const newPermissions = {
    admin: userPermissions.value.admin,
    user_manager: userPermissions.value.user_manager,
    storage_manager: userPermissions.value.storage_manager,
    repository_manager: userPermissions.value.repository_manager,
    default_repository_permissions: {
      can_read: userPermissions.value.default_repository_permissions.can_read,
      can_write: userPermissions.value.default_repository_permissions.can_write,
      can_edit: userPermissions.value.default_repository_permissions.can_edit
    }
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
<style lang="scss" scoped>
.staticPermissions {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  margin: 1rem 2rem;
  flex-wrap: wrap;
}

.twoByGrid {
  display: grid;
  grid-template-columns: 1fr 1fr;
}
#userPermissionsHeader {
  display: flex;
  justify-content: space-between;
  margin: 1rem 2rem;
  button {
    max-width: 4rem;
  }
}
</style>
