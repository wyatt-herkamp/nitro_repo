<template>
  <ul v-if="storages" class="listOfItems">
    <BrowseBox
      v-for="value in storages.storages"
      :key="value"
      :file="{
        name: value,
        full_path: value,
        created: 0,
        file_size: 0,
        directory: true,
        response_type: undefined,
      }"
    />
  </ul>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import httpCommon from "@/http-common";
import { BrowsePath } from "@/api/Browse";
import BrowseBox from "@/components/browse/BrowseBox.vue";
export default defineComponent({
  name: "ListStorages",
  components: { BrowseBox },
  modelValue: {
    type: Object as () => BrowsePath[],
    required: true,
  },
  async setup() {
    const storages = await httpCommon.apiClient
      .get<{ storages: string[] }>("/api/storages")
      .then((response) => response.data);
    console.log(storages);
    return { storages };
  },
});
</script>

