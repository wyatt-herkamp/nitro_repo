<template>
  <Suspense>
    <ViewRepo :storage="storage" :repositoryName="repository" />

    <template #fallback> Loading Repository </template>
  </Suspense>
</template>

<script lang="ts">
import { defineComponent } from "vue";

import { useRoute } from "vue-router";
import ViewRepo from "@/components/repo/ViewRepo.vue";

export default defineComponent({
  components: { ViewRepo },
  setup() {
    const route = useRoute();
    return {
      storage: route.params.storage as string,
      repository: route.params.repository as string,
    };
  },
});
</script>
