<template>
  <div v-if="user != undefined">
    <SubNavBar>
      <LinkNavItem :href="'/admin/storages/'" icon="arrow-back" name="Back" />

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
    <UpdateStorage :storageId="storage" />
  </div>
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
