<template>
  <div v-if="repository !== undefined">
    <MavenUpload :repo="repository" />
  </div>
</template>

<script lang="ts">
import { defineComponent,  ref } from "vue";

import MavenUpload from "@/components/upload/MavenUpload.vue";
import { useRoute } from "vue-router";
import { Repository } from "@/types/repositoryTypes";

export default defineComponent({
  components: { MavenUpload },

  setup() {
    const route = useRoute();

    const storage = route.params.storage as string;
    const repositoryName = route.params.repo as string;
    const repository = ref<Repository | undefined>(undefined);
    // TODO get repository
    return { repository, storage };
  },
});
</script>
