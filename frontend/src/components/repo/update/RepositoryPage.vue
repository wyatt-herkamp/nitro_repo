<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Rules</h2>
    <div class="settingSection">
      <div class="settingBox">
        <label for="grid-style" class="nitroLabel">Repository Page</label>
        <select v-model="settings.page_type" class="nitroTextInput">
          <option value="None">None</option>
          <option value="Markdown">Markdown</option>
        </select>
      </div>
      <div class="settingBox">
        <button class="nitroButton" @click="submitBadge()">
          Update Badge Settings
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import httpCommon from "@/http-common";

export default defineComponent({
  name: "RepositoryPage",
  props: {
    repository: {
      type: Object as () => { storage: string; name: string },
      required: true,
    },
  },
  setup(props) {
    const settings = ref({
      settings: {
        page_type: "",
      },
      page: "",
    });

    httpCommon.apiClient
      .get(
        `api/admin/repositories/${props.repository.storage}/${props.repository.name}/config/repository_page`
      )
      .then((response) => {
        settings.value = response.data;
        console.log(settings.value);
      });
    return { settings };
  },
  methods: {
    async submitBadge() {
      await httpCommon.apiClient
        .put(
          `api/admin/repositories/${this.repository.storage}/${this.repository.name}/config/repository_page`,
          this.settings
        )
        .then((response) => {
          if (response.status === 204) {
            this.$notify({
              title: "Success",
              type: "success",
            });
          } else {
            this.$notify({
              title: "Error",
              type: "error",
            });
            console.log(response);
          }
        })
        .catch((error) => {
          this.$notify({
            title: "Error",
            type: "error",
          });
          console.log(error);
        });
    },
  },
});
</script>

<style scoped></style>
