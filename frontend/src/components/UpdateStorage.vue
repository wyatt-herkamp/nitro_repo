<template>
  <div>
    <SimpleTabs>
      <SimpleTab name="General">
        <div class="settingContent">
          <div class="settingBox">
            <label class="nitroLabel" for="grid-name"> name </label>
            <input
              class="nitroTextInput"
              id="grid-name"
              type="text"
              v-model="storage.name"
              disabled
            />
          </div>
          <div class="settingBox">
            <label class="nitroLabel" for="grid-public-name">
              Public Name
            </label>
            <input
              class="nitroTextInput"
              id="grid-public-name"
              type="text"
              v-model="storage.public_name"
              disabled
            />
          </div>
          <div class="settingBox">
            <label class="nitroLabel" for="grid-created"> Date Created </label>
            <input
              class="nitroTextInput"
              id="grid-created"
              type="text"
              v-model="date"
              disabled
            />
          </div>
        </div>
      </SimpleTab>
      <SimpleTab name="Repositories">
        <Repositories :storage="storage" />
      </SimpleTab>
    </SimpleTabs>
  </div>
</template>
<script lang="ts">
import { defineComponent, inject, ref } from "vue";
import { useMeta } from "vue-meta";
import Repositories from "./Repositories.vue";
import httpCommon from "@/http-common";
import { Storage } from "@/types/storageTypes";

export default defineComponent({
  props: {
    storageId: {
      type: String,
      required: true,
    },
  },
  async setup(props) {
    const storage = ref<Storage | undefined>(undefined);
    const date = ref<string | undefined>(undefined);
    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    const storageTab = ref("General");
    await httpCommon.apiClient
      .get<Storage>(`api/admin/storage/${props.storageId}`)
      .then((res) => {
        if (res.status == 200) {
          storage.value = res.data;
          date.value = new Date(res.data.created).toLocaleString();
          meta.title = `Nitro Repo - ${res.data.public_name}`;
        }
      });
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
