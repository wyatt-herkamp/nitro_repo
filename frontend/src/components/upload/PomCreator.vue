<template>
  <div class="bg-slate-800" v-if="pom == undefined">
    <SubNavBar class="bg-slate-900" v-model="activePage">
      <SubNavItem index="UploadPom"> Upload Pom </SubNavItem>
      <SubNavItem index="CreatePom"> Create Pom </SubNavItem>
    </SubNavBar>
    <div class="mx-auto" v-if="activePage == 'UploadPom'">
      <drag-drop :uppy="uppy"></drag-drop>
    </div>
    <div v-if="activePage == 'CreatePom'">
      <h1>Coming Soon</h1>
    </div>
  </div>
  <div v-else class="h-1/4">
    <div class="flex flex-wrap mb-6 justify-center">
      <div class="settingBox">
        <label class="nitroLabel" for="grid-name"> groupId </label>
        <input
          class="disabled nitroTextInput"
          id="grid-name"
          type="text"
          v-model="pom.project.groupId"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-Storage"> aritfactId </label>
        <input
          class="disabled nitroTextInput"
          id="grid-Storage"
          type="text"
          v-model="pom.project.artifactId"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-created"> Version </label>
        <input
          class="disabled nitroTextInput"
          id="grid-created"
          type="text"
          v-model="pom.project.version"
          disabled
        />
      </div>
    </div>
  </div>
</template>
<style>
</style>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { DragDrop } from "@uppy/vue";

import "@uppy/core/dist/style.css";
import "@uppy/dashboard/dist/style.css";

import Uppy from "@uppy/core";
import SubNavBar from "../common/nav/SubNavBar.vue";
import SubNavItem from "../common/nav/SubNavItem.vue";
import { XMLParser } from "fast-xml-parser";

export default defineComponent({
  props: {
    modelValue: Object,
  },
  computed: {
    uppy: function () {
      return new Uppy({
        autoProceed: true,
        restrictions: {
          allowedFileTypes: ["text/*"],
          maxNumberOfFiles: 1,
          minNumberOfFiles: 1,
        },
      }).on("file-added", (file) => {
        this.handleAdd(file);
      });
    },
  },
  setup(props, { emit }) {
    const pom = ref<Object | undefined>(undefined);
    const activePage = ref<string>("UploadPom");

    const handleChange = (data: Object): void => {
      pom.value = data;
      emit("update:modelValue", data);
    };
    return { pom, activePage, handleChange };
  },
  methods: {
    handleAdd: async function (file: any) {
      const parser = new XMLParser();
      let text = await file.data.text();
      let data = parser.parse(text);
      this.handleChange(data);
      this.uppy.close();
    },
  },
  components: { SubNavBar, SubNavItem, DragDrop },
});
</script>
