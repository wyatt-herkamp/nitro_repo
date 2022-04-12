<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Report Generation Settings</h2>
    <div class="settingSection">
      <div class="settingBox">
        <label for="grid-policy">Report Values</label>
      </div>
    </div>
    <h2 class="settingHeader">Update</h2>
    <div class="settingSection">
      <div class="settingBox">
        <label for="grid-active">Report Generation</label>
        <select
          v-model="repository.deploy_settings.report_generation.active"
          class="nitroTextInput"
        >
          <option>true</option>
          <option>false</option>
        </select>
      </div>
      <div class="settingBox">
        <button class="nitroButton" @click="updateReport()">
          Update Report
        </button>
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { defineComponent, inject, ref } from "vue";
import { Repository } from "nitro_repo-api-wrapper";
import { updateDeployReport } from "nitro_repo-api-wrapper";
import { useRouter } from "vue-router";

export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  data() {
    const token = inject("token") as string;

    return {
      options: ["DeployerUsername", "Time"],
      token: token,
    };
  },
  methods: {
    async updateReport() {
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }

      const response = await updateDeployReport(
        this.repository.storage,
        this.repository.name,
        this.repository.deploy_settings.report_generation.active,
        this.repository.deploy_settings.report_generation.values,
        this.token
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
