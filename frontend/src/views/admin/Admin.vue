<template>
  <div class="w-full md:flex md:flex-row">

    <Storages class="mt-2 md:mt-0"  v-if="page == 'storages'"/>
    <Users class="mt-2 md:mt-0" v-if="page == 'users'"/>
  </div>
</template>

<script lang="ts">
import {defineComponent, onBeforeMount} from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import Me from "@/components/Me.vue";
import Repositories from "@/components/Repositories.vue";
import SideBar from "@/components/SideBar.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import userStore from "@/store/user";
import {useRoute} from "vue-router";

export default defineComponent({
  components: {
    Storages,
    Repositories,
    Users,
    UpdateUser,
    Me,
    SideBar,
  },

  setup() {
    const route = useRoute();
    let page = route.params.page as string;

    onBeforeMount(userStore.getUser);
    return {
      userStore,
      page
    };
  },
});
</script>
