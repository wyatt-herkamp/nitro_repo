<template>
  <el-container style="border: 1px solid #eee">
    <el-aside width="200px" style="background-color: rgb(238, 241, 246)">
      <el-menu
        default-active="1"
        class="el-menu-vertical-demo"
        :collapse="isCollapse"
      >
        <el-menu-item @click="index = 1" index="1">
          <i class="el-icon-watermelon"></i>
          <template #title>Create new Storage</template>
        </el-menu-item>
        <el-menu-item v-if="isLoading">
          <i class="el-icon-watermelon"></i>
          <template #title>Loading </template>
        </el-menu-item>
        <div v-else-if="error != null">
          {{ error.message }} <button @click="getStorage">try again</button>
        </div>
        <el-menu-item v-else v-for="storage in storages.storages" :key="storage.id">
          <i class="el-icon-watermelon"></i>
          <template #title>{{ storage.name }}</template>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-container>
      <div v-if="index == 1">
        <CreateStorage />
      </div>
    </el-container>
  </el-container>
</template>


<script lang="ts">
import { defineComponent, ref } from "vue";
import CreateStorage from "@/components/CreateStorage.vue";
import { useCookie } from "vue-cookie-next";
import { getStorages } from "@/backend/api/Storages";
import { DEFAULT_STORAGE_LIST } from "@/backend/Response";

export default defineComponent({
  components: { CreateStorage },

  setup() {
    const isCollapse = ref(false);
    let index = ref(1);
    const isLoading = ref(false);
    const cookie = useCookie();

    const error = ref(null);
    const storages = ref(DEFAULT_STORAGE_LIST);
    const getStorage = async () => {
      isLoading.value = true;
      try {
        const value = await getStorages(cookie.getCookie("token"));
        storages.value = value;
        isLoading.value = false;
      } catch (e) {
        error.value = e;
      }
    };
    return {
      isCollapse,
      index,
      storages,
      isLoading,
      error,
      getStorage,
    };
  },
});
</script>

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>