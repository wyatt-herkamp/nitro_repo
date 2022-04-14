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
  },
  components: { Switch },
});
</script>
