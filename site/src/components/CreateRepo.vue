

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
        <el-form-item label="Repository Type">
          <el-select
            v-model="form.type"
            placeholder="Please select your Repo Type"
          >
            <el-option label="Maven" value="maven"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="Storage">
          <el-select
            v-model="form.storage"
            placeholder="Please select your storage"
          >
            <el-option
              v-for="storage in storages.storages"
              :key="storage.id"
              :label="storage.public_name"
              :value="storage.name"
            ></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="onSubmit">Create Repository</el-button>
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import axios from "axios";
import {
  AuthToken,
  BasicResponse,
  DEFAULT_STORAGE_LIST,
} from "@/backend/Response";
import router from "@/router";
import http from "@/http-common";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { getStorages } from "@/backend/api/Storages";

export default defineComponent({
  setup() {
    let form = ref({
      name: "",
      storage: "",
      type: "",
      error: "",
    });
    const cookie = useCookie();
    const isLoading = ref(false);

    const error = ref(null);
    let storages = ref(DEFAULT_STORAGE_LIST);
    const getStorage = async () => {
      isLoading.value = true;
      try {
        const value = await getStorages(cookie.getCookie("token"));
        storages.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value = e;
      }
    };
    getStorage();
    return {
      form,
      storages,
      isLoading,
      error,
      getStorage,
    };
  },
  methods: {
    async onSubmit() {
      let newUser = {
        name: this.form.name,
        storage: this.form.storage,
        repo: this.form.type,
        settings: {},
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post("/api/admin/repository/add", body, {
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