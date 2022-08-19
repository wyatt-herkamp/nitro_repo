<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Rules</h2>
    <div id="editor"></div>

    <div class="settingSection">
      <div class="settingBox">
        <label for="grid-style" class="nitroLabel">Repository Page</label>
        <select v-model="settings.settings.page_type" class="nitroTextInput">
          <option selected value="None">None</option>
          <option value="Markdown">Markdown</option>
        </select>
      </div>
      <div class="settingBox">
        <button class="nitroButton" @click="submitBadge()">
          Update Repository Settings
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from "vue";
import httpCommon from "@/http-common";
import { defaultValueCtx, Editor, rootCtx } from "@milkdown/core";
import { nord } from "@milkdown/theme-nord";
import { commonmark } from "@milkdown/preset-commonmark";
import { menu } from "@milkdown/plugin-menu";
import { listener, listenerCtx } from "@milkdown/plugin-listener";
export default defineComponent({
  name: "RepositoryPage",
  props: {
    repository: {
      type: Object as () => { storage: string; name: string },
      required: true,
    },
  },
  setup(props) {
    const settings = ref({ settings: { page_type: "None" }, page: "" });

    httpCommon.apiClient
      .get(
        `api/admin/repositories/${props.repository.storage}/${props.repository.name}/config/repository_page`
      )
      .then((response) => {
        settings.value = response.data;
        console.log(settings.value);

        Editor.make()
          .use(nord)
          .use(commonmark)
          .use(menu)
          .use(listener)
          .config((ctx) => {
            ctx.set(rootCtx, document.querySelector("#editor"));
            ctx.set(defaultValueCtx, response.data.page);
            ctx
              .get(listenerCtx)
              .markdownUpdated((ctx, markdown, prevMarkdown) => {
                console.log(markdown);
                settings.value.page = markdown;
              });
          })
          .create();
      });
    return { settings };
  },
  methods: {
    async submitBadge() {
      console.log(this.settings);
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
