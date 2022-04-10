<template>
   <div>
         <SubNavBar>
      <LinkNavItem
        :href="'/admin/users/'"
        icon="arrow-back"
        name="Back"
      />

      <LinkNavItem href="/admin/users" icon="user" name="Users" />
      <LinkNavItem href="/admin/storages" icon="box" name="Storages" />
    </SubNavBar>
    <UpdateUser :userID="user" />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { defineComponent } from "vue";

import UpdateUser from "@/components/UpdateUser.vue";
import { useRoute } from "vue-router";

export default defineComponent({
  components: {  UpdateUser },
  setup() {
    const route = useRoute();
    const user = Number.parseInt(route.params.user as string);
    return {user};
  },
});
</script>
