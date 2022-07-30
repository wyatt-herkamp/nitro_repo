<template>
  <Tabs>
    <Tabs v-model="tab">
      <Tab name="general">General</Tab>
      <Tab name="permissions">View Permissions</Tab>
      <Tab name="api_keys">API Keys</Tab>
    </Tabs>
  </Tabs>
  <div v-if="tab === 'general'" class="flex flex-wrap flex-row">
    <div class="lg:basis-1/2 flex flex-wrap settingContent mb-4">
      <form class="settingContent" @submit.prevent="onSettingSubmit()">
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
  </div>
  <div v-else-if="tab === 'permissions'">
    <Permissions v-model="user.permissions" />
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, inject, ref } from "vue";

import { useRouter } from "vue-router";
import Permissions from "./Permissions.vue";
import { User } from "@/types/userTypes";

export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup(props) {
    const tab = ref("general");
    const password = ref({
      password: "",
      confirm: "",
    });
    const canSubmitPassword = computed(() => {
      if (password.value.password.length >= 1) {
        if (password.value.password === password.value.confirm) {
          return true;
        }
      }
      return false;
    });
    const date = new Date(props.user.created).toLocaleDateString("en-US");
    return { tab, date, password, canSubmitPassword };
  },
  methods: {
    async onSettingSubmit() {
      // TODO update user
    },
    async updatePassword() {
      // TODO update password
    },
  },
  components: { Permissions },
});
</script>
