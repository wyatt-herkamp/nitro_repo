<template>
  <div class="settingContent">
    <h2 class="settingHeader">Frontend Settings</h2>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import httpCommon from "@/http-common";
import { ColorPicker } from "vue-color-kit";
import "vue-color-kit/dist/vue-color-kit.css";
import { notify } from "@kyvg/vue3-notification";

export default defineComponent({
  name: "FrontendSettings",
  props: {
    repository: {
      type: Object as () => { storage: string; name: string },
      required: true,
    },
  },
  setup(props) {
    const badgeSettings = ref({
      page_provider: "",
    });

    httpCommon.apiClient
      .get(
        `api/admin/repositories/${props.repository.storage}/${props.repository.name}/config/frontend`
      )
      .then((response) => {
        badgeSettings.value = response.data;
        console.log(badgeSettings.value);
      });
    return { badgeSettings };
  },
  methods: {
    async submitBadge() {
      await httpCommon.apiClient
        .put(
          `api/admin/repositories/${this.repository.storage}/${this.repository.name}/config/frontend`,
          this.badgeSettings
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
