<template>
  <div class="w-3/4 mx-auto">
    <h2 class="settingHeader">Generic Config Editor: {{ settingName }}</h2>

    <JsonEditorVue class="editor" :validator="validator" v-model="settings" />
    <button class="nitroButton float-right" @click="pushSettings()">
      Update Settings
    </button>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import httpCommon from "@/http-common";
import JsonEditorVue from "json-editor-vue";
import "vanilla-jsoneditor/themes/jse-theme-dark.css";
import { createAjvValidator, JSONData } from "vanilla-jsoneditor";

export default defineComponent({
  name: "UndefinedSettingConfig",
  components: {
    JsonEditorVue,
  },
  props: {
    repository: {
      type: Object as () => { storage: string; name: string },
      required: true,
    },
    settingName: {
      type: String,
      required: true,
    },
    schema: {
      type: Object as () => JSONData,
      required: true,
    },
  },
  setup(props) {
    const settings = ref<unknown | undefined>(undefined);
    const validator = createAjvValidator(props.schema);
    httpCommon.apiClient
      .get(
        `api/admin/repositories/${props.repository.storage}/${props.repository.name}/config/${props.settingName}`
      )
      .then((response) => {
        settings.value = response.data;
      });
    return { settings, validator };
  },
  methods: {
    async pushSettings() {
      await httpCommon.apiClient
        .put(
          `api/admin/repositories/${this.repository.storage}/${this.repository.name}/config/${this.settingName}`,
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

<style scoped>
.editor {
  @apply pl-2;
  @apply my-auto;
  font-family: "Fira Code", monospace;
  font-size: 14px;
  @apply text-white;
}
</style>
