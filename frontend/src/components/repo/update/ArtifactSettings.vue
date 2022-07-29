<template>
  <MavenSettings
    v-if="repository.repository_type === 'Maven'"
    :repository="repository"
  />
  <NPMSettings
    v-else-if="repository.repository_type === 'NPM'"
    :repository="repository"
  />
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
  setup(props) {},
});
</script>
