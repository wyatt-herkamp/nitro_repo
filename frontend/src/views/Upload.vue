<template>
  <div v-if="repositoryType !== ''">
    <MavenUpload
      v-show="repositoryType === 'Maven'"
      :repo="{ storage: storage, name: repositoryName }"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";

import MavenUpload from "@/components/upload/MavenUpload.vue";
import { useRoute } from "vue-router";
import httpCommon from "@/http-common";

export default defineComponent({
  components: { MavenUpload },

  setup() {
    const route = useRoute();

    const storage = route.params.storage as string;
    const repositoryName = route.params.repo as string;
    const repositoryType = ref<string>("");

    httpCommon.apiClient
      .get<{ repository_type: string }>(`api/repositories/${storage}/${repositoryName}`)
      .then((response) => {
        console.log(response.data)
        repositoryType.value = response.data.repository_type;
      })
      .catch((error) => {
        console.error(error);
        return {
          repository_type: "",
          page_content: "",
          name: "",
          last_updated: 0,
        };
      });
    // TODO get repository
    return { repositoryType, storage, repositoryName };
  },
});
</script>
