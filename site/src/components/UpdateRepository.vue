<template>
  <el-tabs v-model="activeName" @tab-click="handleClick" >
    <el-tab-pane label="General" name="general" >
      <el-form label-position="top" :model="settingForm" >
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
          <el-button type="primary" @click="onSettingSubmit"
            >Update Settings</el-button
          >
        </el-form-item>
      </el-form></el-tab-pane
    >
    <el-tab-pane label="Frontend" name="frontend">
      <el-form label-position="top" :model="frontendForm" >
        <el-form-item>
          <el-form-item label="Frontend Page Enabled">
            <el-switch v-model="frontendForm.frontend_enabled" />
          </el-form-item>
          <el-form-item label="Page Provider">
            <el-select v-model="frontendForm.frontend_page_provider">
              <el-option label="Readme Sent" value="ReadmeSent"></el-option>
              <el-option label="Readme Git" value="ReadmeGit"></el-option>
              <el-option label="None" value="None"></el-option>
            </el-select>
          </el-form-item>
          <el-divider></el-divider>

          <img :src="exampleBadgeURL" />
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
          <el-button type="primary" @click="submitFrontend"
            >Update Frontend Settings</el-button
          >
        </el-form-item>
      </el-form></el-tab-pane
    >
    <el-tab-pane label="Security" name="security">
      <el-form label-position="top" :model="securityForm" >
        <el-form-item label="Visibility">
          <el-select v-model="securityForm.visibility">
            <el-option label="Public" value="Public"></el-option>
            <el-option label="Private" value="Private"></el-option>
            <el-option label="Hidden" value="Hidden"></el-option>
          </el-select>
        </el-form-item>

          <!--Yeah, I know. But please don't judge -->
          <el-button disabled type="primary" @click="submitSecurity"
            >Update Security Settings</el-button
          >
      </el-form>
    </el-tab-pane>
    <el-tab-pane label="Upload" name="upload"> HI </el-tab-pane>
  </el-tabs>
</template>
<style scoped>
</style>
<script lang="ts">
import {BasicResponse, DEFAULT_STORAGE, Repository, RepositoryListResponse,} from "@/backend/Response";
import http, {apiURL} from "@/http-common";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {useRouter} from "vue-router";
import {getStorage} from "@/backend/api/Storages";
import {getRepoByID} from "@/backend/api/Repository";

export default defineComponent({
  props: {
    repo: {
      required: true,
      type: Object as () => RepositoryListResponse,
    },
  },

  setup(props) {
    const router = useRouter();

    let settingForm = ref({
      active: false,
      policy: "props.repo.settings.policy",
      error: "",
    });
    let frontendForm = ref({
      frontend_enabled: false,
      frontend_page_provider: "props.repo.settings.frontend.page_provider",
      badge_style: "props.repo.settings.badge.style",
      badge_label_color: "props.repo.settings.badge.label_color",
      badge_color: "props.repo.settings.badge.color",

      error: "",
    });
    let securityForm = ref({
      open_to_all_deployers: false,
      open_to_all_readers: false,
      visibility: "props.repo.security.visibility",
      error: "",
    });
    let repository = ref<Repository | undefined>(undefined);
    let date = ref<string | undefined>(undefined);
    const cookie = useCookie();
    const isLoading = ref(false);
    const tab = ref(0);
    const activeName = ref("first");
    const error = ref("");
    let storage = ref(DEFAULT_STORAGE);
    const exampleBadgeURL = ref("");

    const getStorageByID = async () => {
      isLoading.value = true;
      try {
        const value = await getStorage(
          cookie.getCookie("token"),
          props.repo.storage
        );
        storage.value = value;
        isLoading.value = false;
      } catch (e) {
        error.value = "";
      }
    };
    getStorageByID();
    console.log(apiURL);
    const getRepo = async () => {
      isLoading.value = true;
      try {
        const value = (await getRepoByID(
          cookie.getCookie("token"),
          props.repo.id
        )) as Repository;
        repository.value = value;
        exampleBadgeURL.value =
            apiURL +
            "/badge/" +
          storage.value.name +
          "/" +
          props.repo.name +
          "/nitro_repo_example/badge.svg";
        date.value = new Date(repository.value.created).toLocaleDateString(
          "en-US"
        );
        isLoading.value = false;
        settingForm.value = {
          active: value.settings.active,
          policy: value.settings.policy,
          error: "",
        };
        frontendForm.value = {
          frontend_enabled: value.settings.frontend.enabled,
          frontend_page_provider: value.settings.frontend.page_provider,
          badge_style: value.settings.badge.style,
          badge_label_color: value.settings.badge.label_color,
          badge_color: value.settings.badge.color,

          error: "",
        };
        securityForm.value = {
          open_to_all_deployers: value.security.open_to_all_deployers,
          open_to_all_readers: value.security.open_to_all_readers,
          visibility: value.security.visibility,
          error: "",
        };
        repository.value = value;
      } catch (e) {
        error.value = "";
      }
    };
    getRepo();

    return {
      settingForm,
      securityForm,
      frontendForm,
      storage,
      tab,
      activeName,
      date,
      exampleBadgeURL,
      repository,
      router,
    };
  },
  methods: {
    handleClick(tab: any, event: any) {
      console.log(tab.paneName);
      if (tab.paneName === "upload") {
        this.router.replace(
          "/upload/" + this.storage.name + "/" + this.repo.name
        );
      }
    },
    async updateValues(repository: Repository) {
      this.settingForm = {
        active: repository.settings.active,
        policy: repository.settings.policy,
        error: "",
      };
      this.frontendForm = {
        frontend_enabled: repository.settings.frontend.enabled,
        frontend_page_provider: repository.settings.frontend.page_provider,
        badge_style: repository.settings.badge.style,
        badge_label_color: repository.settings.badge.label_color,
        badge_color: repository.settings.badge.color,

        error: "",
      };
      this.securityForm = {
        open_to_all_deployers: repository.security.open_to_all_deployers,
        open_to_all_readers: repository.security.open_to_all_readers,
        visibility: repository.security.visibility,
        error: "",
      };
      this.repository = repository;
    },
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
          "/modify/settings/general",
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
        this.updateValues(response.data as Repository);
        this.$notify({
          title: "Updated Repository",
          type: "success",
        });
      } else {
        this.settingForm.error = "Unable to Update Storage";
      }
    },
    async submitSecurity() {
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
          "/modify/settings/general",
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
        this.updateValues(response.data as Repository);
        this.$notify({
          title: "Updated Repository",
          type: "success",
        });
      } else {
        this.settingForm.error = "Unable to Update Storage";
      }
    },
    async submitFrontend() {
      if (this.storage.id == 0) {
        return;
      }
      let newUser = {
        frontend: {
          enabled: this.frontendForm.frontend_enabled,
          page_provider: this.frontendForm.frontend_page_provider,
        },
        badge: {
          style: this.frontendForm.badge_style,
          label_color: this.frontendForm.badge_label_color,
          color: this.frontendForm.badge_color,
        },
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post(
        "/api/admin/repository/" +
          this.storage.name +
          "/" +
          this.repo.name +
          "/modify/settings/frontend",
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
        this.updateValues(response.data as Repository);
        this.$notify({
          title: "Updated Repository",
          type: "success",
        });
      } else {
        this.settingForm.error = "Unable to Update Storage";
      }
    },
  },
});
</script>
