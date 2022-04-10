<template>
  <div :class="createUser ? 'flex w-full' : 'w-full lg:w-3/4  xl:mx-auto'">
    <div
      class="md:p-4"
      :class="createUser ? 'hidden lg:block lg:grow ' : 'w-full'"
    >
      <SearchableList v-model="list">
        <template v-slot:title> Users </template>
        <template v-slot:createButton>
          <button class="openModalButton" @click="createUser = true">
            Create User
          </button>
        </template>
      </SearchableList>
    </div>
    <div
      v-if="createUser"
      :class="createUser ? 'flex   mx-auto' : 'lg:w-1/4 flex-row '"
    >
      <CreateUser v-model="createUser" />
    </div>
  </div>
</template>

<style scoped></style>
<script lang="ts">
import { defineComponent, ref, watch } from "vue";
import CreateUser from "@/components/CreateUser.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import { useCookie } from "vue-cookie-next";
import { getUsers } from "nitro_repo-api-wrapper";
import { ListItem } from "./common/list/ListTypes";

export default defineComponent({
  components: { CreateUser, UpdateUser },

  setup() {
    const cookie = useCookie();
    let createUser = ref(false);

    let list = ref<ListItem[]>([]);
    const getUser = async () => {
      try {
        const value = await getUsers(cookie.getCookie("token"));
        if (value == undefined) {
          return;
        }
        value.users.forEach((user) => {
          list.value.push({
            name: user.name,
            goTo: "/admin/user/" + user.id,
          });
        });
      } catch (e) {}
    };
    getUser();
    return {
      list,
      getUser,
      cookie,
      createUser,
    };
  },
});
</script>
