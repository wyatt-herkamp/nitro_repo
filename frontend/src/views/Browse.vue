<template>
  <div class="flex flex-col m-3">
    <div class="flex flex-wrap">
      <router-link class="py-3 my-1 min-w-max hover:text-red-400" to="/browse">
        <span>Browse</span>
      </router-link>
      <router-link
        class="py-3 my-1 min-w-max hover:text-red-400"
        v-for="value in pathSplit"
        :key="value.name"
        :to="'/browse' + value.path"
      >
        <span>/</span>
        <span> {{ value.name }} </span>
      </router-link>
    </div>
    <div class="flex">
      <div
        v-if="tableData != undefined"
        class="min-w-full grid auto-cols-auto grid-rows-1 text-left"
      >
        <div class="link-box" v-for="value in tableData" :key="value.name">
          <router-link
            class="link"
            :to="'/browse/' + value.full_path"
            v-if="value.directory"
          >
            <span class="linkText">
              {{ value.name }}
            </span>
          </router-link>
          <a
            class="link"
            :href="url + '/storages/' + value.full_path"
            v-if="!value.directory"
          >
            <span class="linkText">
              {{ value.name }}
            </span>
          </a>
        </div>
      </div>
    </div>
    <div class="flex flex-wrap" v-if="activeResponse != undefined">
      <ViewProject
        v-if="activeResponse.Project != undefined"
        :project="activeResponse.Project"
        :child="true"
      />
      <ViewRepo
        v-if="activeResponse.Repository != undefined"
        :repository="activeResponse.Repository.name"
        :storage="activeResponse.Repository.storage"
        :child="true"
      />
    </div>
  </div>
</template>
<style scoped>
.link {
  @apply block;
  @apply min-w-max;

  @apply hover:text-red-400;
  @apply p-3;
}
.linkText {
  @apply pl-2;
}
.link-box {
  @apply min-w-max;
  @apply my-1;
  @apply py-1;
  @apply border-2;
}
</style>
<script lang="ts">
import { browse, BrowseResponse, FileResponse } from "nitro_repo-api-wrapper";

import { apiURL } from "@/http-common";
import router from "@/router";
import { defineComponent, ref } from "vue";
import { useRoute } from "vue-router";
import { useCookie } from "vue-cookie-next";
import { BrowsePath } from "./Browse";
import ViewProject from "@/components/project/ViewProject.vue";
import ViewRepo from "@/components/repo/ViewRepo.vue";

export default defineComponent({
  setup() {
    let url = apiURL;
    const route = useRoute();
    let pathSplit = ref<BrowsePath[]>([]);
    const tableData = ref<FileResponse[] | undefined>();
    const activeResponse = ref<ResponseType | undefined>();
    let catchAll = route.params.catchAll as string;
    const cookie = useCookie();
    const path = route.fullPath;
    const getFiles = async () => {
      try {
        const value = await browse(catchAll, cookie.getCookie("token"));
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
        activeResponse.value;
        if (
          fileResponse.response_type != undefined &&
          typeof fileResponse.response_type != "string"
        ) {
          activeResponse.value = fileResponse.response_type as ResponseType;
          console.log(activeResponse.value);
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
      activeResponse,
    };
  },
  components: { ViewProject, ViewRepo },
});
</script>
