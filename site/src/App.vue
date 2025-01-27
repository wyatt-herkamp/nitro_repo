<template>
  <header>
    <NavBar :user="user" />
  </header>
  <div
    class="contentWithSideBar"
    v-if="hasSideBar">
    <component :is="router.currentRoute.value.meta.sideBar" />
    <RouterView />
  </div>
  <RouterView v-else />
  <Notifications />
</template>
<script setup lang="ts">
import { RouterView } from "vue-router";
import { siteStore } from "./stores/site";
import router from "./router";
import NavBar from "./components/nav/NavBar.vue";
import { sessionStore } from "./stores/session";
import { computed } from "vue";
import { Notifications } from "@kyvg/vue3-notification";
import { apiURL } from "./config";
import routesJson from "../src/router/routes.json";

const site = siteStore();
const session = sessionStore();
const user = computed(() => session.user);
const hasSideBar = computed(() => {
  return router.currentRoute.value.meta.sideBar !== undefined;
});

if (import.meta.env.MODE === "development") {
  const routes: Array<{ path: string; name: string }> = [];

  for (const route of router.options.routes) {
    if (route.meta?.skipRoutesJson === true) {
      continue;
    }
    routes.push({ path: route.path, name: route.name as string });
  }
  for (const route of routes) {
    const foundRoute = routesJson.find((r) => r.path === route.path && (route.name = r.name));
    if (!foundRoute) {
      console.error(`route not found: ${route.path} update routes.json`);
    } else {
      console.log(`route found: ${route.path}`);
    }
  }
  console.log("");
  console.log(JSON.stringify(routes));
}
console.log(`apiURL: ${apiURL}`);
async function init() {
  const info = await site.getInfo();
  if (info == undefined) {
    console.log("info is undefined");
    return;
  }
  console.log(info);

  if (!info?.is_installed) {
    router.push("/admin/install");
  }
  const session = sessionStore();
  const user = await session.updateUser();
  if (user == undefined) {
    console.log("user is undefined");
    return;
  }
}
init();
</script>
<style scoped lang="scss">
.contentWithSideBar {
  display: flex;
  height: 90vh;
  main {
    flex: 1;
    padding: 1rem;
  }
}
</style>
