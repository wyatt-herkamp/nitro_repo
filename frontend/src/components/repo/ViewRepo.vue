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
      <div class="flex flex-col">
        <div class="m-2 bg-slate-800 pb-5">
          <h1 class="text-white mt-5 ml-5 font-bold">Repository Info</h1>
          <div class="text-white mt-5 ml-5">
            <span
              >Last Updated On {{ last_updated_date }} at
              {{ last_updated_time }}</span
            >
          </div>
        </div>
        <div class="m-2 my-10">
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
    useMeta({
      title: `${props.repositoryName} - Nitro Repo`,
    });
    const repository: {
      repository_type: string;
      page_content: string;
      name: string;
      last_updated: number;
    } = await httpCommon.apiClient
      .get(`api/repositories/${props.storage}/${props.repositoryName}`)
      .then((response) => {
        return response.data;
      })
      .catch((error) => {
        console.error(error);
        return {
          repository_type: "",
          page_content: "",
          name: "",
          last_updated: 0,
        };
      });
    const last_updated_date = new Date(
      repository.last_updated
    ).toLocaleDateString("en-US");
    const last_updated_time = new Date(
      repository.last_updated
    ).toLocaleTimeString("en-US");

    return {
      repository,
      last_updated_date,
      last_updated_time,
    };
  },
});
</script>
