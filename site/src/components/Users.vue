<template>

</template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import CreateUser from "@/components/CreateUser.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import {useCookie} from "vue-cookie-next";
import {getUsers} from "@/backend/api/User";
import {DEFAULT_USER_LIST} from "@/backend/Response";

export default defineComponent({
  components: { CreateUser, UpdateUser },

  setup() {
    let index = ref(0);
    const cookie = useCookie();
    const isLoading = ref(false);

    let error = ref("");
    let users = ref(DEFAULT_USER_LIST);
    const getUser = async () => {
      isLoading.value = true;
      try {
        const value = await getUsers(cookie.getCookie("token"));
        users.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value = "Error Loading";
      }
    };
    getUser();
    return {
      index,
      users,
      isLoading,
      error,
      getUser,
      cookie,
    };
  },
  methods: {
    updateList(id: number) {
      const getUser = async () => {
        try {
          const value = await getUsers(this.cookie.getCookie("token"));
          this.users = value;
          this.index = id;
        } catch (e) {
          this.error = "Error Loading";
        }
      };
      getUser();
    },
  },
});
</script>

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>
