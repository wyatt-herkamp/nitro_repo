<template>
  <MavenProjectCopy :child="child" :project="project" />
</template>
<style scoped></style>
<script lang="ts">
import { Project } from "nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { defineComponent, ref } from "vue";
import MavenProjectCopy from "./copy/MavenProjectCopy.vue";
export default defineComponent({
  props: {
    project: {
      required: true,
      type: Object as () => Project,
    },
    child: {
      default: false,
      type: Boolean,
    },
  },
  setup() {
    let url = apiURL;
    return { url };
  },
  components: { MavenProjectCopy },
});
</script>
