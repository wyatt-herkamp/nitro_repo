<template>
  <div class="w-full md:flex md:flex-row">
    <SideBar back="repositories" />
    <UpdateRepo />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import {defineComponent} from "vue";

import SideBar from "@/components/SideBar.vue";
import UpdateRepo from "@/components/repo/update/UpdateRepo.vue";

export default defineComponent({
  components: { SideBar, UpdateRepo },
});
</script>
