<template>
  <el-menu
    aria-expanded="true"
    default-active="0"
    class="el-menu-demo"
    mode="horizontal"
  >
    <el-menu-item @click="tab = 0" index="0">General Settings</el-menu-item>
    <el-menu-item @click="tab = 1" index="1">Password</el-menu-item>
  </el-menu>
  <div v-if="tab == 0">
    <el-alert
      v-if="settingForm.error.length != 0"
      :title="settingForm.error"
      type="error"
      closable="false"
    />
    <el-form label-position="top" :model="settingForm" label-width="120px">
      <el-form-item>
        <!--Yeah, I know. But please don't judge -->
        <el-button disabled type="primary" @click="onSettingSubmit"
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
      closable="false"
    />
    <el-form label-position="top" :model="password" label-width="120px">
      <el-form-item>
        <!--Yeah, I know. But please don't judge -->
        <el-button disabled type="primary" @click="onSettingSubmit"
          >Update Passwords</el-button
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
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },

  setup(props) {
    let settingForm = ref({
      error: "",
    });
    let password = ref({
      error: "",
    });

    const tab = ref(0);

    return { settingForm, password, tab };
  },
  methods: {
    async onSettingSubmit() {
      let newUser = {};
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post("/", body, {
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
        this.settingForm.error = "Unable to Update Storage";
      }
    },
  },
});
</script>
<style scoped></style>
