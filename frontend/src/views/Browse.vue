<template>
  <div class="min-h-screen w-full flex flex-wrap lg:flex-nowrap">
    <div class="w-full max-w-full m-2 rounded-md bg-gray-900">
      <div class="md:flex md:flex-wrap md:ml-2">
        <div class="m-5">
          <router-link class="backLink" to="/browse">
            <span>Browse</span>
          </router-link>
          <router-link
            class="backLink mx-1 sm:m-0 inline-block sm:inline"
            v-for="value in pathSplit"
            :key="value.name"
            :to="'/browse' + value.path"
          >
            <span>/</span>
            <span> {{ value.name }} </span>
          </router-link>
        </div>
        <div class="hidden md:flex flex-row my-5">
          <router-link class="align-middle inline-block" :to="up">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              class="my-auto"
              style="fill: rgba(255, 255, 255, 1); transform: ; msfilter: "
            >
              <path
                d="M21 11H6.414l5.293-5.293-1.414-1.414L2.586 12l7.707 7.707 1.414-1.414L6.414 13H21z"
              ></path>
            </svg>
          </router-link>
        </div>
      </div>
      <div class="md:flex md:flex-wrap md:ml-2">
        <div
          v-if="tableData != undefined"
          class="w-full grid auto-cols-auto grid-rows-1 text-left p-3"
        >
          <BrowseBox
            v-for="value in tableData"
            :key="value.name"
            :file="value"
          />
        </div>
      </div>
    </div>

    <div
      class="hidden md:block float-right lg:w-1/4 m-2 rounded-md bg-slate-900"
      v-if="activeResponse != undefined"
    >
      <div v-if="activeResponse.Project != undefined">
        <router-link
          :to='{
            name: "Project",
            params: {
              storage: activeResponse.Project.repo_summary.storage,
              repo: activeResponse.Project.repo_summary.name,
              id: activeResponse.Project.version.name,
            },
          }'
          >Project Page</router-link
        >
        <ViewProject :project="activeResponse.Project" />
      </div>

      <div v-if="activeResponse.Repository != undefined">
        <router-link
          :to='{
            name: "ViewRepository",
            params: {
              storage: activeResponse.Repository.storage,
              repo: activeResponse.Repository.name,
            },
          }'
          >Repository Page</router-link
        >
        <ViewRepo
          :repositoryName="activeResponse.Repository.name"
          :storage="activeResponse.Repository.storage"
        />
      </div>
    </div>
  </div>
</template>
<style scoped>
.backLink {
  @apply md:py-3;
  @apply md:my-1;
  @apply min-w-max;

  @apply hover:text-slate-300;
  @apply transition;
  @apply ease-in-out;
  @apply duration-100;
}
</style>
<script lang="ts">
import { browse, BrowseResponse, FileResponse } from "nitro_repo-api-wrapper";

import { apiURL } from "@/http-common";
import { defineComponent, inject, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { BrowsePath } from "./Browse";
import ViewProject from "@/components/project/ViewProject.vue";
import ViewRepo from "@/components/repo/ViewRepo.vue";
import BrowseBox from "@/components/browse/BrowseBox.vue";

export default defineComponent({
  setup() {
    const token: string | undefined = inject("token");

    let url = apiURL;
    const route = useRoute();
    let pathSplit = ref<BrowsePath[]>([]);
    const tableData = ref<FileResponse[] | undefined>();
    const activeResponse = ref<ResponseType | undefined>();
    const catchAll = ref(route.params.catchAll as string);
    const path = ref("");

    const up = ref("");
    const getFiles = async () => {
      path.value = route.fullPath;
      catchAll.value = route.params.catchAll as string;

      var upperPath = path.value.split("/");

      if (upperPath.length > 0) {
        upperPath.splice(upperPath.length - 1);
      }
      up.value = upperPath.join("/");
      try {
        const value = await browse(route.params.catchAll as string, token);
        if (value == undefined) {
          console.warn("No Response from Backend");
          return;
        }
        const fileResponse: BrowseResponse = value as BrowseResponse;
        {
          // Generates the needed information for the path
          let url = "";
          fileResponse.active_dir.split("/").forEach((element) => {
            url = url + "/" + element;
            pathSplit.value.push({ name: element, path: url });
          });
        }
        if (
          fileResponse.response_type != undefined &&
          typeof fileResponse.response_type != "string"
        ) {
          activeResponse.value = fileResponse.response_type as ResponseType;
        }
        tableData.value = value.files;
      } catch (e) {
        console.error(e);
      }
    };

    getFiles();
    return {
      path,
      tableData,
      catchAll,
      pathSplit,
      url,
      up,
      activeResponse,
    };
  },
  components: { ViewProject, ViewRepo, BrowseBox },
});
</script>
