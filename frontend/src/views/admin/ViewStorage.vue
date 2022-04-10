<template>
   <div class="w-full md:flex md:flex-row">

    <SideBar back="storages" />
    <UpdateStorage :storageId="storage" />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { defineComponent } from "vue";

import SideBar from "@/components/SideBar.vue";
import UpdateStorage from "@/components/UpdateStorage.vue";
import { useRoute } from "vue-router";

export default defineComponent({
  components: { SideBar, UpdateStorage },
  setup() {
    const route = useRoute();
    const storage = route.params.storage as string;
    return {storage};
  },
});
</script>
