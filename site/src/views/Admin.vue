<template>
  <el-container style="border: 1px solid #eee">
    <el-aside width="200px" style="background-color: rgb(238, 241, 246)">
      <el-menu
        default-active="4"
        class="el-menu-vertical-demo"
        :collapse="isCollapse"
      >
        <el-menu-item @click="index = 1" index="1">
          <i class="el-icon-menu"></i>
          <template #title>Storages</template>
        </el-menu-item>
        <el-menu-item @click="index = 2" index="2">
          <i class="el-icon-menu"></i>
          <template #title>Repositories</template>
        </el-menu-item>
        <el-menu-item @click="index = 3" index="3">
          <i class="el-icon-user"></i>
          <template #title>User</template>
        </el-menu-item>
        <el-menu-item @click="index = 4" index="4">
          <i class="el-icon-setting"></i>
          <template #title>Setting</template>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-container>
      <div v-if="index == 1">
        <Storages />
      </div>
      <div v-else-if="index == 2">
        <Repositories />

      </div>
      <div v-else-if="index == 3">
        <Users />

      </div>
      <div v-else-if="index == 4">
        <h1>Settings</h1>
        <p>Paragraph 1</p>
        <p>Paragraph 2</p>
      </div>
    </el-container>
  </el-container>
</template>


<script lang="ts">
import { defineComponent, ref } from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import Repositories from "@/components/Repositories.vue";
export default defineComponent({
  components: { Storages, Repositories, Users },
  setup() {
    const isCollapse = ref(false);
    let index = ref(4);

    return {
      isCollapse,
      index,
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