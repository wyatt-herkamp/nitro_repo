<template>
  <div class="flex flex-row">
    <SideBar/>

    <Repositories v-if="page == 'repositories'"/>
    <Storages v-if="page == 'storages'"/>
    <Users v-if="page == 'users'"/>
  </div>
</template>

<script lang="ts">
import {defineComponent, onBeforeMount} from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import Me from "@/components/Me.vue";
import Repositories from "@/components/Repositories.vue";
import SideBar from "@/components/admin/SideBar.vue";
import Settings from "@/components/Settings.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import userStore from "@/store/user";
import {useRoute} from "vue-router";

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
    const route = useRoute();
    let page = route.params.page;

    onBeforeMount(userStore.getUser);
    return {
      userStore,
      page
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
