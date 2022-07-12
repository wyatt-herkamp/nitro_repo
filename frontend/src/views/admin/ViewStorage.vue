<template>
  <Suspense>
    <UpdateStorage :storageId="storage" />

    <template #fallback> Loading... </template>
  </Suspense>
</template>
<style scoped></style>
<script lang="ts">
import { computed, defineComponent } from "vue";

import UpdateStorage from "@/components/UpdateStorage.vue";
import { useRoute } from "vue-router";
import { useUserStore } from "@/store/user";

export default defineComponent({
  components: { UpdateStorage },
  setup() {
    const userStore = useUserStore();
    const user = computed(() => {
      return userStore.$state.user;
    });
    const route = useRoute();
    const storage = route.params.storage as string;
    return { storage, user };
  },
});
</script>
