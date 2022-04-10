<template>
  <div>
    <SubNavBar>
      <LinkNavItem :href="'/admin/storages/'" icon="arrow-back" name="Back" />

      <LinkNavItem href="/admin/users" icon="user" name="Users" />
      <LinkNavItem href="/admin/storages" icon="box" name="Storages" />
    </SubNavBar>
    <UpdateStorage :storageId="storage" />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { defineComponent } from "vue";

import UpdateStorage from "@/components/UpdateStorage.vue";
import { useRoute } from "vue-router";

export default defineComponent({
  components: { UpdateStorage },
  setup() {
    const route = useRoute();
    const storage = route.params.storage as string;
    return { storage };
  },
});
</script>
