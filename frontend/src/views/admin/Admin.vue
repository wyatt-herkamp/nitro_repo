<template>
  <div v-if="user != undefined" class="w-full">
    <SubNavBar v-model="page">
      <LinkNavItem
        v-if="user.permissions.admin || user.permissions.user_manager"
        href="/admin/users"
        icon="user"
        name="Users"
      />
      <LinkNavItem
        v-if="user.permissions.admin || user.permissions.repository_manager"
        href="/admin/storages"
        icon="box"
        name="Storages"
      />
    </SubNavBar>
    <Storages class="mt-2 md:mt-0" v-if="page == 'storages'" />
    <Users class="mt-2 md:mt-0" v-if="page == 'users'" />
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import { useRoute } from "vue-router";
import SubNavBar from "@/components/common/nav/SubNavBar.vue";
import LinkNavItem from "../../components/common/nav/LinkNavItem.vue";
import { useUserStore } from "@/store/user";

export default defineComponent({
  components: {
    Storages,
    Users,
    SubNavBar,
    LinkNavItem,
  },

  setup() {
    const route = useRoute();
    let page = ref(route.params.page as string);
    const userStore = useUserStore();
    const user = computed(() => {
      return userStore.$state.user;
    });
    return {
      user,
      page,
    };
  },
});
</script>
