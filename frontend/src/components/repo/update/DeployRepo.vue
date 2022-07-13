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
import { defineComponent, inject } from "vue";
import { Repository } from "@/types/repositoryTypes";

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
      // TODO update report
    },
  },
});
</script>
