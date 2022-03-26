<template>
  <CopyMenu :child="child" :repository="repository" />
</template>
<style scoped></style>
<script lang="ts">
import { PublicRepositoryInfo } from "nitro_repo-api-wrapper";
import { Repository } from "nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { defineComponent, ref } from "vue";
import CopyMenu from "@/components/repo/types/maven/copy/CopyMenu.vue";
export default defineComponent({
  components: { CopyMenu },
  props: {
    repository: {
      required: true,
      type: Object as () => Repository | PublicRepositoryInfo,
    },
    child: {
      default: false,
      type: Boolean,
    },
  },
  setup() {
    let url = apiURL;
    return { url };
  },
});
</script>
