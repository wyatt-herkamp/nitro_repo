<template>
  <div :class="openModel ? 'flex w-full' : 'w-full lg:w-3/4  xl:mx-auto'">
    <div
      class="md:p-4"
      :class="openModel ? 'hidden lg:block lg:grow ' : 'w-full'"
    >
      <SearchableList v-model="list">
        <template v-slot:title> Repositories </template>
        <template v-slot:createButton>
          <button class="buttonOne" @click="openModel = true">
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

import { ListItem } from "./common/list/ListTypes";
import { useRouter } from "vue-router";
import httpCommon from "@/http-common";
import { Repository } from "@/types/repositoryTypes";

export default defineComponent({
  components: { CreateRepo },
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
  },
  async setup(props) {
    const list = ref<ListItem[]>([]);
    const openModel = ref(false);
    await httpCommon.apiClient
      .get<Array<Repository>>(`api/admin/repositories/${props.storage.name}`)
      .then((response) => {
        response.data.forEach((repo) => {
          list.value.push({
            name: repo.name,
            goTo: "/admin/repo/" + repo.name,
          });
        });
      });

    return {
      list,
      openModel,
    };
  },
});
</script>
