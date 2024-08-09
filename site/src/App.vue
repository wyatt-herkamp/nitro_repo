<template>
  <header>
    <NavBar />
  </header>
  <div class="admin-content" v-if="isAdminPage && !isInstallPage">
    <AdminNav />
    <RouterView />
  </div>
  <RouterView v-else />
</template>
<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { siteStore } from './stores/site'
import router from './router'
import NavBar from './components/nav/NavBar.vue'
import { sessionStore } from './stores/session'
import { computed } from 'vue'
import AdminNav from './components/nav/AdminNav.vue'
const site = siteStore()
const session = sessionStore()
const isAdminPage = computed(() => {
  return router.currentRoute.value.path.startsWith('/admin')
})
const isInstallPage = computed(() => {
  return router.currentRoute.value.name === 'AdminInstall'
})
async function init() {
  const info = await site.getInfo()
  if (info == undefined) {
    console.log('info is undefined')
    return
  }
  console.log(info)

  if (!info?.is_installed) {
    router.push('/admin/install')
  }
}
init()
</script>
<style scoped>
.admin-content {
  display: flex;
  height: 100vh;
}
</style>
