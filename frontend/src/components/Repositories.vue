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
import { defineComponent, inject, ref } from "vue";
import CreateRepo from "@/components/CreateRepo.vue";
import { getRepositoriesByStorage } from "@nitro_repo/nitro_repo-api-wrapper";

import { ListItem } from "./common/list/ListTypes";
import { useRouter } from "vue-router";

export default defineComponent({
  components: { CreateRepo },
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
  },
  setup(props) {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const list = ref<ListItem[]>([]);
    const openModel = ref(false);

    const getRepos = async () => {
      try {
        const value = await getRepositoriesByStorage(
          token as string,
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
      list,
      getRepos,
      openModel,
    };
  },
});
</script>
