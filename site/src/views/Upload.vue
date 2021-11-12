<template>
  <el-container style="border: 1px solid #eee">
    <MavenUpload
      v-if="repository != undefined && repository.repo_type == 'maven'"
      :repo="repository" :storage="storage"
    />
  </el-container>
</template>

<script lang="ts">
import { defineComponent, onBeforeMount, ref } from "vue";

import userStore from "@/store/user";
import { Repository } from "@/backend/Response";
import { useCookie } from "vue-cookie-next";
import MavenUpload from "@/components/upload/MavenUpload.vue";
import {
  getRepoByNameAndStorage,
} from "@/backend/api/Repository";
import { useRoute } from "vue-router";

export default defineComponent({
  components: { MavenUpload },

  setup() {
    const route = useRoute();
    const cookie = useCookie();

    const storage = route.params.storage as string;
    const repositoryName = route.params.repo as string;
    const repository = ref<Repository | undefined>(undefined);
    const getRepo = async () => {
      try {
        const value = await getRepoByNameAndStorage(
          cookie.getCookie("token"),
          storage,
          repositoryName
        );
        repository.value = value;
      } catch (e) {
        console.log(e);
      }
    };
    getRepo();
    onBeforeMount(userStore.getUser);
    return { repository ,storage};
  },
});
</script>

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>
