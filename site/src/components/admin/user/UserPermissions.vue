<template>
  <div>
    <div id="userPermissionsHeader">
      <h1>User Permission</h1>
      <SubmitButton
        :disabled="!hasChanged"
        @click="save"
        >Save</SubmitButton
      >
    </div>
    <div class="staticPermissions">
      <div class="permissionsSection">
        <h2>Primary Permissions</h2>
        <p>General Permissions for Nitro Repo</p>
        <div
          id="primaryPermissions"
          class="twoByGrid">
          <SwitchInput
            id="admin"
            v-model="userPermissions.admin">
            <template #comment>
              Admins have full control over the system.
              <br />
              <small>All other permissions are ignored</small>
            </template>
            Admin
          </SwitchInput>
          <SwitchInput
            id="userManager"
            v-model="userPermissions.user_manager">
            <template #comment>
              Can create, edit, and remove users.
              <br />
              <small>Admins can only be edited by admins</small>
            </template>
            User Manager
          </SwitchInput>
          <SwitchInput
            id="systemManager"
            v-model="userPermissions.system_manager">
            <template #comment>
              Can create, edit, and remove storages and repositories. They will also have full read
              and write access to all repositories
            </template>
            System Manager
          </SwitchInput>
        </div>
      </div>
      <div class="permissionsSection">
        <h2>Default Repository Permissions</h2>
        <p>
          Default Permissions for a Repository. Used if the person does not have a set permissions
        </p>
        <div
          id="defaultRepository"
          class="twoByGrid">
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
import SubmitButton from "@/components/form/SubmitButton.vue";
import SwitchInput from "@/components/form/SwitchInput.vue";
import http from "@/http";
import { type UserResponseType } from "@/types/base";
import { RepositoryActionsType } from "@/types/user";
import { notify } from "@kyvg/vue3-notification";
import { computed, ref, watch, type PropType } from "vue";

const props = defineProps({
  user: {
    type: Object as PropType<UserResponseType>,
    required: true,
  },
});
const hasChanged = computed(() => {
  if (userPermissions.value.admin !== props.user.admin) {
    return true;
  }
  if (userPermissions.value.user_manager !== props.user.user_manager) {
    return true;
  }
  if (userPermissions.value.system_manager !== props.user.system_manager) {
    return true;
  }

  return !userPermissions.value.default_repository_permissions.equalsArray(
    props.user.default_repository_actions,
  );
});
const userPermissions = ref({
  admin: props.user.admin,
  user_manager: props.user.user_manager,
  system_manager: props.user.system_manager,
  default_repository_permissions: new RepositoryActionsType(props.user.default_repository_actions),
});
watch(
  userPermissions,
  () => {
    console.log(`User Permissions: ${JSON.stringify(userPermissions)}`);
  },
  { deep: true },
);
async function save() {
  console.log("Saving User Permissions");
  const newPermissions = {
    admin: userPermissions.value.admin,
    user_manager: userPermissions.value.user_manager,
    system_manager: userPermissions.value.system_manager,
    default_repository_actions: userPermissions.value.default_repository_permissions.asArray(),
  };
  console.log(`Saving: ${JSON.stringify(newPermissions)}`);
  await http
    .put(`/api/user-management/update/${props.user.id}/permissions`, newPermissions)
    .then(() => {
      notify({
        type: "success",
        title: "Permissions Saved",
        text: "Permissions have been saved.",
      });
    })
    .catch((error) => {
      let text = "An error occurred while saving permissions.";
      if (error.response.data) {
        text = error.response.data;
      }
      notify({
        type: "error",
        title: "Error Saving Permissions",
        text: text,
      });
    });
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
