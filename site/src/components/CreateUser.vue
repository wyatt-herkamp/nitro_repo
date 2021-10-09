

<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
      <el-alert
        v-if="form.error.length != 0"
        :title="form.error"
        type="error"
      />
      <el-form label-position="top" :model="form" label-width="120px">
        <el-form-item label="Email">
          <el-input v-model="form.email" autocomplete="email"></el-input>
        </el-form-item>
        <el-form-item label="Name">
          <el-input v-model="form.name"></el-input>
        </el-form-item>
        <el-form-item label="Username">
          <el-input v-model="form.username"></el-input>
        </el-form-item>
        <el-form-item label="Passowrd">
          <el-input
            v-model="this.form.password.password"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item label="Confirm Password">
          <el-input
            v-model="form.password.password_two"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="onSubmit">Login</el-button>
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import axios from "axios";
import { AuthToken, BasicResponse } from "@/backend/Response";
import router from "@/router";
import http from "@/http-common";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";

export default defineComponent({
  setup() {
    let form = ref({
      error: "",
      name: "",
      username: "",
      email: "",
      password: {
        password: "",
        password_two: "",
      },
      permissions: { deployer: false, admin: false },
    });
    return { form };
  },
  methods: {
    async onSubmit() {
      let newUser = {
        name: this.form.name,
        username: this.form.username,
        email: this.form.email,
        password: this.form.password,
        permissions: { deployer: false, admin: false },
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post("/api/admin/user/add", body, {
        headers: {
          "Content-Type": "application/json",
          Authorization: "Bearer " + this.$cookie.getCookie("token"),
        },
      });
      if (res.status != 200) {
        console.log("Data" + res.data);
        return;
      }
      const result = res.data;
      let value = JSON.stringify(result);
      console.log(value);

      let response: BasicResponse<unknown> = JSON.parse(value);

      if (response.success) {
        let loginRequest = response as BasicResponse<AuthToken>;
        let date = new Date(loginRequest.data.expiration * 1000);
        this.$cookie.setCookie("token", loginRequest.data.token, {
          expire: date,
          sameSite: "lax",
        });
        router.push("/");
      } else {
        this.form.error = "Invalid Password or Username";
      }
    },
  },
});
</script>
<style scoped>
</style>