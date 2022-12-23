<template>
  <div v-show="!create">
    <div class="md:p-4">
      <SearchableList v-model="list">
        <template v-slot:title> Repositories </template>
        <template v-slot:createButton>
          <button class="buttonOne" @click="create = true">
            Create Repository
          </button>
        </template>
      </SearchableList>
    </div>
  </div>
  <div v-show="create" class="w-1/2 mx-auto">
    <button class="buttonOne mt-2" @click="create = false">
      View Repositories
    </button>
    <CreateRepo :storage="storage" />
  </div>
</template>

<script lang="ts">
import { defineComponent, inject, ref } from "vue";
import CreateRepo from "@/components/CreateRepo.vue";

import { ListItem } from "./common/list/ListTypes";
import { useRoute, useRouter } from "vue-router";
import httpCommon from "@/http-common";
import { Repository } from "@/types/repositoryTypes";
import { Storage } from "@/types/storageTypes";
import SearchableList from "@/components/common/list/SearchableList.vue";
export default defineComponent({
  components: {SearchableList, CreateRepo },
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
  },
  async setup(props) {
    const list = ref<ListItem[]>([]);
    const create = ref(useRoute().query.create === "true");

    await httpCommon.apiClient
      .get<Array<Repository>>(`api/admin/repositories/${props.storage.id}`)
      .then((response) => {
        response.data.forEach((repo) => {
          list.value.push({
            name: repo.name,
            goTo: `/admin/repository/${repo.storage}/${repo.name}`,
          });
        });
      });

    return {
      list,
      create,
    };
  },
});
</script>
