<template>
  <el-tabs v-model="activeName" @tab-click="handleClick" v-loading="isLoading">
    <el-tab-pane label="General" name="general">
      <el-form label-position="top" :model="repository">
        <el-form-item>
          <el-form-item label="Name">
            <el-input disabled v-model="$props.repo.name"></el-input>
          </el-form-item>
          <el-form-item label="Storage">
            <el-input v-model="repository.storage" disabled></el-input>
          </el-form-item>
          <el-form-item label="Type">
            <el-input disabled v-model="repository.repo_type"></el-input>
          </el-form-item>
          <el-form-item label="Created On">
            <el-input disabled v-model="date"></el-input>
          </el-form-item>
          <el-form-item label="Active">
            <el-switch
              v-model="repository.settings.active"
              @change="updateActiveStatus()"
            />
          </el-form-item>
          <el-form-item label="Repository Policy">
            <el-select v-model="repository.settings.policy">
              <el-option
                label="Release"
                value="Release"
                @click="updatePolicy()"
              ></el-option>
              <el-option
                label="Snapshot"
                value="Snapshot"
                @click="updatePolicy()"
              ></el-option>
              <el-option
                label="Mixed"
                value="Mixed"
                @click="updatePolicy()"
              ></el-option>
            </el-select>
          </el-form-item>
        </el-form-item> </el-form
    ></el-tab-pane>
    <el-tab-pane label="Frontend" name="frontend">
      <el-form label-position="top" :model="repository.settings">
        <el-form-item>
          <el-form-item label="Frontend Page Enabled">
            <el-switch v-model="repository.settings.frontend.enabled" />
          </el-form-item>
          <el-form-item label="Page Provider">
            <el-select v-model="repository.settings.frontend.page_provider">
              <el-option label="Readme Sent" value="ReadmeSent"></el-option>
              <el-option label="Readme Git" value="ReadmeGit"></el-option>
              <el-option label="None" value="None"></el-option>
            </el-select>
          </el-form-item>
          <el-divider></el-divider>

          <img :src="exampleBadgeURL" />
          <el-divider></el-divider>
          <el-form-item label="Badge Style ">
            <el-select v-model="repository.settings.badge.style">
              <el-option label="Flat" value="Flat"></el-option>
              <el-option label="FlatSquare" value="FlatSquare"></el-option>
              <el-option label="Plastic" value="Plastic"></el-option>
            </el-select>
          </el-form-item>
          <el-form-item label="Badge Color ">
            <el-color-picker v-model="repository.settings.badge.color" />
          </el-form-item>
          <el-form-item label="Badge Color ">
            <el-color-picker v-model="repository.settings.badge.label_color" />
          </el-form-item>

          <!--Yeah, I know. But please don't judge -->
          <el-button type="primary" @click="submitFrontend"
            >Update Frontend Settings</el-button
          >
        </el-form-item>
      </el-form></el-tab-pane
    >
    <el-tab-pane label="Security" name="security">
      <el-form label-position="top" :model="repository.security">
        <el-form-item label="Visibility">
          <el-select
            @change="updateVisibility()"
            v-model="repository.security.visibility"
          >
            <el-option label="Public" value="Public"></el-option>
            <el-option label="Private" value="Private"></el-option>
            <el-option label="Hidden" value="Hidden"></el-option>
          </el-select>
        </el-form-item>
      </el-form>
    </el-tab-pane>

    <el-tab-pane label="Deploy Settings" name="deploy">
      <el-form label-position="top" :model="repository.deploy_settings">
        <el-form-item label="Report Values">
          <el-select-v2
            v-model="repository.deploy_settings.report_generation.values"
            :options="options"
            placeholder="Please select"
            multiple
          />
        </el-form-item>
        <el-form-item label="Active">
          <el-checkbox
            v-model="repository.deploy_settings.report_generation.active"
            label="Enable Report Generation"
          ></el-checkbox>
        </el-form-item>
        <el-button type="primary" @click="updateReport()"
          >Update Report Generation</el-button
        >
      </el-form>
    </el-tab-pane>
    <el-tab-pane label="Upload" name="upload"> HI </el-tab-pane>
  </el-tabs>
</template>
<style scoped></style>
<script lang="ts">
import {DEFAULT_REPO, Repository, RepositoryListResponse,} from "@/backend/Response";
import {apiURL} from "@/http-common";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {useRouter} from "vue-router";
import {getRepoByID} from "@/backend/api/Repository";
import {
  setActiveStatus,
  setPolicy,
  setVisibility,
  updateBadge,
  updateDeployReport,
  updateFrontend,
} from "@/backend/api/admin/Repository";

export default defineComponent({
  props: {
    repo: {
      required: true,
      type: Object as () => RepositoryListResponse,
    },
  },

  setup(props) {
    const router = useRouter();
    const options = ref([
      { value: "DeployerUsername", label: "Deploy Username" },
      { value: "Time", label: "Time" },
    ]);
    let repository = ref<Repository>(DEFAULT_REPO);
    let date = ref<string | undefined>(undefined);
    const cookie = useCookie();
    const isLoading = ref(false);
    const activeName = ref("general");
    const exampleBadgeURL = ref("");


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
            props.repo.storage +
            "/" +
          props.repo.name +
          "/nitro_repo_example/badge.svg";
        date.value = new Date(repository.value.created).toLocaleDateString(
          "en-US"
        );
        isLoading.value = false;
      } catch (e) {
        console.log(e);
      }
    };
    getRepo();

    return {
      isLoading,
      activeName,
      date,
      exampleBadgeURL,
      repository,
      router,
      options,
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

    async updateActiveStatus() {
      if (this.repository.id == 0) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }

      const response = await setActiveStatus(
        this.repository.id,
        this.repository.settings.active,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        this.$notify({
          title: "Updated Repository",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async updatePolicy() {
      if (this.repository.id == 0) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }
      const response = await setPolicy(
        this.repository.id,
        this.repository.settings.policy,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        this.$notify({
          title: "Updated Repository",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async updateVisibility() {
      if (this.repository.id == 0) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }
      const response = await setVisibility(
        this.repository.id,
        this.repository.security.visibility,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        console.log(response.val.security.visibility);
        this.$notify({
          title: "Updated Repository",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async submitFrontend() {
      if (this.repository.id == 0) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }
      {
        const response = await updateFrontend(
          this.repository.id,
          this.repository.settings.frontend.enabled,
          this.repository.settings.frontend.page_provider,
          this.$cookie.getCookie("token")
        );
        if (response.ok) {
          console.log(response.val.security.visibility);
          this.$notify({
            title: "Updated Frontend",
            type: "info",
          });
        } else {
          this.$notify({
            title: "Unable Update Repository",
            text: JSON.stringify(response.val.user_friendly_message),
            type: "error",
          });
        }
      }
      {
        let response = await updateBadge(
          this.repository.id,
          this.repository.settings.badge.style,
          this.repository.settings.badge.label_color,
          this.repository.settings.badge.color,
          this.$cookie.getCookie("token")
        );
        if (response.ok) {
          console.log(response.val.security.visibility);
          this.$notify({
            title: "Updated Badge",
            type: "info",
          });
        } else {
          this.$notify({
            title: "Unable Update Repository",
            text: JSON.stringify(response.val.user_friendly_message),
            type: "error",
          });
        }
      }
    },
    async updateReport() {
      if (this.repository.id == 0) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }

      const response = await updateDeployReport(
        this.repository.id,
        this.repository.deploy_settings.report_generation.active,
        this.repository.deploy_settings.report_generation.values,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        console.log(response.val.security.visibility);
        this.$notify({
          title: "Updated Report Settings",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
