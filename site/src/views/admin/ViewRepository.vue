<template>
  <div class="flex flex-row">
    <SideBar back="repositories" />
    <ViewRepo />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import {defineComponent} from "vue";

import SideBar from "@/components/admin/SideBar.vue";
import ViewRepo from "@/components/repo/ViewRepo.vue";

export default defineComponent({
  components: { SideBar, ViewRepo },
});
</script>
