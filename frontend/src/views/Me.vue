<template>
  <div v-if="user != undefined" class="flex flex-wrap flex-row">
    <div class="lg:basis-1/2 flex flex-wrap settingContent mb-4">
      <form class="settingContent" @submit.prevent>
        <h2 class="settingHeader">User General</h2>
        <div class="settingBox">
          <label class="nitroLabel" for="grid-name"> Name </label>
          <input
            class="nitroTextInput"
            id="grid-name"
            type="text"
            disabled
            v-model="user.name"
          />
          <label class="nitroLabel" for="grid-name"> Email </label>
          <input
            class="nitroTextInput"
            id="grid-name"
            type="text"
            disabled
            v-model="user.email"
          />
        </div>
      </form>
    </div>
    <div class="lg:basis-1/2 flex flex-wrap settingContent mb-4">
      <form class="settingContent" @submit.prevent="updatePassword()">
        <h2 class="settingHeader">Update Password</h2>

        <div class="settingContent">
          <div class="settingBox">
            <label class="nitroLabel" for="grid-name"> Password </label>
            <input
              class="nitroTextInput"
              id="grid-name"
              type="password"
              v-model="password.password"
            />
            <label class="nitroLabel" for="grid-name"> Confirm Password </label>
            <input
              class="nitroTextInput"
              id="grid-name"
              type="password"
              v-model="password.confirm"
            />
          </div>
        </div>
        <div class="settingBox">
          <button :disabled="!canSubmitPassword" class="nitroButton">
            Update Password
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { useUserStore } from "@/store/user";
import { updateMyPassword, User } from "@nitro_repo/nitro_repo-api-wrapper";
import { computed, defineComponent, inject, onMounted, ref } from "vue";
import { useRouter } from "vue-router";

export default defineComponent({
  setup() {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const userStore = useUserStore();
    const user = computed(() => {
      return userStore.$state.user;
    });
    const date = computed(() => {
      return userStore.$state.date;
    });
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

    return { user, date, canSubmitPassword, password, token };
  },
  methods: {
    async updatePassword() {
      if (!this.canSubmitPassword) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
        return;
      }
      const response = await updateMyPassword(
        this.password.password,
        this.token as string
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
