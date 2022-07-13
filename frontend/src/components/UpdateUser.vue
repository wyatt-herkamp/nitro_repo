<template>
  <div class="min-h-screen w-full flex flex-wrap lg:flex-nowrap">
    <div class="flex flex-col w-full">
      <UserGeneral :user="user" />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, inject, ref } from "vue";

import UserGeneral from "./user/update/UserGeneral.vue";
import { useRouter } from "vue-router";
import { User } from "@/types/user";

export default defineComponent({
  props: {
    userID: {
      required: true,
      type: Number,
    },
  },
  async setup(props) {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      await useRouter().push("login");
    }
    const user = ref<User | undefined>();
    //TODO: get user from API

    return { user };
  },

  components: {
    UserGeneral,
  },
});
</script>
