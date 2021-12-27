<template>
  <div class="flex flex-row h-96">
    <div class="flex flex-col w-56 bg-slate-800 rounded-r-3xl overflow-hidden">
      <ul class="flex flex-col py-4">
        <Item href="/admin/users" icon="user" name="Users"/>
        <Item href="/admin/repositories" icon="package" name="Repositories"/>
        <Item href="/admin/storages" icon="box" name="Storages"/>
        <Item href="/admin/settings" icon="dots-horizontal-rounded" name="Settings"/>
      </ul>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, onBeforeMount, ref} from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import Me from "@/components/Me.vue";
import Repositories from "@/components/Repositories.vue";
import Item from "@/components/admin/Item.vue";
import Settings from "@/components/Settings.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import userStore from "@/store/user";

export default defineComponent({
  components: {Storages, Repositories, Users, UpdateUser, Settings, Me, Item},

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