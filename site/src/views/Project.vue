<template>
  <el-container>
    <el-aside width="200px">Aside</el-aside>
    <el-container>
      <el-header>{{ project. }}</el-header>
      <el-main>Main</el-main>
    </el-container>
  </el-container>

</template>

<script lang="ts">
import {getProject} from "@/backend/api/Repository";
import {Project} from "@/backend/Response";
import {defineComponent, ref} from "vue";
import {useRoute} from "vue-router";

export default defineComponent({
  setup() {
    const route = useRoute();
    const storage = route.params.storage as string;
    const repository = route.params.repo as string;
    let catchAll = route.params.catchAll as string;
    const project = ref<Project | undefined>(undefined);
    const getInfo = async () => {
      let value = await getProject(storage, repository, catchAll);
      project.value = value;
    };
    getInfo();
    return {project}
  },
});
</script>
<style scoped>
.pointer:hover {
  cursor: pointer;
}
</style>
