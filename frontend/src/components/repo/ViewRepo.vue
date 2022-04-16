<template>
  <div class="grid grid-row-2 gap-4" v-if="repository != undefined">
    <div class=" m-2 ">
      <RepositoryBadge :repository="repository" />
    </div>
    <div class=" m-2">
      <MavenRepoInfo
        v-if="repositoryType === 'Maven'"
        :repository="repository"
      />
    </div>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { getRepoPublic, PublicRepositoryInfo } from "nitro_repo-api-wrapper";
import { Repository } from "nitro_repo-api-wrapper";
import MavenRepoInfo from "@/components/repo/types/maven/MavenRepoInfo.vue";
import { defineComponent, inject, ref } from "vue";
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
    repositoryName: {
      required: false,
      type: String,
    },
    repositoryType: {
      required: false,
      type: Object as () => Repository | PublicRepositoryInfo,
    },
  },
  setup(props) {
    const router = useRouter();

    let repository = ref<Repository | PublicRepositoryInfo | undefined>(
      props.repositoryType
    );
    const token: string | undefined = inject("token");
    const repositoryType = ref("");
    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    if (repository.value == undefined) {
      if (props.repositoryName != undefined && props.storage != undefined) {
        const getRepo = async () => {
          try {
            const value = (await getRepoPublic(
              token,
              props.storage as string,
              props.repositoryName as string
            )) as PublicRepositoryInfo;
            repository.value = value;
            repositoryType.value = value.repo_type;
            meta.title = value.name;
          } catch (e) {
            console.log(e);
          }
        };
        getRepo();
      }
    } else {
      meta.title = repository.value.name;
      repositoryType.value = Object.keys(repository.value.repo_type)[0];
    }

    return {
      repository,
      router,
      repositoryType,
    };
  },
});
</script>
