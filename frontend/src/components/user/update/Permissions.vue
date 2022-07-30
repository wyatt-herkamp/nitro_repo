<template>
  <div class="md:w-1/2 mx-auto">
    <div class="flex flex-col mb-6">
      <div class="flex flex-row">
        <div class="basis-1/2 settingBox">
          <h3 class="settingHeader">Admin</h3>
          <Switch id="admin" :disabled="disabled" v-model="permissions.admin">
            <div class="ml-1 text-quaternary">Admin</div>
          </Switch>
        </div>
      </div>
      <div class="settingBox" v-show="!admin">
        <h3 class="settingHeader">Normal Permissions</h3>
        <div class="otherPermissions flex flex-col">
          <div>
            <Switch
              :disabled="disabled"
              id="repository_manager"
              v-model="permissions.repository_manager"
            >
              <div class="ml-1 text-quaternary">Repository Manager</div>
            </Switch>

            <Switch
              :disabled="disabled"
              id="user_manager"
              v-model="permissions.user_manager"
            >
              <div class="ml-1 text-quaternary">User Manager</div>
            </Switch>
          </div>
          <div class="permissionBox mt-6">
            <PermissionList class="w-full" v-model="permissions.deployer">
              Deployer
            </PermissionList>
          </div>
          <div class="permissionBox mt-20">
            <PermissionList class="w-full" v-model="permissions.viewer">
              Viewer
            </PermissionList>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped>
.permissionBox {
  @apply md:flex;
  @apply md:flex-row;
  @apply md:flex-wrap;
  @apply md:h-40;
  @apply rounded-md;
}
.otherPermissions {
  @apply w-full;
}
</style>
<script lang="ts">
import { computed, defineComponent, ref, watch } from "vue";
import PermissionList from "./PermissionList.vue";
import { UserPermissions } from "@/types/userTypes";

export default defineComponent({
  props: {
    modelValue: {
      required: true,
      type: Object as () => UserPermissions,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  setup(props, { emit }) {
    const permissions = ref(props.modelValue);
    watch(permissions, () => {
      emit("update:modelValue", permissions);
    });
    const admin = computed(() => {
      return permissions.value.admin;
    });
    return { admin, permissions };
  },
  methods: {},
  components: { PermissionList },
});
</script>
