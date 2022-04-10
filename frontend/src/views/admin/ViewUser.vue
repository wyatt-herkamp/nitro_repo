<template>
   <div class="w-full md:flex md:flex-row">
    <SideBar back="users" />
    <UpdateUser :userID="user" />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { defineComponent } from "vue";

import SideBar from "@/components/SideBar.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import { useRoute } from "vue-router";

export default defineComponent({
  components: { SideBar, UpdateUser },
  setup() {
    const route = useRoute();
    const user = Number.parseInt(route.params.user as string);
    return {user};
  },
});
</script>
