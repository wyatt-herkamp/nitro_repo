<template>
  <MavenRepoInfo
    :repository="{
      name: project.repo_summary.name,
      storage: project.repo_summary.storage,
    }"
  />
  <MavenProjectCopy :project="project" />
</template>
<style scoped></style>
<script lang="ts">
import { apiURL } from "@/http-common";
import { defineComponent } from "vue";
import MavenProjectCopy from "./copy/MavenProjectCopy.vue";
import { Project } from "@/types/repositoryTypes";
import MavenRepoInfo from "@/components/repo/types/maven/MavenRepoInfo.vue";

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
  components: { MavenRepoInfo, MavenProjectCopy },
});
</script>
