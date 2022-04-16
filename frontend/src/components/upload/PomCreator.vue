<template>
  <div class="" v-if="pom == undefined">
    <SubNavBar v-model="activePage">
      <SubNavItem index="UploadPom"> Upload Pom </SubNavItem>
      <SubNavItem index="CreatePom"> Create Pom </SubNavItem>
    </SubNavBar>
    <div class="mx-2 mb-3" v-if="activePage == 'UploadPom'">
      <drag-drop :uppy="uppy"></drag-drop>
    </div>
    <div class="mx-2 mb-3" v-if="activePage == 'CreatePom'">
      <form
        autocomplete="off"
        class="settingContent flex flex-row"
        @submit.prevent="handleChange(creatingPom)"
      >
        <div class="settingBox">
          <label class="nitroLabel"> groupId </label>
          <input
            class="nitroTextInput"
            type="text"
            v-model="creatingPom.project.groupId"
          />
          <label class="nitroLabel"> artifactId </label>
          <input
            class="nitroTextInput"
            type="text"
            v-model="creatingPom.project.artifactId"
          />
        </div>
        <div class="settingBox flex flex-col">
          <div class="">
            <label class="nitroLabel"> version </label>
            <input
              class="nitroTextInput"
              type="text"
              v-model="creatingPom.project.version"
            />
          </div>
          <button class="md:!mt-10 nitroButton">Create Pom</button>
        </div>
      </form>
    </div>
  </div>
  <div v-else class="md:h-1/4">
    <div class="flex flex-wrap mb-6 justify-center">
      <div class="settingBox">
        <label class="nitroLabel" for="grid-name"> groupId </label>
        <input
          class="disabled nitroTextInput"
          id="grid-Storage"
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
import { defineComponent, onBeforeUnmount, ref } from "vue";
import { DragDrop } from "@uppy/vue";

import "@uppy/core/dist/style.css";
import "@uppy/dashboard/dist/style.css";

import Uppy from "@uppy/core";
import SubNavBar from "../common/nav/SubNavBar.vue";
import SubNavItem from "../common/nav/SubNavItem.vue";
import { XMLParser } from "fast-xml-parser";
import { Pom, xmlOptions } from "./PomCreator";

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
  beforeUnmount() {
    this.uppy.close();
  },
  setup(props, { emit }) {
    const pom = ref<Object | undefined>(undefined);
    const creatingPom = ref<Pom>({
      project: {
        modelVersion: "",
        groupId: "",
        artifactId: "",
        version: "",
      },
    });
    const activePage = ref<string>("UploadPom");

    const handleChange = (data: Object): void => {
      pom.value = data;
      emit("update:modelValue", data);
    };

    return { pom, activePage, handleChange, creatingPom };
  },
  methods: {
    handleAdd: async function (file: any) {
      const parser = new XMLParser(xmlOptions);
      let text = await file.data.text();
      let data = parser.parse(text);
      this.handleChange(data);
      this.uppy.close();
    },
  },
  components: { SubNavBar, SubNavItem, DragDrop },
});
</script>
