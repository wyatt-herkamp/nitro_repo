<template>
  <SideBar />
</template>

<script lang="ts">
import {defineComponent, onBeforeMount, ref} from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import Me from "@/components/Me.vue";
import Repositories from "@/components/Repositories.vue";
import SideBar from "@/components/admin/SideBar.vue";
import Settings from "@/components/Settings.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import userStore from "@/store/user";

export default defineComponent({
  components: {
    Storages,
    Repositories,
    Users,
    UpdateUser,
    Settings,
    Me,
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

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>
