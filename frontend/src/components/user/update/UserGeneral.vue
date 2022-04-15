<template>
  <div class="flex flex-wrap flex-row">
    <div class="lg:basis-1/2 flex flex-wrap settingContent mb-4">
      <form class="settingContent" @submit.prevent="updatePassword()">
        <h2 class="settingHeader">User General</h2>
        <div class="settingBox">
          <label class="nitroLabel"  for="grid-name"> Name </label>
          <input
            class="nitroTextInput"
            id="grid-name"
            type="text"
            v-model="user.name"
          />
          <label class="nitroLabel" for="grid-name"> Email </label>
          <input
            class="nitroTextInput"
            id="grid-name"
            type="text"
            v-model="user.email"
          />
        </div>
        <div class="settingBox">
          <button class="nitroButton" @click="onSettingSubmit()">
            Update User
          </button>
        </div>
      </form>
    </div>
    <div class="lg:basis-1/2 flex flex-wrap settingContent mb-4">
      <form class="settingContent" @submit.prevent="updatePassword()">
        <h2 class="settingHeader">Update Password</h2>

        <div class="settingContent">
          <div class="settingBox">
            <label  class="nitroLabel"  for="grid-name"> Password </label>
            <input
              class="nitroTextInput"
              id="grid-name"
              type="password"
              v-model="password.password"
            />
            <label class="nitroLabel"  for="grid-name"> Confirm Password </label>
            <input
              class="nitroTextInput"
              id="grid-name"
              type="password"
              v-model="password.confirm"
            />
          </div>
        </div>
        <div class="settingBox">
          <button
            :disabled="!canSubmitPassword"
            class="nitroButton"
            @click="updatePassword()"
          >
            Update Password
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, inject, ref } from "vue";
import { updateOtherPassword, User } from "nitro_repo-api-wrapper";
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
    let password = ref({
      password: "",
      confirm: "",
    });
    let canSubmitPassword = computed(() => {
      if (password.value.password.length >= 1) {
        if (password.value.password === password.value.confirm) {
          return true;
        }
      }
      return false;
    });
    const date = new Date(props.user.created).toLocaleDateString("en-US");
    return { date, token: token as string, password, canSubmitPassword };
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
    async updatePassword() {
      if (!this.canSubmitPassword) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
        return;
      }
      const response = await updateOtherPassword(
        this.user.username,
        this.password.password,
        this.token
      );
      this.password.password = "";
      this.password.confirm = "";
      if (response.ok) {
        let data = response.val as User;
        this.$notify({
          title: "Password Updated",
          type: "success",
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
