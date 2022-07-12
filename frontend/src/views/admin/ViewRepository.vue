<template>
  <Suspense>
    <LoadThenRenderRepo :storageId="storage" :repositoryId="repo" />

    <template #fallback> Loading repository... </template>
  </Suspense>
</template>
<style scoped></style>
<script lang="ts">
import { computed, defineComponent } from "vue";

import { useRoute } from "vue-router";
import { useUserStore } from "@/store/user";
import LoadThenRenderRepo from "@/components/repo/update/LoadThenRenderRepo.vue";

export default defineComponent({
  components: { LoadThenRenderRepo },
  setup() {
    const route = useRoute();
    const userStore = useUserStore();
    const user = computed(() => {
      return userStore.$state.user;
    });

    const storage = route.params.storage as string;
    const repo = route.params.repo as string;

    return { storage, repo, user };
  },
});
</script>
