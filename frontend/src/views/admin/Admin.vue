<template>
  <div class="w-full">
    <SubNavBar v-model="page">
      <LinkNavItem href="/admin/users" icon="user" name="Users" />
      <LinkNavItem href="/admin/storages" icon="box" name="Storages" />
    </SubNavBar>
    <Storages class="mt-2 md:mt-0" v-if="page == 'storages'" />
    <Users class="mt-2 md:mt-0" v-if="page == 'users'" />
  </div>
</template>

<script lang="ts">
import { defineComponent, onBeforeMount, ref } from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import Me from "@/components/Me.vue";
import Repositories from "@/components/Repositories.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import { useRoute } from "vue-router";
import SubNavBar from "@/components/common/nav/SubNavBar.vue";
import LinkNavItem from "../../components/common/nav/LinkNavItem.vue";

export default defineComponent({
  components: {
    Storages,
    Repositories,
    Users,
    UpdateUser,
    Me,
    SubNavBar,
    LinkNavItem,
  },

  setup() {
    const route = useRoute();
    let page = ref(route.params.page as string);

    return {
      page,
    };
  },
});
</script>
