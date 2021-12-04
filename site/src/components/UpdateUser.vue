<template>
  <el-menu
    :aria-expanded="true"
    default-active="0"
    class="el-menu-demo"
    mode="horizontal"
    v-loading="isLoading"
  >
    <el-menu-item @click="tab = 0" index="0">General Settings</el-menu-item>
    <el-menu-item @click="tab = 1" index="1">Password</el-menu-item>
    <el-menu-item @click="tab = 2" index="2">Permissions</el-menu-item>
  </el-menu>
  <div v-if="tab == 0">
    <el-alert
      v-if="settingForm.error.length != 0"
      :title="settingForm.error"
      type="error"
      :closable="false"
    />
    <el-alert
      v-if="settingForm.success.length != 0"
      :title="settingForm.success"
      type="success"
      :closable="false"
    />
    <el-form label-position="top" :model="settingForm" label-width="120px">
      <el-form-item>
        <el-form-item label="Name">
          <el-input v-model="settingForm.name"></el-input>
        </el-form-item>
        <el-form-item label="Email">
          <el-input v-model="settingForm.email"></el-input>
        </el-form-item>
        <!--Yeah, I know. But please don't judge -->
        <el-button
          :disabled="settingButton()"
          type="primary"
          @click="onSettingSubmit"
          >Update Settings</el-button
        >
      </el-form-item>
    </el-form>
  </div>
  <div v-if="tab == 1">
    <el-alert
      v-if="settingForm.error.length != 0"
      :title="settingForm.error"
      type="error"
      :closable="false"
    />
    <el-form label-position="top" :model="password" label-width="120px">
      <el-form-item>
        <el-form-item label="Password">
          <el-input
            v-model="this.password.password"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item label="Confirm Password">
          <el-input
            v-model="password.confirm"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <!--Yeah, I know. But please don't judge -->
        <el-button
          :disabled="
            password.password.length == 0 ||
            password.password != password.confirm
          "
          type="primary"
          @click="updatePassword"
          >Update Passwords</el-button
        >
      </el-form-item>
    </el-form>
  </div>
  <div v-if="tab == 2">
    <el-form label-position="top" :model="user.permissions" label-width="120px">
      <el-form-item>
        <el-form-item label="Admin">
          <el-switch
            v-model="user.permissions.admin"
            @change="onPermissionUpdate('admin')"
          />
        </el-form-item>
        <el-form-item label="Deployer">
          <el-switch
            v-model="user.permissions.deployer"
            @change="onPermissionUpdate('deployer')"
          />
        </el-form-item>
      </el-form-item>
    </el-form>
  </div>
</template>

<script lang="ts">
import axios from "axios";
import {
  BasicResponse,
  RepoSettings,
  Repository,
  DEFAULT_STORAGE,
  Storage,
  User,
  UserListResponse,
} from "@/backend/Response";
import router from "@/router";
import http from "@/http-common";
import { computed, defineComponent, onMounted, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useRouter } from "vue-router";
import { getStorage } from "@/backend/api/Storages";
import { getUser, getUserByID } from "@/backend/api/User";
import {
  updateNameAndEmail,
  updateOtherPassword,
  updatePermission,
} from "@/backend/api/admin/User";
import { ANON_USER } from "@/store/user";

export default defineComponent({
  props: {
    userResponse: {
      required: false,
      type: Object as () => UserListResponse,
    },
  },

  setup(props) {
    let settingForm = ref({
      email: "",
      name: "",
      error: "",
      success: "",
    });
    let password = ref({
      password: "",
      confirm: "",
      error: "",
    });
    const isLoading = ref(false);
    const error = ref("");
    const cookie = useCookie();
    const tab = ref(0);
    const user = ref<User>(ANON_USER);
    const loadUser = async () => {
      isLoading.value = true;
      try {
        let value = (await getUserByID(
          cookie.getCookie("token"),
          (props.userResponse as UserListResponse).id
        )) as User;

        user.value = value as User;

        isLoading.value = false;
        settingForm.value = {
          email: user.value.email,
          name: user.value.name,
          error: "",
          success: "",
        };
        password.value = {
          password: "",
          confirm: "",
          error: "",
        };
      } catch (e) {
        error.value = "";
      }
    };
    loadUser();

    return { user, settingForm, password, tab, isLoading };
  },
  methods: {
    settingButton() {
      if (this.user == undefined) return true;
      let user = this.user as User;
      return (
        user.name == this.settingForm.name &&
        user.email == this.settingForm.email
      );
    },
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
        this.settingForm.name,
        this.settingForm.email,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as User;
        this.$notify({
          title: "User Updated",
          type: "success",
        });
        this.user = data;
      } else {
        this.$notify({
          title: "Unable Update User",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async onPermissionUpdate(permission: string) {
      if (this.user == undefined) {
        this.$notify({
          title: "Unable Update Permission",
          text: "User is still undefined",
          type: "error",
        });
        return;
      }
      let user = this.user as User;
      let value: boolean = user.permissions[permission];

      const response = await updatePermission(
        this.user.username,
        permission,
        value,
        this.$cookie.getCookie("token")
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
    async updatePassword() {
      if (this.user == undefined) {
        this.$notify({
          title: "Unable Update Password",
          text: "User is still undefined",
          type: "error",
        });
        return;
      }
      if (this.password.password != this.password.confirm) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
      }
      const response = await updateOtherPassword(
        this.user.username,
        this.password.password,
        this.$cookie.getCookie("token")
      );
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
<style scoped></style>
