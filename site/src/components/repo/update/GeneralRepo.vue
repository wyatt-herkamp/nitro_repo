<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Rules</h2>

    <div class="flex flex-wrap mb-6 justify-center">
      <div class="settingBox">
        <label class="nitroLabel" for="grid-name"> name </label>
        <input
          class="disabled nitroTextInput"
          id="grid-name"
          type="text"
          v-model="repository.name"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-Storage"> Storage </label>
        <input
          class="disabled nitroTextInput"
          id="grid-Storage"
          type="text"
          v-model="repository.storage"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-created"> Date Created </label>
        <input
          class="disabled nitroTextInput"
          id="grid-created"
          type="text"
          v-model="date"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-type"> Repo Type</label>
        <input
          class="disabled nitroTextInput"
          id="grid-type"
          type="text"
          v-model="repository.repo_type"
          disabled
        />
      </div>
    </div>
    <h2 class="settingHeader">Repository General Properties</h2>
    <div class="flex flex-wrap mb-6">
      <div class="settingBox">
        <label class="nitroLabel" for="grid-policy"> Repo Policy</label>
        <select v-model="repository.settings.policy" class="nitroSelectBox" @change="updatePolicy()" >
          <option>Mixed</option>
          <option>Release</option>
          <option>Snapshot</option>
        </select>
      </div>

      <div class="settingBox">
        <label class="nitroLabel" for="grid-active">Repo Active</label>
        <select v-model="repository.settings.active" class="nitroSelectBox" @change="updateActiveStatus()" >
          <option>true</option>
          <option>false</option>
        </select>
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import { Repository } from "@/backend/Response";
import { setActiveStatus, setPolicy } from "@/backend/api/admin/Repository";
export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  setup(props) {
    const date = new Date(props.repository.created).toLocaleDateString("en-US");
    return { date };
  },
  methods: {
    async updateActiveStatus() {
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }

      const response = await setActiveStatus(
                this.repository.storage,this.repository.name,
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
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }
      const response = await setPolicy(
                this.repository.storage,this.repository.name,
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
  },
});
</script>
