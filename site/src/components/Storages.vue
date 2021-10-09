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
      </el-menu>
    </el-aside>
    <el-container>
      <div v-if="index == 1">
         <CreateStorage/>
      </div>
    </el-container>
  </el-container>
</template>


<script lang="ts">
import { defineComponent, ref } from "vue";
import CreateStorage from "@/components/CreateStorage.vue";
export default defineComponent({
  components: { CreateStorage  },
  setup() {
    const isCollapse = ref(false);
    let index = ref(1);

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