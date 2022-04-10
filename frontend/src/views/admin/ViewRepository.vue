<template>
  <div>
    <SubNavBar>
      <LinkNavItem
        :href="'/admin/storage/' + storage"
        icon="arrow-back"
        name="Back"
      />

      <LinkNavItem href="/admin/users" icon="user" name="Users" />
      <LinkNavItem href="/admin/storages" icon="box" name="Storages" />
    </SubNavBar>
    <UpdateRepo />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { defineComponent } from "vue";

import UpdateRepo from "@/components/repo/update/UpdateRepo.vue";
import { useRoute } from "vue-router";

export default defineComponent({
  components: { UpdateRepo },
  setup() {
    const route = useRoute();

    const storage = route.params.storage as string;
    const repo = route.params.repo as string;
    return { storage, repo };
  },
});
</script>
