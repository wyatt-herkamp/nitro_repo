<template>

</template>

<script lang="ts">
import {fileListing, getRepositoriesPublicAccess,} from "@/backend/api/Repository";
import {getStoragesPublicAccess} from "@/backend/api/Storages";
import {FileResponse} from "@/backend/Response";
import router from "@/router";
import {defineComponent, ref} from "vue";
import {useRoute} from "vue-router";
import {ArrowRight} from "@element-plus/icons";

export default defineComponent({
  setup() {
    const route = useRoute();
    let values = ref<string[]>([]);
    const tableData = ref([{}]);
    const storage = route.params.storage as string;
    const repository = route.params.repo as string;
    let catchAll = route.params.catchAll as string;
    const loading = ref(true);

    if (storage != undefined && storage != "") {
      values.value.push(storage);
      if (repository != undefined && repository != "") {
        values.value.push(repository as string);
        if (route.params.catchAll != undefined) {
          for (var s of catchAll.split("/")) {
            values.value.push(s);
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
              tableData.value.push({ name: storage });
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
            tableData.value.push({ name: storage });
          }
          loading.value = false;
        } catch (e) {
          console.error(e);
        }
      };
      getLocalStorage();
    }
    return {
      loading,
      values,
      tableData,
      storage,
      repository,
      catchAll,
      ArrowRight,
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
<style scoped>
.pointer:hover {
  cursor: pointer;
}
</style>
