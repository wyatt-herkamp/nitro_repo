<template>
  <div class="flex flex-col p-2">
    <div class="bg-slate-600 m-1"><PomCreator v-model="pom" /></div>
    <div class="flex flex-row bg-slate-800 m-1">
      <div>
        <drag-drop class="uploader" :uppy="uppy"></drag-drop>
      </div>
      <div class="m-3">
        <ul :key="files">
          <li v-for="file in uppy.getFiles()" v-bind:key="file.id">
            <input
              class="file"
              id="grid-Storage"
              type="text"
              :value="file.name"
              disabled
            />
            <input
              class="file"
              id="grid-Storage"
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
  @apply w-fit
}
</style>

<script lang="ts">
import { defineComponent, ref, watch } from "vue";
import { DragDrop } from "@uppy/vue";

import "@uppy/core/dist/style.css";
import "@uppy/drag-drop/dist/style.css";

import Uppy from "@uppy/core";
import { useCookie } from "vue-cookie-next";
import { Repository } from "nitro_repo-api-wrapper";
import PomCreator from "./PomCreator.vue";

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
      return new Uppy().on("file-added", (file) => {
        this.files = this.files + 1;
      });
    },
  },
  setup() {
    const pom = ref<Object | undefined>(undefined);
    //This exists to trigger a rerender on Vue.
    const files = ref<number>(0);
    const cookie = useCookie();
    watch(pom, () => {
      console.log("New Data");
    });
    return { cookie, pom, files };
  },
  methods: {},
  components: { PomCreator, DragDrop },
});
</script>
