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
import httpCommon from "@/http-common";

export default defineComponent({
  props: {
    userID: {
      required: true,
      type: Number,
    },
  },
  async setup(props) {
    const user = ref<User | undefined>();
    await httpCommon.apiClient
      .get<User>(`api/admin/user/${props.userID}`)
      .then((response) => {
        user.value = response.data;
      });

    return { user };
  },

  components: {
    UserGeneral,
  },
});
</script>
