<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-breadcrumb separator="/">
      <el-breadcrumb-item v-for="path in values" :key="path">{{
        path
      }}</el-breadcrumb-item>
    </el-breadcrumb>
    <el-main>
      <h1>Welcome to Nitro Repo Browse 0.1.0</h1>
      <el-table class="pointer" :data="tableData" @row-click="onRowClick" style="width: 100%">
        <el-table-column prop="name" label="name" />
      </el-table>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import {
  fileListing,
  getRepositoriesPublicAccess,
} from "@/backend/api/Repository";
import { getStorages, getStoragesPublicAccess } from "@/backend/api/Storages";
import router from "@/router";
import { defineComponent, ref } from "vue";
import { useRoute } from "vue-router";

export default defineComponent({
  setup() {
    const route = useRoute();
    let values = ref([""]);
    const tableData = ref([{}]);
    console.log(route.params);
    const storage = route.params.storage as string;
    const repository = route.params.repo as string;
    const catchAll = route.params.catchAll as string;

    if (storage != undefined && storage != "") {
      values.value.push(storage);
      if (repository != undefined && repository != "") {
        values.value.push(repository as string);
        if (route.params.catchAll != undefined) {
          for (var s of catchAll.split("/")) {
            values.value.push(s);
          }
        }
        const getFiles = async () => {
          try {
            const value = (await fileListing(
              storage,
              repository,
              catchAll
            )) as string[];
            for (const storage of value) {
              console.log(storage);
              tableData.value.push({ name: storage });
            }
          } catch (e) {
            console.log(e);
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
              console.log(storage);
              tableData.value.push({ name: storage });
            }
          } catch (e) {
            console.log(e);
          }
        };
        getLocalRepos();
      }
    } else {
      const getLocalStorage = async () => {
        try {
          const value = (await getStoragesPublicAccess()) as string[];
          for (const storage of value) {
            console.log(storage);
            tableData.value.push({ name: storage });
          }
        } catch (e) {
          console.log(e);
        }
      };
      getLocalStorage();
    }
    return { values, tableData, storage, repository, catchAll };
  },
  methods: {
    onRowClick(row: any) {
      if (this.repository != undefined && this.repository != "") {
        let value = row.name as string;

        let url = this.catchAll;
        if (url == "") {
          url = value;
        }else{
          url = url +"/"+value;
        }
        console.log("Path " + url);
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
<style scoped>
.pointer:hover{
  cursor: pointer;
}
</style>