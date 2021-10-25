<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-breadcrumb separator="/">
      <el-breadcrumb-item v-for="path in values" :key="path">{{
        path
      }}</el-breadcrumb-item>
    </el-breadcrumb>
    <el-main>
      <h1>Welcome to Nitro Repo Browse 0.1.0</h1>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRoute } from "vue-router";

export default defineComponent({
  setup() {
    const route = useRoute();
    let values = ref([""]);
    console.log(route.params);

    if (route.params.storage != undefined) {
      values.value.push(route.params.storage as string);
      if (route.params.repo != undefined) {
        values.value.push(route.params.repo as string);
        if (route.params.catchAll!=undefined){
          let catchAll = route.params.catchAll as string;
          for (var s of catchAll.split("/")){
            values.value.push(s);
          }
        }
      }
    }
    const path = ref({
      values: values,
    });
    return { values };
  },
});
</script>
