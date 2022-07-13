<template>
  <MavenSettings v-if="repositoryType === 'Maven'" :repository="repository" />
  <NPMSettings v-else-if="repositoryType === 'NPM'" :repository="repository" />
</template>
<script lang="ts">
import { defineComponent } from "vue";
import MavenSettings from "@/components/repo/types/maven/MavenSettings.vue";
import NPMSettings from "@/components/repo/types/npm/NPMSettings.vue";
import { Repository } from "@/types/repositoryTypes";

export default defineComponent({
  name: "ArtifactSettings",
  components: { NPMSettings, MavenSettings },
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  setup(props) {
    const repositoryType = Object.keys(props.repository.repo_type)[0];
    return {
      repositoryType,
    };
  },
});
</script>
