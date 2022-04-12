<template>
  <div
    v-if="user != undefined"
    class="min-h-screen w-full flex flex-wrap lg:flex-nowrap"
  >
    <div class="flex flex-col w-full">
      <SubNavBar v-model="view">
        <SubNavItem index="General"> General </SubNavItem>
        <SubNavItem index="Password"> Password </SubNavItem>
      </SubNavBar>
      <UserGeneral :user="user" v-if="view == 'General'" />
      <UserPassword :user="user" v-if="view == 'Password'" />
    </div>
  </div>
</template>

<script lang="ts">
import { User } from "nitro_repo-api-wrapper";
import { defineComponent, inject, ref } from "vue";
import { getUserByID } from "nitro_repo-api-wrapper";

import UserGeneral from "./user/update/UserGeneral.vue";
import UserPassword from "./user/update/UserPassword.vue";
import SubNavBar from "./common/nav/SubNavBar.vue";
import SubNavItem from "./common/nav/SubNavItem.vue";
import { useRouter } from "vue-router";

export default defineComponent({
  props: {
    userID: {
      required: true,
      type: Number,
    },
  },
  setup(props) {
    let view = ref("General");

    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const user = ref<User | undefined>();
    const loadUser = async () => {
      try {
        let value = (await getUserByID(token as string, props.userID)) as User;
        user.value = value as User;
      } catch (e) {}
    };
    loadUser();
    return { user, view };
  },

  components: {
    UserGeneral,
    UserPassword,
    SubNavBar,
    SubNavItem,
  },
});
</script>
