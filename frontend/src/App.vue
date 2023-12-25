<template>
  <metainfo> </metainfo>
  <div id="container">
    <Navbar :user="user" />
    <router-view :key="$route.fullPath" />
  </div>

  <notifications position="bottom right" />
</template>
<style scoped>
#container {
  height: 100vh;
}
</style>
<script setup lang="ts">
import Navbar from "@/components/nav/Navbar.vue";
import { sessionStore } from "./store/session";
const user = sessionStore().account;
</script>

