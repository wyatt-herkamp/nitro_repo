<template>
  <MavenProjectCopy :project="project" />
</template>
<style scoped></style>
<script lang="ts">
import { Project } from "@nitro_repo/nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { defineComponent } from "vue";
import MavenProjectCopy from "./copy/MavenProjectCopy.vue";

export default defineComponent({
  props: {
    project: {
      required: true,
      type: Object as () => Project,
    },
  },
  setup() {
    const url = apiURL;
    return { url };
  },
  components: { MavenProjectCopy },
});
</script>
