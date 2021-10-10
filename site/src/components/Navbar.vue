<template>
  <el-menu :default-active="activeIndex" class="el-menu-demo" mode="horizontal">
    <el-menu-item index="Home" @click="router.push('/')">Index</el-menu-item>
    <el-menu-item index="Browse" @click="router.push('browse')">Browse</el-menu-item>
    <el-menu-item v-if="user.id != 0" index="Admin" @click="router.push('admin')"
      >Admin</el-menu-item
    >
    <el-menu-item v-else index="Login" @click="router.push('login')"
      >Login</el-menu-item
    >
  </el-menu>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import userStore from "@/store/user";
import { useRouter } from "vue-router";
import { User } from "@/backend/Response";
export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup() {
    const router = useRouter();
    const activeIndex = ref(router.currentRoute.value.name);
    return { activeIndex, router };
  },
});
</script>