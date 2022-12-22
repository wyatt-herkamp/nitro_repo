<template>
  <ul class="listOfItems">
    <BrowseBox
      v-for="value in repositories"
      :key="value"
      :file="{
        name: value.name,
        full_path: `${storage}/${value.name}`,
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
  name: "ListRepositories",
  components: { BrowseBox },
  props: {
    storage: {
      type: String,
      required: true,
    },
    modelValue: {
      type: Object as () => BrowsePath[],
      required: true,
    },
  },

  async setup(props) {
    const repositories = await httpCommon.apiClient
      .get<{ name: string }[]>(`/api/repositories/${props.storage}`)
      .then((response) => response.data);

    return { repositories };
  },
});
</script>

