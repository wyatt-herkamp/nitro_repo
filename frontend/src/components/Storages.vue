<template>
  <div :class="openModel ? 'flex w-full' : 'w-full lg:w-3/4  xl:mx-auto'">
    <div
      class="md:p-4"
      :class="openModel ? 'hidden lg:block lg:grow ' : 'w-full'"
    >
      <SearchableList v-model="list">
        <template v-slot:title> Storages </template>
        <template v-slot:createButton>
          <button class="buttonOne" @click="openModel = true">
            Create Storage
          </button>
        </template>
      </SearchableList>
    </div>
    <div v-if="openModel" class="mx-auto lg:w-1/4 lg:flex-row">
      <CreateStorage v-model="openModel" />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, inject, ref } from "vue";
import CreateStorage from "@/components/CreateStorage.vue";
import SearchableList from "./common/list/SearchableList.vue";
import { ListItem } from "./common/list/ListTypes";
import { useRouter } from "vue-router";
import httpCommon from "@/http-common";
import { Storage } from "@/types/storageTypes";

export default defineComponent({
  components: { CreateStorage, SearchableList },

  setup() {
    const list = ref<ListItem[]>([]);
    const openModel = ref(false);

    const getStorage = async () => {
      await httpCommon.apiClient
        .get<Array<Storage>>("api/admin/storages")
        .then((response) => {
          response.data.forEach((storage) => {
            list.value.push({
              name: storage.id,
              goTo: "/admin/storage/" + storage.id,
            });
          });
        });
    };
    getStorage();
    return {
      getStorage,
      list,
      openModel,
    };
  },
});
</script>
