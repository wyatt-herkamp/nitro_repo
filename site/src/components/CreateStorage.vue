<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
      <el-alert
        v-if="form.error.length != 0"
        :title="form.error"
        type="error"
      />
      <el-form label-position="top" :model="form" label-width="120px">
        <el-form-item label="Name">
          <el-input v-model="form.name"></el-input>
        </el-form-item>
        <el-form-item label="Public Name">
          <el-input v-model="form.public_name"></el-input>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="onSubmit"
            >Create New Storage</el-button
          >
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import axios from "axios";
import { AuthToken, BasicResponse, Storage } from "@/backend/Response";
import router from "@/router";
import http from "@/http-common";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";

export default defineComponent({
  props: {
    updateList: {
      required: true,
      type: Function,
    },
  },
  setup() {
    let form = ref({
      name: "",
      public_name: "",
      error: "",
    });
    return { form };
  },
  methods: {
    async onSubmit() {
      let newUser = {
        name: this.form.name,
        public_name: this.form.public_name,
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post("/api/admin/storages/add", body, {
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

      let response: BasicResponse<unknown> = JSON.parse(value);

      if (response.success) {
        let value = response.data as Storage;
        this.updateList(value.id);
        this.$notify({
          title: "Storage Created",
          type: "success",
        });
      } else {
        this.$notify({
          title: "Unable to Create Storage",
          text: JSON.stringify(response.data),
          type: "error",
        });      }
    },
  },
});
</script>
<style scoped></style>
