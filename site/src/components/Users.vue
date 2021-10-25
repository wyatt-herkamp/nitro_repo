<template>
  <el-container style="border: 1px solid #eee">
    <el-aside width="200px" style="background-color: rgb(238, 241, 246)">
      <el-menu
        default-active="0"
        class="el-menu-vertical-demo"
        :collapse="false"
      >
        <el-menu-item @click="index = 0" index="0">
          <i class="el-icon-watermelon"></i>
          <template #title>Create new Storage</template>
        </el-menu-item>
        <el-menu-item v-if="isLoading">
          <i class="el-icon-watermelon"></i>
          <template #title>Loading </template>
        </el-menu-item>
        <div v-else-if="error != ''">
          {{ error }} <button @click="getUser">try again</button>
        </div>
        <el-menu-item
          v-for="user in users.users"
          :key="user.id"
          @click="index = user.id"
        >
          <i class="el-icon-user"></i>
          <template #title>{{ user.name }}</template>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-container>
      <div v-if="index == 0">
        <CreateUser />
      </div>
      <div v-for="user in users.users" :key="user.id">
        <div v-if="index == user.id">
          <UpdateUser :user="user" :me="false" />
        </div>
      </div>
    </el-container>
  </el-container>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import CreateUser from "@/components/CreateUser.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import { useCookie } from "vue-cookie-next";
import { getUsers } from "@/backend/api/User";
import { DEFAULT_USER_LIST } from "@/backend/Response";

export default defineComponent({
  components: { CreateUser, UpdateUser },

  setup() {
    let index = ref(0);
    const cookie = useCookie();
    const isLoading = ref(false);

    let error = ref("");
    let users = ref(DEFAULT_USER_LIST);
    const getUser = async () => {
      isLoading.value = true;
      try {
        const value = await getUsers(cookie.getCookie("token"));
        users.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value="Error Loading";
      }
    };
    getUser();
    return {
      index,
      users,
      isLoading,
      error,
      getUser,
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
