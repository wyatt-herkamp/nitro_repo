<template>
  <div class="flex flex-row">
    <SideBar />
    <Storages />
  </div>
</template>

<script lang="ts">
import { defineComponent, onBeforeMount, ref } from "vue";
import Storages from "@/components/Storages.vue";
import SideBar from "@/components/admin/SideBar.vue";

import userStore from "@/store/user";

export default defineComponent({
  components: {
    Storages,
    SideBar,
  },

  setup() {
    let index = ref(4);
    onBeforeMount(userStore.getUser);
    return {
      index,
      userStore,
    };
  },
});
</script>
