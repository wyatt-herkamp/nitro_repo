<template>
  <div class="flex flex-row">
    <SideBar back="repositories" />
    <UpdateRepo />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import {defineComponent} from "vue";

import SideBar from "@/components/admin/SideBar.vue";
import UpdateRepo from "@/components/repo/UpdateRepo.vue";

export default defineComponent({
  components: { SideBar, UpdateRepo },
});
</script>
