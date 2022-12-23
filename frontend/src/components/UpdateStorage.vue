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
                <td>{{ date }}</td>
              </tr>
              <tr>
                <th scope="row">Type</th>
                <td>{{ storageType }}</td>
              </tr>
              <tr
                v-for="(value, key) in storage.handler_config[storageType]"
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
import { defineComponent, ref } from "vue";
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
    const storage = ref<Storage | undefined>(undefined);
    const date = ref<string | undefined>(undefined);
    const storageType = ref("");

    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    const storageTab = ref("General");
    await httpCommon.apiClient
      .get<Storage>(`api/admin/storage/${props.storageId}`)
      .then((res) => {
        if (res.status == 200) {
          storage.value = res.data;
          storageType.value = Object.keys(res.data.handler_config)[0];
          date.value = new Date(res.data.created).toLocaleString();
          meta.title = `Nitro Repo - ${res.data.id}`;
        }
      });
    return {
      date,
      storage,
      storageTab,
      storageType,
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
