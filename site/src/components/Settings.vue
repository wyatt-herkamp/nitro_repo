<template>
  <el-menu
    :aria-expanded="true"
    default-active="0"
    class="el-menu-demo"
    mode="horizontal"
  >
    <el-menu-item @click="tab = 0" index="0">General Settings</el-menu-item>
    <el-menu-item @click="tab = 1" index="1">Email</el-menu-item>
  </el-menu>
  <div v-if="tab == 0">
    <el-form label-position="top" :model="settingForm" label-width="120px">
      <el-form-item>
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
    <el-form label-position="top" :model="email" label-width="120px">
      <el-form-item>
        <!--Yeah, I know. But please don't judge -->
        <el-button type="primary" @click="updateEmail"
          >Update Email</el-button
        >
      </el-form-item>
    </el-form>
  </div>
</template>

<script lang="ts">
import axios from "axios";
import {
  AuthToken,
  BasicResponse,
  RepoSettings,
  Repository,
  DEFAULT_STORAGE,
  Storage,
  User,
} from "@/backend/Response";
import router from "@/router";
import http from "@/http-common";
import { computed, defineComponent, onMounted, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useRouter } from "vue-router";
import { getStorage } from "@/backend/api/Storages";

export default defineComponent({
  setup() {
    let settingForm = ref({});
    let email = ref({
      "email.host": "",
    });

    const tab = ref(0);

    return { settingForm, email, tab };
  },
  methods: {
    settingButton() {
      return true;
    },
    async onSettingSubmit() {
      for (const [k, v] of Object.entries(this.settingForm)) {
        let update = {
          value: v,
        };
        let body = JSON.stringify(update);
        console.log(body);
        const res = await http.post(
          "/api/admin/setting/" + k + "/update",
          body,
          {
            headers: {
              "Content-Type": "application/json",
              Authorization: "Bearer " + this.$cookie.getCookie("token"),
            },
          }
        );
        if (res.status != 200) {
          console.log("Data" + res.data);
          return;
        }
        const result = res.data;
        let value = JSON.stringify(result);
        console.log(value);

        let response: BasicResponse<unknown> = JSON.parse(value);

        if (response.success) {
          this.$notify({
            title: "Updated Setting " + k,
            type: "success",
          });
        } else {
          this.$notify({
            title: "Unable to update Setting " + k,
            type: "error",
          });
        }
      }
    },
    async updateEmail() {
      for (const [k, v] of Object.entries(this.email)) {
        let update = {
          value: v,
        };
        let body = JSON.stringify(update);
        console.log(body);
        const res = await http.post(
          "/api/admin/setting/" + k + "/update",
          body,
          {
            headers: {
              "Content-Type": "application/json",
              Authorization: "Bearer " + this.$cookie.getCookie("token"),
            },
          }
        );
        if (res.status != 200) {
          console.log("Data" + res.data);
          return;
        }
        const result = res.data;
        let value = JSON.stringify(result);
        console.log(value);

        let response: BasicResponse<unknown> = JSON.parse(value);

        if (response.success) {
          this.$notify({
            title: "Updated Setting " + k,
            type: "success",
          });
        } else {
          this.$notify({
            title: "Unable to update Setting " + k,
            type: "error",
          });
        }
      }
    },
  },
});
</script>
<style scoped></style>
