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
import { getUsers } from "@nitro_repo/nitro_repo-api-wrapper";
import { ListItem } from "./common/list/ListTypes";
import { useRouter } from "vue-router";

export default defineComponent({
  components: { CreateUser },

  setup() {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const createUser = ref(false);

    const list = ref<ListItem[]>([]);
    const getUser = async () => {
      try {
        const value = await getUsers(token as string);
        if (value == undefined) {
          return;
        }
        value.users.forEach((user) => {
          list.value.push({
            name: user.name,
            goTo: "/admin/User/" + user.id,
          });
        });
      } catch (e) {}
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
