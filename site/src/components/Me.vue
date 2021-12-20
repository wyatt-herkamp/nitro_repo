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
    <el-form label-position="top" label-width="120px">
      <el-form-item>
        <el-form-item label="Name">
          <el-input disabled v-model="user.name"></el-input>
        </el-form-item>
        <el-form-item label="Email">
          <el-input disabled v-model="user.email"></el-input>
        </el-form-item>
      </el-form-item>
    </el-form>
  </div>
  <div v-if="tab == 1">
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
    <el-form label-position="top" label-width="120px">
      <el-form-item>
        <el-form-item label="Admin">
          <el-switch disabled v-model="user.permissions.admin" />
        </el-form-item>
        <el-form-item label="Deployer">
          <el-switch disabled v-model="user.permissions.deployer" />
        </el-form-item>
      </el-form-item>
    </el-form>
  </div>
</template>

<script lang="ts">
import {User,} from "@/backend/Response";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {getUser} from "@/backend/api/User";
import {updateMyPassword} from "@/backend/api/backend/User";
import {ANON_USER} from "@/store/user";

export default defineComponent({
  setup() {
    let password = ref({
      password: "",
      confirm: "",
      error: "",
    });

    const isLoading = ref(false);
    const cookie = useCookie();
    const tab = ref(0);
    const user = ref<User>(ANON_USER);
    const loadUser = async () => {
      isLoading.value = true;
      try {
        let value = await getUser(cookie.getCookie("token"));

        user.value = value as User;

        isLoading.value = false;
      } catch (e) {}
    };
    loadUser();

    return { user, password, tab, isLoading };
  },
  methods: {
    async updatePassword() {
      if (this.password.password != this.password.confirm) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
      }
      const response = await updateMyPassword(
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
