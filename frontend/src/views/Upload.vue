<template>
  <div v-if="repository != undefined">
    <MavenUpload :repo="repository"/>
  </div>
</template>

<script lang="ts">
import { defineComponent, inject, onBeforeMount, ref } from "vue";

import userStore from "@/store/user";
import MavenUpload from "@/components/upload/MavenUpload.vue";
import { useRoute } from "vue-router";
import { getRepoByNameAndStorage, Repository } from "nitro_repo-api-wrapper";

export default defineComponent({
  components: { MavenUpload },

  setup() {
    const route = useRoute();
    const token: string | undefined = inject('token')

    const storage = route.params.storage as string;
    const repositoryName = route.params.repo as string;
    const repository = ref<Repository | undefined>(undefined);
    const getRepo = async () => {
      try {
        const value = await getRepoByNameAndStorage(
          token,
          storage,
          repositoryName
        );
        repository.value = value;
      } catch (e) {
        console.log(e);
      }
    };
    getRepo();
    onBeforeMount(userStore.getUser);
    return { repository, storage };
  },
});
</script>
