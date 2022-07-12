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
import { getStorage, Storage } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import Repositories from "./Repositories.vue";

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
    const token: string | undefined = inject("token");
    if (token == undefined) {
      await useRouter().push("login");
    }
    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    const storageTab = ref("General");
    try {
      const value = (await getStorage(
        token as string,
        props.storageId
      )) as Storage;
      storage.value = value;
      date.value = new Date(storage.value.created).toLocaleDateString("en-US");
      meta.title = value.name;
    } catch (e) {
      console.log(e);
    }

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
