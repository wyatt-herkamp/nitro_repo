<template>
  <div v-if="repository != undefined">
    <RepositoryBadge :repository="repository" :child="child" />
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { getRepoPublic, PublicRepositoryInfo } from "nitro_repo-api-wrapper";
import { Repository } from "nitro_repo-api-wrapper";
import MavenRepoInfo from "@/components/repo/types/maven/MavenRepoInfo.vue";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import RepositoryBadge from "./badge/RepositoryBadge.vue";

export default defineComponent({
  components: { MavenRepoInfo, RepositoryBadge },
  props: {
    storage: {
      required: false,
      type: String,
    },
    repository: {
      required: false,
      type: String,
    },
    repositoryType: {
      required: false,
      type: Object as () => Repository,
    },
    child: {
      default: false,
      type: Boolean,
    },
  },
  setup(props) {
    const router = useRouter();

    const options = ref([
      { value: "DeployerUsername", label: "Deploy Username" },
      { value: "Time", label: "Time" },
    ]);
    let repository = ref<Repository | PublicRepositoryInfo | undefined>(
      props.repositoryType
    );
    const cookie = useCookie();
    const isLoading = ref(props.repositoryType == undefined);
    const exampleBadgeURL = ref("");
    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    if (repository.value == undefined) {
      if (props.repository != undefined && props.storage != undefined) {
        const getRepo = async () => {
          try {
            const value = (await getRepoPublic(
              cookie.getCookie("token"),
              props.storage,
              props.repository
            )) as PublicRepositoryInfo;
            repository.value = value;
            meta.title = value.name;
          } catch (e) {
            console.log(e);
          }
        };
        getRepo();
      }
    } else {
      meta.title = repository.value.name;
    }

    return {
      exampleBadgeURL,
      repository,
      router,
      options,
      isLoading,
    };
  },
});
</script>
