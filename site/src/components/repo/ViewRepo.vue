<template>
  <div v-if="repository != undefined">
    <h1 class="text-slate-50">
      {{ repository.storage }}/{{ repository.name }}
      <RepoInfo :repository="repository" />
    </h1>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { getRepoByNameAndStorage } from "@/backend/api/Repository";
import { Repository } from "@/backend/Response";
import RepoInfo from "@/components/repo/types/maven/RepoInfo.vue";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";

export default defineComponent({
  components: { RepoInfo },
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
  },
  setup(props) {
    const router = useRouter();

    const options = ref([
      { value: "DeployerUsername", label: "Deploy Username" },
      { value: "Time", label: "Time" },
    ]);
    let repository = ref<Repository | undefined>(props.repositoryType);
    let date = ref<string | undefined>(undefined);
    const cookie = useCookie();
    const isLoading = ref(props.repositoryType == undefined);
    const exampleBadgeURL = ref("");
    const { meta } = useMeta({
      title: "Nitro Repo",
    });
    if (repository.value != undefined) {
      if (props.repository != undefined && props.storage != undefined) {
        const getRepo = async () => {
          try {
            const value = (await getRepoByNameAndStorage(
              cookie.getCookie("token"),
              props.storage,
              props.repository
            )) as Repository;
            repository.value = value;
            date.value = new Date(repository.value.created).toLocaleDateString(
              "en-US"
            );
            meta.title = value.name;
          } catch (e) {
            console.log(e);
          }
        };
        getRepo();
      }
    } else {
      date.value = new Date(repository.value.created).toLocaleDateString(
        "en-US"
      );
      meta.title = repository.value.name;
    }

    return {
      date,
      exampleBadgeURL,
      repository,
      router,
      options,
      isLoading,
    };
  },
});
</script>
