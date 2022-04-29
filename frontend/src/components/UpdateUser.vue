<template>
  <div
    v-if="user != undefined"
    class="min-h-screen w-full flex flex-wrap lg:flex-nowrap"
  >
    <div class="flex flex-col w-full">
      <UserGeneral :user="user" />
    </div>
  </div>
</template>

<script lang="ts">
import { User } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref } from "vue";
import { getUserByID } from "@nitro_repo/nitro_repo-api-wrapper";

import UserGeneral from "./user/update/UserGeneral.vue";
import { useRouter } from "vue-router";

export default defineComponent({
  props: {
    userID: {
      required: true,
      type: Number,
    },
  },
  setup(props) {

    const user = ref<User | undefined>();
    const loadUser = async () => {
      try {
        let value = (await getUserByID(undefined, props.userID)) as User;
        user.value = value as User;
      } catch (e) {}
    };
    loadUser();
    return { user };
  },

  components: {
    UserGeneral,
  },
});
</script>
