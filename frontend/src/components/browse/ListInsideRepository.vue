<template>
  <ul class="w-full text-left p-3">
    <BrowseBox v-for="value in files" :key="value" :file="value" />
  </ul>
</template>

<script lang="ts">
import { defineComponent, ref, watch } from "vue";
import { BrowsePath } from "@/api/Browse";
import httpCommon from "@/http-common";
import BrowseBox from "@/components/browse/BrowseBox.vue";
import { FileResponse } from "@/types/repositoryTypes";

export default defineComponent({
  name: "ListInsideRepository",
  components: { BrowseBox },
  props: {
    storage: {
      type: String,
      required: true,
    },
    repository: {
      type: String,
      required: true,
    },
    catchAll: {
      type: String,
      required: true,
      default: "",
    },
    modelValue: {
      type: Object as () => BrowsePath[],
      required: true,
    },
  },

  async setup(props) {
    const files = ref<FileResponse[]>([]);

    await httpCommon.apiClient
      .get(
        `/repositories/${props.storage}/${props.repository}/${props.catchAll}`
      )
      .then((response) => {
        files.value = response.data.files;
      });

    return { files };
  },
});
</script>

<style scoped></style>
