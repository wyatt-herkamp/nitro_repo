<template>
  <CreateUser v-model="openModel" />
  <div class="w-full">
    <div class="flex p-4">
      <SearchableList v-model="list">
        <template v-slot:title> Users </template>
        <template v-slot:createButton>
          <button class="openModalButton" @click="openModel = true">
            Create User
          </button>
        </template>
      </SearchableList>
    </div>
  </div>
</template>


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
    let openModel = ref(false);

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
      openModel,
    };
  },
});
</script>

