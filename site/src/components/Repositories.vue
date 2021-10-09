<template>
  <el-container style="height: 500px; border: 1px solid #eee">
    <el-aside width="200px" style="background-color: rgb(238, 241, 246)">
      <el-menu
        default-active="1"
        class="el-menu-vertical-demo"
        :collapse="isCollapse"
      >
        <el-menu-item @click="index = 1" index="1">
          <i class="el-icon-watermelon"></i>
          <template #title>Create new Repository</template>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-container>
      <div v-if="index == 1">
         <CreateRepo/>
      </div>
    </el-container>
  </el-container>
</template>


<script lang="ts">
import { defineComponent, ref } from "vue";
import CreateRepo from "@/components/CreateRepo.vue";
export default defineComponent({
  components: { CreateRepo  },
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