<template>
  <div :class="openModel ? 'flex w-full' : 'w-full lg:w-3/4  xl:mx-auto'">
    <div
      class="md:p-4"
      :class="openModel ? 'hidden lg:block lg:grow ' : 'w-full'"
    >
      <SearchableList v-model="list">
        <template v-slot:title> Storages </template>
        <template v-slot:createButton>
          <button class="openModalButton" @click="openModel = true">
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
import { getStorages } from "@nitro_repo/nitro_repo-api-wrapper";
import SearchableList from "./common/list/SearchableList.vue";
import { ListItem } from "./common/list/ListTypes";
import { useRouter } from "vue-router";

export default defineComponent({
  components: { CreateStorage, SearchableList },

  setup() {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const list = ref<ListItem[]>([]);
    const openModel = ref(false);

    const getStorage = async () => {
      try {
        const value = await getStorages(token as string);
        if (value == undefined) {
          return;
        }
        value.forEach((storage) => {
          list.value.push({
            name: storage.public_name,
            goTo: "/admin/storage/" + storage.name,
          });
        });
      } catch (e) {
        console.error(e);
      }
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
