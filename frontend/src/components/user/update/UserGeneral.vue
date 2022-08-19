<template>
  <Tabs>
    <Tabs v-model="tab">
      <Tab name="general">General</Tab>
      <Tab name="permissions">View Permissions</Tab>
      <Tab name="api_keys">API Keys</Tab>
      <Tab name="delete_user" @click="deleteUser = true">Delete User</Tab>
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
  <vue-final-modal
    v-model="deleteUser"
    classes="flex justify-center items-center"
    @click-outside="deleteUser = false"
  >
    <div class="modal">
      <div class="header">
        <h1>Delete User</h1>
        <button @click="deleteUser = false">🗙</button>
      </div>
      <div class="content">
        <p class="text-quaternary">
          Are you sure you want to delete this user?
        </p>
        <button
          class="nitroButton bg-red-500 mx-5 px-5"
          @click="deleteUser = false"
        >
          Cancel
        </button>
        <button
          class="nitroButton bg-green-500 mx-5 px-5 mb-0"
          @click="deleteTheUser()"
        >
          Delete
        </button>
      </div>
    </div>
  </vue-final-modal>
</template>

<script lang="ts">
import { computed, defineComponent, inject, ref } from "vue";

import { useRouter } from "vue-router";
import Permissions from "./Permissions.vue";
import { User } from "@/types/userTypes";
import Tabs from "@/components/common/tabs/Tabs.vue";
import Tab from "@/components/common/tabs/Tab.vue";
import httpCommon from "@/http-common";

export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup(props) {
    const tab = ref("general");
    const deleteUser = ref(false);
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
    return { tab, date, password, canSubmitPassword, deleteUser };
  },
  methods: {
    async deleteTheUser() {
      await httpCommon.apiClient
        .delete("api/admin/user/" + this.user.id)
        .then(() => {
          this.$router.push("/admin/Users");
        })
        .catch((err) => {
          this.$notify({
            type: "error",
            title: "Error",
            text: err.response.data,
          });
        });
    },
    async onSettingSubmit() {
      // TODO update user
    },
    async updatePassword() {
      // TODO update password
    },
  },
  components: { Permissions, Tab, Tabs },
});
</script>
