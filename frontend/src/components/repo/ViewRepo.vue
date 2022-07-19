<template>
  <div class="flex flex-row flex-wrap lg:flex-nowrap">
    <div class="lg:basis-3/4 m-4 text-quaternary">
      <div class="flex flex-row border-b-2">
        <h1 class="text-3xl">
          Repository Description - {{ repository.repository_type }}
        </h1>
        <DynamicIcon
          class="ml-5"
          :repositoryType="repository.repository_type"
        />
      </div>
      <div
        class="mt-5 bg-secondary bg-slate-800 rounded-md p-2"
        v-show="repository.page_content !== ''"
        v-html="repository.page_content"
      ></div>
      <div
        class="mt-5 bg-secondary bg-slate-800 rounded-md p-2"
        v-show="repository.page_content === ''"
      >
        <p class="text-center text-quaternary">No description available.</p>
      </div>
    </div>
    <div class="lg:basis-1/4">
      <div class="grid grid-row-2 gap-4">
        <div class="m-2">
          <RepositoryBadge
            :repository="{ name: repositoryName, storage: storage }"
          />
        </div>
        <div class="m-2">
          <MavenRepoInfo
            v-if="repository.repository_type === 'Maven'"
            :repository="{ name: repositoryName, storage: storage }"
          />
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import MavenRepoInfo from "@/components/repo/types/maven/MavenRepoInfo.vue";
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import RepositoryBadge from "./badge/RepositoryBadge.vue";
import httpCommon from "@/http-common";
import DynamicIcon from "@/components/repo/DynamicIcon.vue";

export default defineComponent({
  components: { DynamicIcon, MavenRepoInfo, RepositoryBadge },
  props: {
    storage: {
      type: String,
    },
    repositoryName: {
      type: String,
    },
  },
  async setup(props) {
    const repository = ref<
      | { repository_type: string; page_content: string; name: string }
      | undefined
    >(undefined);
    useMeta({
      title: `${props.repositoryName} - Nitro Repo`,
    });
    await httpCommon.apiClient
      .get(`api/repositories/${props.storage}/${props.repositoryName}`)
      .then((response) => {
        if (response.status == 200) {
          repository.value = response.data;
        } else {
          //TODO handle 404
          console.error("Error fetching repository ");
        }
      });
    return {
      repository,
    };
  },
});
</script>
