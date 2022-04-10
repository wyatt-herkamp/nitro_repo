<template>
  <div :class="openModel ? 'flex w-full' : 'w-full lg:w-3/4  xl:mx-auto'">
    <div
      class="md:p-4"
      :class="openModel ? 'hidden lg:block lg:grow ' : 'w-full'"
    >
      <SearchableList v-model="list">
        <template v-slot:title> Repositories </template>
        <template v-slot:createButton>
          <button class="openModalButton" @click="openModel = true">
            Create Repository
          </button>
        </template>
      </SearchableList>
    </div>
    <div v-if="openModel" class="mx-auto lg:w-1/4 lg:flex-row">
      <CreateRepo v-model="openModel" :storage="storage" />
    </div>
  </div>
</template>


<script lang="ts">
import { defineComponent, ref } from "vue";
import CreateRepo from "@/components/CreateRepo.vue";
import { useCookie } from "vue-cookie-next";
import { getRepositoriesByStorage } from "nitro_repo-api-wrapper";

import { ListItem } from "./common/list/ListTypes";

export default defineComponent({
  components: { CreateRepo },
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
  },
  setup(props) {
    const cookie = useCookie();
    const list = ref<ListItem[]>([]);
    let openModel = ref(false);

    const getRepos = async () => {
      try {
        const value = await getRepositoriesByStorage(
          cookie.getCookie("token"),
          props.storage.name
        );
        if (value == undefined) {
          return;
        }
        value.repositories.forEach((repository) => {
          list.value.push({
            name: repository.name,
            goTo:
              "/admin/repository/" + repository.storage + "/" + repository.name,
          });
        });
      } catch (e) {}
    };
    getRepos();
    return {
      cookie,
      list,
      getRepos,
      openModel,
    };
  },
});
</script>
