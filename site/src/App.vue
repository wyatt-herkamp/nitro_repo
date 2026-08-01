<template>
  <NavBar :user="user" />

  <div
    v-if="hasSideBar"
    class="withSidebar">
    <component :is="router.currentRoute.value.meta.sideBar" />
    <RouterView />
  </div>
  <RouterView v-else />

  <ModalsContainer />
  <Notifications position="bottom right" />
</template>

<script setup lang="ts">
import { RouterView } from "vue-router";
import { siteStore } from "./stores/site";
import router from "./router";
import NavBar from "./components/nav/NavBar.vue";
import { sessionStore } from "./stores/session";
import { computed } from "vue";
import { Notifications } from "@kyvg/vue3-notification";
import { ModalsContainer } from "vue-final-modal";

const site = siteStore();
const session = sessionStore();
const user = computed(() => session.user);
const hasSideBar = computed(() => router.currentRoute.value.meta.sideBar !== undefined);

async function init() {
  const info = await site.getInfo();
  if (info === undefined) {
    return;
  }
  if (!info.is_installed) {
    router.push("/admin/install");
  }
  await session.updateUser();
}
init();
</script>

<style scoped lang="scss">
// The shell used to be `height: 90vh` here with several views setting `height: 100vh` inside it, so
// content was clipped on short pages and double-scrolled on long ones. The sidebar and the content
// now grow with the page, and `body` owns the scrolling.
.withSidebar {
  display: flex;
  flex: 1;
  align-items: stretch;
  min-height: 0;
}
</style>
