<template>
  <div class="flex flex-row">
    <SideBar />
    <Repositories />
  </div>
</template>

<script lang="ts">
import {defineComponent} from "vue";
import Repositories from "@/components/Repositories.vue";
import SideBar from "@/components/admin/SideBar.vue";

export default defineComponent({
  components: {
    Repositories,
    SideBar,
  },

  setup() {},
});
</script>
