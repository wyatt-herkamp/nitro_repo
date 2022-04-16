<template>
  <div v-if="me != undefined">
    <SubNavBar>
      <LinkNavItem :href="'/admin/users/'" icon="arrow-back" name="Back" />

      <LinkNavItem
        v-if="me.permissions.admin || me.permissions.user_manager"
        href="/admin/users"
        icon="user"
        name="Users"
      />
      <LinkNavItem
        v-if="me.permissions.admin || me.permissions.repository_manager"
        href="/admin/storages"
        icon="box"
        name="Storages"
      />
    </SubNavBar>
    <UpdateUser :userID="user" />
  </div>
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
