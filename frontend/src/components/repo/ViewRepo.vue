<template>
  <div class="grid grid-row-2 gap-4" v-if="repository !== undefined">
    <div class="m-2">
      <RepositoryBadge :repository="repository" />
    </div>
    <div class="m-2">
      <MavenRepoInfo
        v-if="repository.repo_type === 'Maven'"
        :repository="repository"
      />
    </div>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import MavenRepoInfo from "@/components/repo/types/maven/MavenRepoInfo.vue";
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import RepositoryBadge from "./badge/RepositoryBadge.vue";
import { Repository } from "@/types/repositoryTypes";

export default defineComponent({
  components: { MavenRepoInfo, RepositoryBadge },
  props: {
    storage: {
      required: false,
      type: String,
    },
    repositoryName: {
      required: false,
      type: String,
    },
    repositoryType: {
      required: false,
      type: Object as () => Repository,
    },
  },
  setup(props) {
    const router = useRouter();

    const repository = ref<Repository | undefined>(props.repositoryType);
    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    // TODO pull repository data
    return {
      repository,
      router,
    };
  },
});
</script>
