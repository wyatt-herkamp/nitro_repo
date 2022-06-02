<template>
  <div v-if="user != undefined">
    <SubNavBar>
      <LinkNavItem
        :href="'/admin/storage/' + storage"
        icon="arrow-back"
        name="Back"
      />

      <LinkNavItem
        v-if="user.permissions.admin || user.permissions.user_manager"
        href="/admin/users"
        icon="user"
        name="Users"
      />
      <LinkNavItem
        v-if="user.permissions.admin || user.permissions.repository_manager"
        href="/admin/storages"
        icon="box"
        name="Storages"
      />
    </SubNavBar>
    <UpdateRepo />
  </div>
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
