<template>
  <div :class="createUser ? 'flex w-full' : 'w-full lg:w-3/4  xl:mx-auto'">
    <div
      class="md:p-4"
      :class="createUser ? 'hidden lg:block lg:grow ' : 'w-full'"
    >
      <SearchableList v-model="list">
        <template v-slot:title> Users </template>
        <template v-slot:createButton>
          <button class="buttonOne" @click="createUser = true">
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
import { defineComponent, inject, ref } from "vue";
import CreateUser from "@/components/CreateUser.vue";
import { ListItem } from "./common/list/ListTypes";
import { useRouter } from "vue-router";
import httpCommon from "@/http-common";
import { User } from "@/types/userTypes";

export default defineComponent({
  components: { CreateUser },

  setup() {
    const createUser = ref(false);

    const list = ref<ListItem[]>([]);
    const getUser = async () => {
      await httpCommon.apiClient
        .get<Array<User>>("api/admin/users/list")
        .then((response) => {
          response.data.forEach((user) => {
            list.value.push({
              name: user.name,
              goTo: "/admin/user/" + user.name,
            });
          });
        });
    };
    getUser();
    return {
      list,
      getUser,
      createUser,
    };
  },
});
</script>
