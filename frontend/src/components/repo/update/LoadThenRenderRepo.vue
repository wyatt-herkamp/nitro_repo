<template>
  <div v-show="repository">
    <UpdateRepo :repository="repository" />
  </div>
</template>
<script lang="ts">
import {
  getRepoByNameAndStorage,
  Repository,
} from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref } from "vue";
import { useRouter } from "vue-router";
import UpdateRepo from "@/components/repo/update/UpdateRepo.vue";
import { apiURL } from "@/http-common";

export default defineComponent({
  components: {
    UpdateRepo,
  },
  props: {
    storageId: {
      type: String,
      required: true,
    },
    repositoryId: {
      type: String,
      required: true,
    },
  },
  async setup(props) {
    const url = apiURL;

    const router = useRouter();
    const view = ref("General");

    const repository = ref<Repository | undefined>(undefined);
    const token: string | undefined = inject("token");
    if (token == undefined) {
      await useRouter().push("login");
    }

    repository.value = (await getRepoByNameAndStorage(
      token as string,
      props.storageId,
      props.repositoryId
    )) as Repository;

    return {
      repository,
      router,
      view,
      url,
    };
  },
});
</script>
