<template>
  <div class="flex flex-wrap flex-row">
    <div class="lg:basis-1/2 flex flex-wrap settingContent mb-4">
      <form class="settingContent" @submit.prevent="updateGenericProperties()">
        <h2 class="settingHeader">User General</h2>
        <div class="settingBox">
          <label class="nitroLabel" for="grid-name"> Name </label>
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
          <button class="nitroButton">Update User</button>
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
    <div class="flex-grow lg:m-5 border-slate-500 border-2 rounded-lg">
      <h1
        class="text-left ml-2 md:text-4xl text-3xl border-b-2 border-slate-500 p-2"
      >
        Permissions
      </h1>
      <Permissions :user="user" />
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import Permissions from "./Permissions.vue";
import { User } from "@/types/user";

export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup(props) {
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
    return { date, password, canSubmitPassword };
  },
  methods: {
    async updateGenericProperties() {
      // TODO generic user data
    },
    async updatePassword() {
      if (!this.canSubmitPassword) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
        return;
      }
      // TODO update password
    },
  },
  components: { Permissions },
});
</script>
