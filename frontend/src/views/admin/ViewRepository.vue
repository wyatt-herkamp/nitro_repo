<template>
  <UpdateRepo />
</template>
<style scoped></style>
<script lang="ts">
import { computed, defineComponent } from "vue";

import UpdateRepo from "@/components/repo/update/UpdateRepo.vue";
import { useRoute } from "vue-router";
import { useUserStore } from "@/store/user";

export default defineComponent({
  components: { UpdateRepo },
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
