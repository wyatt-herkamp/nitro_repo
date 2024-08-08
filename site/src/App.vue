<template>
  <header>
    <NavBar />
  </header>

  <RouterView />
</template>
<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { siteStore } from './stores/site'
import router from './router'
import NavBar from './components/nav/NavBar.vue'
import { sessionStore } from './stores/session'
const site = siteStore()
const session = sessionStore()
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
<style scoped></style>
