<template>
  <div v-show="repository">
    <UpdateRepo :repository="repository" />
  </div>
</template>
<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import UpdateRepo from "@/components/repo/update/UpdateRepo.vue";
import httpCommon, { apiURL } from "@/http-common";
import { Repository } from "@/types/repositoryTypes";

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
    await httpCommon.apiClient
      .get<Repository>(
        `admin/repositories/${props.storageId}/${props.repositoryId}`
      )
      .then((response) => {
        repository.value = response.data;
      });
    return {
      repository,
      router,
      view,
      url,
    };
  },
});
</script>
