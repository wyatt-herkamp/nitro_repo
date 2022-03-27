<template>
  <div class="m-3">
    <div class="flex flex-wrap">
      <router-link class="py-3 my-1 min-w-max hover:text-red-400" to="/browse">
        <span>Browse</span>
      </router-link>
      <router-link
        class="py-3 my-1 min-w-max hover:text-red-400"
        v-for="value in values"
        :key="value.name"
        :to="'/browse' + '/' + value.path"
      >
        <span>/</span>
        <span> {{ value.name }} </span>
      </router-link>
    </div>
    <div class="flex">
      <div class="min-w-full grid auto-cols-auto grid-rows-1 text-left">
        <div class="link-box" v-for="value in tableData" :key="value.name">
          <router-link
            class="link"
            :to="path + '/' + value.name"
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
import {
  fileListing,
  getRepositoriesPublicAccess,
} from "nitro_repo-api-wrapper";
import { getStoragesPublicAccess } from "nitro_repo-api-wrapper"
import { FileResponse } from "nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import router from "@/router";
import { defineComponent, ref } from "vue";
import { useRoute } from "vue-router";

export default defineComponent({
  setup() {
    let url = apiURL;
    const route = useRoute();
    let values = ref([]);
    const tableData = ref([]);
    const storage = route.params.storage as string;
    const repository = route.params.repo as string;
    let catchAll = route.params.catchAll as string;
    const loading = ref(true);
    const path = route.fullPath;
    if (storage != undefined && storage != "") {
      let value = { name: storage, path: storage };
      values.value.push(value);
      if (repository != undefined && repository != "") {
        let value = {
          name: repository,
          path: values.value[values.value.length - 1].path + "/" + repository,
        };

        values.value.push(value);
        if (route.params.catchAll != undefined) {
          for (var s of catchAll.split("/")) {
            let value = {
              name: s,
              path: values.value[values.value.length - 1].path + "/" + s,
            };

            values.value.push(value);
          }
          loading.value = false;
        } else {
          catchAll = "";
        }
        const getFiles = async () => {
          try {
            const value = (await fileListing(
              storage,
              repository,
              catchAll
            )) as FileResponse[];
            for (const storage of value) {
              console.log(storage);
              tableData.value.push(storage);
            }
            loading.value = false;
          } catch (e) {
            console.error(e);
          }
        };
        getFiles();
      } else {
        const getLocalRepos = async () => {
          try {
            const value = (await getRepositoriesPublicAccess(
              storage
            )) as string[];
            for (const storage of value) {
              tableData.value.push({ name: storage,directory: true });
            }
            loading.value = false;
          } catch (e) {
            console.error(e);
          }
        };
        getLocalRepos();
      }
    } else {
      const getLocalStorage = async () => {
        try {
          const value = (await getStoragesPublicAccess()) as string[];
          for (const storage of value) {
            tableData.value.push({ name: storage, directory: true });
          }
          loading.value = false;
        } catch (e) {
          console.error(e);
        }
      };
      getLocalStorage();
    }
    console.log(values.value);
    return {
      loading,
      values,
      tableData,
      storage,
      repository,
      catchAll,
      path,
      url,
    };
  },
  methods: {
    onRowClick(row: any) {
      if (this.repository != undefined && this.repository != "") {
        let value = row.name as string;
        for (const i of this.tableData) {
          let data = i as FileResponse;
          if (data.name == value) {
            if (!data.directory) {
              return;
            }
          }
        }
        let url = this.catchAll;
        if (url == "") {
          url = value;
        } else {
          url = url + "/" + value;
        }
        router.push({
          name: "Browse",
          params: {
            storage: this.storage,
            repo: this.repository,
            catchAll: url,
          },
        });
      } else if (this.storage != undefined && this.storage != "") {
        let value = row.name as string;

        router.push({
          name: "Browse",
          params: { storage: this.storage, repo: value },
        });
      } else {
        let value = row.name as string;
        router.push({ name: "Browse", params: { storage: value } });
      }
    },
  },
});
</script>
