<template>
  <Suspense>
    <ViewProject
      :storage="storage"
      :repositoryName="repository"
      :projectName="project"
      version="version"
    />

    <template #fallback> Loading Repository </template>
  </Suspense>
</template>

<script lang="ts">
import { defineComponent } from "vue";

import { useRoute } from "vue-router";
import ViewProject from "@/components/project/ViewProject.vue";

export default defineComponent({
  components: { ViewProject },
  setup() {
    const route = useRoute();
    return {
      storage: route.params.storage as string,
      repository: route.params.repository as string,
      project: route.params.project as string,
      version: route.params.version as string | undefined,
    };
  },
});
</script>
