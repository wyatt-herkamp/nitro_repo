<template>
  <div v-show="!createUser" class="lg:w-1/2 mx-auto">
    <div class="md:p-4">
      <SearchableList v-model="list">
        <template v-slot:title> Users </template>
        <template v-slot:createButton>
          <button class="buttonOne" @click="createUser = true">
            Create User
          </button>
        </template>
      </SearchableList>
    </div>
  </div>
  <div v-if="createUser" class="w-fit mx-auto">
    <CreateUser @close="close" v-model="createUser" />
  </div>
</template>
/
<style scoped></style>
<script lang="ts">
import { defineComponent, inject, ref } from "vue";
import CreateUser from "@/components/CreateUser.vue";
import { ListItem } from "./common/list/ListTypes";
import { useRouter } from "vue-router";
import httpCommon from "@/http-common";
import { User } from "@/types/userTypes";
import SearchableList from "@/components/common/list/SearchableList.vue";

export default defineComponent({
  components: { SearchableList, CreateUser },

  setup() {
    const createUser = ref(false);

    const list = ref<ListItem[]>([]);
    const getUser = async () => {
      await httpCommon.apiClient
        .get<Array<User>>("api/admin/users/list")
        .then((response) => {
          response.data.forEach((user) => {
            list.value.push({
              name: `${user.username}`,
              goTo: "/admin/user/" + user.id,
            });
          });
        });
    };
    const close = async () => {
      list.value = [];
      await getUser();
      createUser.value = false;
    };
    getUser();
    return {
      list,
      close,
      getUser,
      createUser,
    };
  },
});
</script>
