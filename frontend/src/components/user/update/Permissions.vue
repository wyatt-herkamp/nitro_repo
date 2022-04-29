<template>
  <div class="nitroForm">
    <div class="flex flex-col mb-6">
      <div class="flex flex-row">
        <div class="basis-1/2 settingBox">
          <h3 class="settingHeader">Admin</h3>
          <Switch id="admin" v-model="user.permissions.admin">
            <div class="ml-1">Admin</div>
          </Switch>
        </div>
        <div class="basis-1/2 my-auto">
          <button @click="updatePermissions()" class="nitroButton">
            Save Permissions
          </button>
        </div>
      </div>

      <div class="settingBox">
        <h3 class="settingHeader">Normal Permissions</h3>
        <h6
          class="appearance-none text-left font-sm"
          v-if="user.permissions.admin"
        >
          These Permissions Do not matter. This user is Admin
        </h6>

        <div
          class="otherPermissions flex flex-col"
          :class="[admin ? 'admin' : '']"
        >
          <div>
            <Switch
              id="repository_manager"
              v-model="user.permissions.repository_manager"
            >
              <div class="ml-1">Repository Manager</div>
            </Switch>

            <Switch id="user_manager" v-model="user.permissions.user_manager">
              <div class="ml-1">User Manager</div>
            </Switch>
          </div>
          <div class="permissionBox mb-5">
            <div class="md:basis-1/4">
              <h6 class="text-left block" v-if="user.permissions.admin">
                Deployer Permissions
              </h6>
              <Switch class="block" id="deployer" v-model="deployer">
                <div class="ml-1">Deployer</div>
              </Switch>
            </div>
            <PermissionList
              class="md:basis-3/4"
              v-if="user.permissions.deployer != undefined"
              v-model="user.permissions.deployer"
            />
          </div>
          <div class="permissionBox">
            <div class="md:basis-1/4">
              <h6 class="text-left block" v-if="user.permissions.admin">
                Viewer Permissions
              </h6>
              <Switch class="block" id="viewer" v-model="viewer">
                <div class="ml-1">Viewer</div>
              </Switch>
            </div>
            <PermissionList
              class="md:basis-3/4"
              v-if="user.permissions.viewer != undefined"
              v-model="user.permissions.viewer"
            />
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
}
.otherPermissions {
  @apply w-full;
}
.admin {
  @apply opacity-50;
  @apply backdrop-brightness-95;
}
</style>
<script lang="ts">
import { computed, defineComponent, inject, ref, watch } from "vue";
import { User } from "@nitro_repo/nitro_repo-api-wrapper";
import { updatePermission } from "@nitro_repo/nitro_repo-api-wrapper";
import Switch from "@/components/common/forms/Switch.vue";
import { useRouter } from "vue-router";
import PermissionList from "./PermissionList.vue";
export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup(props) {
    const permissions = ref(props.user.permissions);
    const token: string | undefined = inject("token");
    const admin = computed(() => {
      return permissions.value.admin;
    });
    const deployer = ref(permissions.value.deployer != undefined);
    const viewer = ref(permissions.value.viewer != undefined);

    watch(deployer, () => {
      if (deployer.value == true) {
        if (permissions.value.deployer == undefined) {
          permissions.value.deployer = {
            permissions: [],
          };
        }
      } else {
        permissions.value.deployer = undefined;
      }
    });
    watch(viewer, () => {
      if (viewer.value == true) {
        if (permissions.value.viewer == undefined) {
          permissions.value.viewer = {
            permissions: [],
          };
          console.log(permissions.value.viewer);
        }
      } else {
        permissions.value.viewer = undefined;
      }
    });

    return {
      token: token as string,
      permissions,
      admin,
      deployer,
      viewer,
    };
  },
  methods: {
    async updatePermissions() {
      const response = await updatePermission(
        this.user.username,
        this.permissions,
        this.token
      );
      if (response.ok) {
        this.$notify({
          title: "Updated Permissions",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Permissions",
          text: JSON.stringify(response.val.user_friendly_message),
          r,
        });
      }
    },
  },
  components: { Switch, PermissionList },
});
</script>
