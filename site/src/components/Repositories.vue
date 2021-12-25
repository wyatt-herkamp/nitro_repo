<template>

</template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import CreateRepo from "@/components/CreateRepo.vue";
import UpdateRepository from "@/components/UpdateRepository.vue";
import {useCookie} from "vue-cookie-next";
import {getRepositories} from "@/backend/api/Repository";
import {DEFAULT_REPO_LIST} from "@/backend/Response";

export default defineComponent({
  components: { CreateRepo, UpdateRepository },

  setup() {
    const index = ref<number>(0);
    const isLoading = ref(false);
    const cookie = useCookie();

    const error = ref("");
    let repositories = ref(DEFAULT_REPO_LIST);
    const getRepos = async () => {
      isLoading.value = true;
      try {
        const value = await getRepositories(cookie.getCookie("token"));
        repositories.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value = "Error";
      }
    };
    getRepos();
    return {
      cookie,
      index,
      repositories,
      isLoading,
      error,
      getRepos,
    };
  },
  methods: {
    updateList(id: number) {
      console.log("Updating Repos");
      const getRepos = async () => {
        try {
          const value = await getRepositories(this.cookie.getCookie("token"));
          this.repositories = value;
          this.index = id;
        } catch (e) {
          this.error = "Error";
        }
      };
      getRepos();
    },
  },
});
</script>

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>
