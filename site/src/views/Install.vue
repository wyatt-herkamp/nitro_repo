<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
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
            v-model="this.form.password"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item label="Confirm Password">
          <el-input
            v-model="form.confirm_password"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="onSubmit">Install</el-button>
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import axios from "axios";
import { BasicResponse } from "@/backend/Response";
import router from "@/router";
import http from "@/http-common";
import { defineComponent, ref } from "vue";

export default defineComponent({
  setup() {
    let form = ref({
      email: "",
      name: "",
      username: "",
      password: "",
      confirm_password: "",
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
        password_two: this.form.confirm_password,
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post("install", body);
      if (res.status != 200) {
        console.log("Data" + res.data);
        return;
      }
      const result = res.data;
      let value = JSON.stringify(result);
      console.log(value);

      let response: BasicResponse<unknown> = JSON.parse(value);

      if (!response.success) {
        alert("Unable to install");
      } else {
        router.push("Home");
      }
    },
  },
});
</script>
<style scoped></style>
