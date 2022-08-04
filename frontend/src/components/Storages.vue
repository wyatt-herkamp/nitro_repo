<template>
  <div v-show="!create" class="w-full lg:w-3/4 xl:mx-auto">
    <div class="md:p-4 w-full">
      <SearchableList v-model="list">
        <template v-slot:title> Storages </template>
        <template v-slot:createButton>
          <button class="buttonOne" @click="create = true">
            Create Storage
          </button>
        </template>
      </SearchableList>
    </div>
  </div>
  <div v-show="create" class="w-full lg:w-3/4 xl:mx-auto">
    <div class="md:p-4 w-full">
      <CreateStorage :storagesThatExist="storages" />
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, inject, ref } from "vue";
import CreateStorage from "@/components/CreateStorage.vue";
import SearchableList from "./common/list/SearchableList.vue";
import { ListItem } from "./common/list/ListTypes";
import { useRoute, useRouter } from "vue-router";
import httpCommon from "@/http-common";
import { Storage } from "@/types/storageTypes";

export default defineComponent({
  components: { CreateStorage, SearchableList },

  setup() {
    const storages = ref<Storage[]>([]);
    const list = computed(() => {
      return storages.value.map((storage) => {
        return {
          name: storage.id,
          goTo: "/admin/storage/" + storage.id,
        };
      });
    });
    const router = useRoute();
    const create = ref(router.query.create === "true");

    const getStorage = async () => {
      await httpCommon.apiClient
        .get<Array<Storage>>("api/admin/storages")
        .then((response) => {
          storages.value = response.data;
        });
    };
    getStorage();
    return {
      getStorage,
      list,
      create,
      storages,
    };
  },
});
</script>
