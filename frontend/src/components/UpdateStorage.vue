<template>
  <div v-if="storage != undefined" class="min-w-full md:min-w-0 md:w-3/4">
    <SubNavBar v-model="storageTab">
      <SubNavItem index="General"> General </SubNavItem>
      <SubNavItem index="Repositories"> Repositories </SubNavItem>
    </SubNavBar>
    <Repositories :storage="storage" v-if="storageTab == 'Repositories'" />
    <div v-else-if="storageTab == 'General'" class="settingContent">
      <div class="settingBox">
        <label  class="nitroLabel"  for="grid-name"> name </label>
        <input
          class="nitroTextInput"
          id="grid-name"
          type="text"
          v-model="storage.name"
          disabled
        />
      </div>
      <div class="settingBox">
        <label  class="nitroLabel"  for="grid-public-name"> Public Name </label>
        <input
          class="nitroTextInput"
          id="grid-public-name"
          type="text"
          v-model="storage.public_name"
          disabled
        />
      </div>
      <div class="settingBox">
        <label  class="nitroLabel"  for="grid-created"> Date Created </label>
        <input
          class="nitroTextInput"
          id="grid-created"
          type="text"
          v-model="date"
          disabled
        />
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { getStorage } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRoute, useRouter } from "vue-router";
import { Storage } from "@nitro_repo/nitro_repo-api-wrapper";
import Repositories from "./Repositories.vue";

export default defineComponent({
  props: {
    storageId: {
      type: String,
      required: true,
    },
  },
  setup(props) {
    let storage = ref<Storage | undefined>(undefined);
    let date = ref<string | undefined>(undefined);

    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    const storageTab = ref("General");
    const getStorageInternal = async () => {
      try {
        const value = (await getStorage(
          undefined,
          props.storageId
        )) as Storage;
        storage.value = value;
        date.value = new Date(storage.value.created).toLocaleDateString(
          "en-US"
        );
        meta.title = value.name;
      } catch (e) {
        console.log(e);
      }
    };
    getStorageInternal();
    return {
      date,
      storage,
      storageTab,
    };
  },
  methods: {
    async deleteStorage() {
      console.log("TODO");
    },
  },
  components: { Repositories },
});
</script>
