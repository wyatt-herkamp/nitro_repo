<template>
  <Tabs v-model="tab">
    <Tab name="general">General</Tab>
    <Tab name="password">Change Password</Tab>
    <Tab name="permissions">View Permissions</Tab>
    <Tab name="api_keys">API Keys</Tab>
  </Tabs>
  <div v-if="user !== undefined">
    <div v-if="tab === 'general'">
      <div>Welcome, {{ user.username }}!</div>
    </div>
    <div v-if="tab === 'password'">
      <div>
        <h1>Change Password</h1>
        <PasswordBox v-model="password.newPassword">
          <div class="formGroup">
            <label class="formLabel" for="grid-password"> Password </label>
            <input
              class="formInput"
              id="grid-password"
              type="password"
              v-model="password.oldPassword"
            />
          </div>
        </PasswordBox>
      </div>
    </div>
    <div v-else-if="tab === 'permissions'">
      <Permissions :disabled="true" v-model="user.permissions"></Permissions>
    </div>
    <div v-else-if="tab === 'api_keys'">
      <MyAPIKeys />
    </div>
  </div>
</template>

<script lang="ts">
import { useUserStore } from "@/store/user";
import { computed, defineComponent, ref } from "vue";
import PasswordBox from "@/components/user/PasswordBox.vue";
import httpCommon from "@/http-common";
import Permissions from "@/components/user/update/Permissions.vue";
import Tabs from "@/components/common/tabs/Tabs.vue";
import Tab from "@/components/common/tabs/Tab.vue";
import MyAPIKeys from "@/components/user/keys/MyAPIKeys.vue";

export default defineComponent({
  components: {MyAPIKeys, Tab, Tabs, Permissions, PasswordBox },
  setup() {
    const tab = ref("general");
    const userStore = useUserStore();
    const user = computed(() => {
      return userStore.$state.user;
    });
    const date = computed(() => {
      return userStore.$state.date;
    });
    const password = ref({
      oldPassword: "",
      newPassword: "",
    });
    const canSubmitPassword = computed(() => {
      return (
        password.value.newPassword.length >= 1 &&
        password.value.newPassword !== "" &&
        password.value.oldPassword !== ""
      );
    });

    return { user, date, canSubmitPassword, password, tab };
  },
  methods: {
    async updatePassword() {
      if (this.user == undefined) {
        console.error("User is undefined");
        return;
      }
      await httpCommon.apiClient.put("api/me/password", {
        username: this.user.username,
        password: this.password.oldPassword,
        secure_data: this.password.newPassword,
      });
    },
  },
});
</script>
