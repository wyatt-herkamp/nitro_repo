<template>
  <div class="flex flex-wrap flex-row">
    <div class="flex flex-wrap settingContent mb-4">
      <h3 class="settingHeader">User General</h3>

      <div class="flex flex-row settingContent">
        <div class="settingBox">
          <label for="grid-name"> Name </label>
          <input
            class="nitroTextInput"
            id="grid-name"
            type="text"
            v-model="user.name"
          />
        </div>
        <div class="settingBox">
          <label for="grid-name"> Email </label>
          <input
            class="nitroTextInput"
            id="grid-name"
            type="text"
            v-model="user.email"
          />
        </div>
      </div>
      <div class="settingBox">
        <button class="nitroButton" @click="onSettingSubmit()">
          Update User
        </button>
      </div>
    </div>
    <div class="flex flex-wrap flex-row mr-3 h-96 w-96 bg-slate-800">
      <div class="flex flex-wrap flex-col">
        <h3 class="settingHeader h-min">User permission</h3>
        <div class="flex flex-col settingContent">
          <Switch
            id="admin"
            @change="onPermissionUpdate('admin')"
            v-model="user.permissions.admin"
          >
            <div class="ml-3 text-gray-700 font-medium">Admin</div>
          </Switch>
          <Switch
            id="deplyoer"
            @change="onPermissionUpdate('deployer')"
            v-model="user.permissions.deployer"
          >
            <div class="ml-3 text-gray-700 font-medium">Deployer</div>
          </Switch>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, inject } from "vue";
import { User } from "nitro_repo-api-wrapper";
import { updateNameAndEmail, updatePermission } from "nitro_repo-api-wrapper";
import Switch from "@/components/common/forms/Switch.vue";
import { useRouter } from "vue-router";
export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup(props) {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const date = new Date(props.user.created).toLocaleDateString("en-US");
    return { date, token: token as string };
  },
  methods: {
    async onSettingSubmit() {
      if (this.user == undefined) {
        this.$notify({
          title: "Unable Update Name and Email",
          text: "User is still undefined",
          type: "error",
        });
        return;
      }
      const response = await updateNameAndEmail(
        this.user.username,
        this.user.name,
        this.user.email,
        this.token
      );
      if (response.ok) {
        let data = response.val as User;
        this.$notify({
          title: "User Updated",
          type: "success",
        });
      } else {
        this.$notify({
          title: "Unable Update User",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async onPermissionUpdate(permission: string) {
      let value: boolean = this.user.permissions[permission];
      const response = await updatePermission(
        this.user.username,
        permission,
        value,
        this.token
      );
      if (response.ok) {
        this.$notify({
          title: "Updated Permission: " + permission + ": " + value,
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Password",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
  components: { Switch },
});
</script>
