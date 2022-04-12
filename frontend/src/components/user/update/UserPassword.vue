<template>
  <div class="settingContent">
    <h2 class="text-white m-3 text-left">
      Update {{ user.username }}'s Password
    </h2>

    <div class="flex flex-wrap mb-6 justify-center">
      <div class="settingBox">
        <label for="grid-name"> Password </label>
        <input
          class="nitroTextInput"
          id="grid-name"
          type="password"
          v-model="password.password"
        />
      </div>
      <div class="settingBox">
        <label for="grid-name"> Confirm Password </label>
        <input
          class="nitroTextInput"
          id="grid-name"
          type="password"
          v-model="password.confirm"
        />
      </div>
      <div class="settingBox">
        <button class="nitroButton" @click="updatePassword()">
          Update Password
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, inject, ref } from "vue";
import { User } from "nitro_repo-api-wrapper";
import { updateOtherPassword } from "nitro_repo-api-wrapper";
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
    return { password,token: token as string};
  },
  methods: {
    async updatePassword() {
      if (this.password.password !== this.password.confirm) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
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
});
</script>
