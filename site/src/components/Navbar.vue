<template>
  <el-menu :default-active="activeIndex" class="el-menu-demo" mode="horizontal">
    <el-menu-item index="Home" @click="router.push('/')">Index</el-menu-item>
    <el-menu-item index="Browse" @click="router.push('/browse')"
      >Browse</el-menu-item
    >
    <el-menu-item
      v-if="user.id != 0"
      index="Admin"
      @click="router.push('/admin')"
      >Admin
    </el-menu-item>
    <el-menu-item v-else index="Login" @click="dialogVisible = true"
      >Login
    </el-menu-item>
  </el-menu>
  <el-dialog v-model="dialogVisible" title="Login" width="30%">
    <el-form
      :model="form"
      label-position="top"
      label-width="120px"
      v-on:submit="onSubmit"
    >
      <el-form-item label="Username">
        <el-input v-model="form.username"></el-input>
      </el-form-item>
      <el-form-item label="Password">
        <el-input
          v-model="this.form.password"
          placeholder="Please input password"
          show-password
        />
      </el-form-item>
      <el-form-item>
        <el-button block native-type="submit" type="primary">Log In </el-button>
      </el-form-item>
    </el-form>
  </el-dialog>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import { BasicResponse, User } from "@/backend/Response";
import http from "@/http-common";
import { login } from "@/backend/api/backend/User";
import { AuthToken } from "@/backend/api/User";

export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup() {
    let form = ref({
      username: "",
      password: "",
    });
    const router = useRouter();
    const activeIndex = ref(router.currentRoute.value.name);
    const dialogVisible = ref(false);
    return { activeIndex, router, dialogVisible, form };
  },
  methods: {
    async onSubmit(e: any) {
      e.preventDefault();
      const value = await login(this.form.username, this.form.password);
      if (value.ok) {
        let loginRequest = value.val as AuthToken;
        let date = new Date(loginRequest.expiration * 1000);
        this.$cookie.setCookie("token", loginRequest.token, {
          expire: date,
          sameSite: "lax",
        });
        this.dialogVisible = false;
        location.reload();
      } else {
        this.form.password = "";
        this.$notify({
          title: value.val.user_friendly_message,
          type: "warn",
        });
      }
    },
  },
});
</script>
