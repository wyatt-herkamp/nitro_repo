<template>
  <UpdateUser :userID="user" />
  <Suspense>
    <UpdateUser :userID="user" />

    <template #fallback> Loading user... </template>
  </Suspense>
</template>
<style scoped></style>
<script lang="ts">
import { computed, defineComponent } from "vue";

import UpdateUser from "@/components/UpdateUser.vue";
import { useRoute } from "vue-router";
import { useUserStore } from "@/store/user";

export default defineComponent({
  components: { UpdateUser },
  setup() {
    const userStore = useUserStore();
    const me = computed(() => {
      return userStore.$state.user;
    });
    const route = useRoute();
    const user = Number.parseInt(route.params.user as string);
    return { user, me };
  },
});
</script>
