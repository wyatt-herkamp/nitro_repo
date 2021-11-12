<template>
  <el-container style="border: 1px solid #eee">
    <el-aside width="200px" style="background-color: rgb(238, 241, 246)">
      <el-menu
        default-active="0"
        class="el-menu-vertical-demo content"
        :collapse="false"
      >
        <el-menu-item @click="index = 0" index="0">
          <i class="el-icon-watermelon"></i>
          <template #title>Create new Repository</template>
        </el-menu-item>
        <el-menu-item v-if="isLoading">
          <i class="el-icon-watermelon"></i>
          <template #title>Loading </template>
        </el-menu-item>
        <div v-else-if="error != ''">
          {{ error }} <button @click="getRepos">try again</button>
        </div>
        <el-menu-item
          v-for="repo in repositories.repositories"
          :key="repo.id"
          @click="index = repo.id"
          :index="repo.id"
        >
          <i class="el-icon-watermelon"></i>
          <template #title>{{ repo.name }}</template>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-container class="content">
      <div class="content" v-if="index == 0">
        <CreateRepo :updateList="updateList" />
      </div>
      <div
        class="content"
        v-for="repo in repositories.repositories"
        :key="repo.id"
      >
        <div class="content" v-if="index == repo.id">
          <UpdateRepository :repo="repo" />
        </div>
      </div>
    </el-container>
  </el-container>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import CreateRepo from "@/components/CreateRepo.vue";
import UpdateRepository from "@/components/UpdateRepository.vue";
import { useCookie } from "vue-cookie-next";
import { getRepositories } from "@/backend/api/Repository";
import { DEFAULT_REPO_LIST } from "@/backend/Response";

export default defineComponent({
  components: { CreateRepo, UpdateRepository },

  setup() {
    let index = ref(0);
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
