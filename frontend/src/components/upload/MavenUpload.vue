<template>
  <div class="flex flex-row flex-wrap p-2">
    <div class="w-full max-w-6xl mx-auto rounded-md bg-slate-600 m-1">
      <PomCreator v-model="pom" />
    </div>
    <div
      class="
        flex flex-row flex-wrap
        w-full
        max-w-6xl
        mx-auto
        mt-2
        rounded-md
        bg-slate-800
      "
    >
      <div class="flex flex-col basis-full md:basis-1/2 lg:border-r-2 pb-5">
        <h1 class="text-left mx-5 mt-6 font-bold mb-0">Upload Files</h1>
        <drag-drop class="uploader" :uppy="uppy"></drag-drop>
      </div>
      <div class="flex flex-col basis-full md:basis-1/2">
        <div class="flex flex-row">
          <h1 class="text-left basis-1/2 mx-5 mt-6 font-bold mb-0">
            Files Ready for Upload
          </h1>
          <button
            class="basis-1/2 nitroButtonLight mr-2 hover:bg-slate-600"
            @click="upload()"
          >
            Upload
          </button>
        </div>

        <ul :key="files">
          <li
            class="flex flex-row p-2 m-1"
            v-for="file in uppy.getFiles()"
            v-bind:key="file.id"
          >
            <input class="file grow" type="text" :value="file.name" disabled />
            <input
              class="file w-1/6 text-center"
              type="text"
              :value="file.extension"
              disabled
            />
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>
<style>
.uploader {
  @apply mx-4;
  @apply mb-0;
  @apply mt-7;
}
.file {
  @apply appearance-none;
  @apply inline-block;
  @apply bg-gray-200;
  @apply text-gray-700;
  @apply border;
  @apply border-gray-200;
  @apply rounded;
  @apply mx-1;
}
</style>

<script lang="ts">
import { defineComponent, inject, ref, watch } from "vue";
import { DragDrop } from "@uppy/vue";

import "@uppy/core/dist/style.css";
import "@uppy/drag-drop/dist/style.css";

import Uppy from "@uppy/core";
import { Repository } from "@nitro_repo/nitro_repo-api-wrapper";
import PomCreator from "./PomCreator.vue";
import apiClient, { apiURL } from "@/http-common";
import { XMLBuilder, XMLParser } from "fast-xml-parser";
import { xmlOptions } from "./PomCreator";
import { useRouter } from "vue-router";

/**
 * How does the manual upload work?
 * Basically I let the backend do it's thing with one addition of accepting a bearer token instead of basic when doing put requests. This keeps the backend basically the same with not aditional changes
 * Then I accept files in the frontend and do put request simulating a query.
 */
export default defineComponent({
  props: {
    repo: {
      required: true,
      type: Object as () => Repository,
    },
  },
  computed: {
    uppy: function () {
      return new Uppy()
        .on("file-added", (file) => {
          this.files = this.files + 1;
        })
        .on("file-removed", (file) => {
          this.files = this.files - 1;
        });
    },
  },
  beforeUnmount() {
    this.uppy.close();
  },
  setup() {
    const token: string | undefined = inject("token");

    const pom = ref<Object | undefined>(undefined);
    //This exists to trigger a rerender on Vue.
    const files = ref<number>(0);
    watch(pom, () => {
      console.log("New Data");
    });
    return { pom, files, token: token as string };
  },
  methods: {
    async upload() {
      if (this.pom == undefined) {
        this.$notify({
          title: "POM.XML is missing!",
          type: "warn",
        });
        return;
      }
      const repo = this.$props.repo;
      const url = apiURL;
      const groupIdFormatted = this.pom.project.groupId.replace(".", "/");
      const path = `${groupIdFormatted}/${this.pom.project.artifactId}/${this.pom.project.version}`;
      const baseURL = `${url}/storages/${repo.storage}/${repo.name}/${path}`;
      console.log(groupIdFormatted);
      console.log(path);
      console.log(baseURL);
      this.uppy.getFiles().forEach(async (file) => {
        this.uploadFile(`${baseURL}/${file.name}`, file.data);
        this.uppy.removeFile(file.id, undefined);
      });
      const ser = new XMLBuilder(xmlOptions);
      const pom = ser.build(this.pom);
      const textEncoder = new TextEncoder();
      const encoding = textEncoder.encode(pom);
      await this.uploadFile(
        `${baseURL}/${this.pom.project.artifactId}-${this.pom.project.version}.pom`,
        encoding
      );
    },

    async uploadFile(url: string, file: any) {
      await apiClient
        .put(url, file, {
          headers: {
            Authorization: "Bearer " + this.token,
          },
        })
        .then(
          (result) => {
            this.$notify({
              title: `${url} was uploaded`,
              type: "info",
            });
          },
          (err) => {
            if (err.response) {
              this.$notify({
                title: `${url} failed to upload error ${err.response.status}`,
                type: "error",
              });
            } else if (err.request) {
              this.$notify({
                title: `${url} failed to upload error ${err.request}`,
                type: "error",
              });
            } else {
              this.$notify({
                title: `${url} failed to unknown error`,
                type: "error",
              });
            }
          }
        );
    },
  },
  components: { PomCreator, DragDrop },
});
</script>
