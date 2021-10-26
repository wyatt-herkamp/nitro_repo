<template>
  <el-menu
    aria-expanded="true"
    default-active="0"
    class="el-menu-demo"
    mode="horizontal"
  >
    <el-menu-item @click="tab = 0" index="0">General Settings</el-menu-item>
    <el-menu-item @click="tab = 1" index="1">Frontend Settings</el-menu-item>
    <el-menu-item @click="tab = 2" index="2">Security Settings</el-menu-item>
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
        <el-form-item label="Name">
          <el-input disabled v-model="$props.repo.name"></el-input>
        </el-form-item>
        <el-form-item label="Storage">
          <el-input disabled v-model="storage.name"></el-input>
        </el-form-item>
        <el-form-item label="Type">
          <el-input disabled v-model="$props.repo.repo_type"></el-input>
        </el-form-item>
        <el-form-item label="Created On">
          <el-input disabled v-model="date"></el-input>
        </el-form-item>
        <el-form-item label="Active">
          <el-switch v-model="settingForm.active" />
        </el-form-item>
        <el-form-item label="Repository Policy">
          <el-select v-model="settingForm.policy">
            <el-option label="Release" value="Release"></el-option>
            <el-option label="Snapshot" value="Snapshot"></el-option>
            <el-option label="Mixed" value="Mixed"></el-option>
          </el-select>
        </el-form-item>
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
    <el-form label-position="top" :model="frontendForm" label-width="120px">
      <el-form-item>
        <el-form-item label="Frontend Page Enabled">
          <el-switch v-model="frontendForm.frontend_enabled" />
        </el-form-item>
        <el-form-item label="Page Provider">
          <el-select v-model="frontendForm.frontend_page_provider">
            <el-option label="README" value="README"></el-option>
            <el-option label="None" value="None"></el-option>
          </el-select>
        </el-form-item>
        <el-divider></el-divider>
        
        <img  :src="exampleBadgeURL" />
        <el-divider></el-divider>
        <el-form-item label="Badge Style ">
          <el-select v-model="frontendForm.badge_style">
            <el-option label="Flat" value="Flat"></el-option>
            <el-option label="FlatSquare" value="FlatSquare"></el-option>
            <el-option label="Plastic" value="Plastic"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="Badge Color ">
          <el-color-picker v-model="frontendForm.badge_color" />
        </el-form-item>
        <el-form-item label="Badge Color ">
          <el-color-picker v-model="frontendForm.badge_label_color" />
        </el-form-item>

        <!--Yeah, I know. But please don't judge -->
        <el-button disabled type="primary" @click="onSettingSubmit"
          >Update Frontend Settings</el-button
        >
      </el-form-item>
    </el-form>
  </div>
  <div v-if="tab == 2">
    <el-alert
      v-if="settingForm.error.length != 0"
      :title="settingForm.error"
      type="error"
      closable="false"
    />
    <el-form label-position="top" :model="securityForm" label-width="120px">
      <el-form-item>
        <el-select v-model="securityForm.visibility">
          <el-option label="Public" value="Public"></el-option>
          <el-option label="Private" value="Private"></el-option>
          <el-option label="Hidden" value="Hidden"></el-option>
        </el-select>

        <!--Yeah, I know. But please don't judge -->
        <el-button disabled type="primary" @click="onSettingSubmit"
          >Update Security Settings</el-button
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
} from "@/backend/Response";
import router from "@/router";
import http, { baseURL } from "@/http-common";
import { computed, defineComponent, onMounted, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useRouter } from "vue-router";
import { getStorage } from "@/backend/api/Storages";

export default defineComponent({
  props: {
    repo: {
      required: true,
      type: Object as () => Repository,
    },
  },

  setup(props) {
    let settingForm = ref({
      active: props.repo.settings.active,
      policy: props.repo.settings.policy,
      error: "",
    });
    let frontendForm = ref({
      frontend_enabled: props.repo.settings.frontend.enabled,
      frontend_page_provider: props.repo.settings.frontend.page_provider,
      badge_style: props.repo.settings.badge.style,
      badge_label_color: props.repo.settings.badge.label_color,
      badge_color: props.repo.settings.badge.color,

      error: "",
    });
    let securityForm = ref({
      open_to_all_deployers: props.repo.security.open_to_all_deployers,
      open_to_all_readers: props.repo.security.open_to_all_readers,
      visibility: props.repo.security.visibility,
      error: "",
    });
    let date = new Date(props.repo.created).toLocaleDateString("en-US");
    const cookie = useCookie();
    const isLoading = ref(false);
    const tab = ref(0);
    const activeName = ref("first");
    const error = ref("");
    let storage = ref(DEFAULT_STORAGE);
    const exampleBadgeURL =  ref("");

    const getStorageByID = async () => {
      isLoading.value = true;
      try {
        const value = await getStorage(
          cookie.getCookie("token"),
          props.repo.storage
        );
        storage.value = value;
        exampleBadgeURL.value =
          baseURL + 
          "/badge/" +
          storage.value.name +
          "/" +
          props.repo.name +
          "/nitro_repo_example/badge.svg";
        isLoading.value = false;
      } catch (e) {
        error.value = "";
      }
    };
    getStorageByID();

    return {
      settingForm,
      securityForm,
      frontendForm,
      storage,
      tab,
      activeName,
      date,
      exampleBadgeURL,
    };
  },
  methods: {
    async onSettingSubmit() {
      if (this.storage.id == 0) {
        return;
      }
      let newUser = {
        active: this.settingForm.active,
        policy: this.settingForm.policy,
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post(
        "/api/admin/repository/" +
          this.storage.name +
          "/" +
          this.repo.name +
          "/modify/settings",
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
        router.push("/");
      } else {
        this.settingForm.error = "Unable to Update Storage";
      }
    },
  },
});
</script>
