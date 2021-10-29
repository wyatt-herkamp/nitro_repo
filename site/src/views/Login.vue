<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
      <el-alert
        v-if="form.error.length != 0"
        :title="form.error"
        type="error"
      />
      <el-form label-position="top" :model="form" label-width="120px">
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
          <el-button type="primary" @click="onSubmit">Log In</el-button>
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
      username: "",
      password: "",
      error: "",
    });
    return { form };
  },
  methods: {
    async onSubmit() {
      let newUser = {
        username: this.form.username,
        password: this.form.password,
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http
        .post("api/login", body)
        .then((res) => {
          console.log(typeof res);
          if (res.status != 200) {
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
            this.form.password = "";

            this.$notify({
              title: "Invalid Username or Password",
              type: "warn",
            });
          }
        })
        .catch((error) => {
          if (error.response) {
            if (error.response.status == 401) {
              this.form.password = "";
              this.$notify({
                title: "Invalid Username or Password",
                type: "warn",
              });
            } else {
              this.$notify({
                title: "Unkown Error Occured",
                type: "warn",
              });
            }
          }
        });
    },
  },
});
</script>
<style scoped></style>
