<template>
  <div>
    <SimpleTabs>
      <SimpleTab name="General">
        <div class="w-1/2 mx-auto bg-tertiary mt-5 rounded-l">
          <h1 class="text-quaternary text-2xl mx-2 mt-4 border-b-2 w-fit px-2">
            Storage Configuration
          </h1>
          <table class="table-auto text-quaternary">
            <tbody>
              <tr>
                <th scope="row">ID/Name</th>
                <td>{{ storage.id }}</td>
              </tr>
              <tr>
                <th scope="row">Created</th>
                <td>{{ storage.created.toLocaleString() }}</td>
              </tr>
              <tr>
                <th scope="row">Type</th>
                <td>{{ storage.handler_config.storage_type }}</td>
              </tr>
              <tr
                v-for="(value, key) in storage.handler_config.settings"
                v-bind:key="key"
              >
                <th scope="row">{{ key }}</th>
                <td>{{ value }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </SimpleTab>
      <SimpleTab name="Repositories">
        <Repositories :storage="storage" />
      </SimpleTab>
    </SimpleTabs>
  </div>
</template>
<script lang="ts">
import { defineComponent, Ref, ref } from "vue";
import { useMeta } from "vue-meta";
import Repositories from "./Repositories.vue";
import httpCommon from "@/http-common";
import { Storage } from "@/types/storageTypes";
import SimpleTab from "@/components/common/simple_tabs/SimpleTab.vue";
import SimpleTabs from "@/components/common/simple_tabs/SimpleTabs.vue";

export default defineComponent({
  props: {
    storageId: {
      type: String,
      required: true,
    },
  },
  async setup(props) {
    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    const storageTab = ref("General");
    const storage: Ref<Storage> = await httpCommon.apiClient
      .get<Storage>(`api/admin/storage/${props.storageId}`)
      .then((res) => {
        if (res.status == 200) {
          meta.title = `Nitro Repo - ${res.data.id}`;
          return ref(res.data);
        } else {
          return ref({} as Storage);
        }
      });
    return {
      storage,
      storageTab,
    };
  },
  methods: {
    async deleteStorage() {
      console.log("TODO");
    },
  },
  components: { SimpleTabs, SimpleTab, Repositories },
});
</script>
