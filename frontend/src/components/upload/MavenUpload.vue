<template>
  <div>
    <div class="flex-row"><PomCreator v-model="pom" /></div>
  </div>
</template>
<style>
</style>

<script lang="ts">
import { defineComponent, ref, watch } from "vue";
import { DragDrop } from "@uppy/vue";

import "@uppy/core/dist/style.css";
import "@uppy/dashboard/dist/style.css";

import Uppy from "@uppy/core";
import { useCookie } from "vue-cookie-next";
import { Repository } from "nitro_repo-api-wrapper";
import FileUpload from "./../../src/FileUpload.vue";
import http from "@/http-common";
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
    uppy: () => new Uppy().use(DragDrop),
  },
  setup() {
    const pom = ref<Object | undefined>(undefined);
    const cookie = useCookie();
    watch(pom, () => {
      console.log("New Data");
    });
    return { cookie, pom };
  },
  methods: {},
  components: { PomCreator },
});
</script>
