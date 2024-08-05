<template>
  <header>
    <div class="wrapper">
      <nav>
        <RouterLink to="/">Home</RouterLink>
        <RouterLink to="/about">About</RouterLink>
      </nav>
    </div>
  </header>

  <RouterView />
</template>
<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { siteStore } from './stores/site'
import router from './router'
const site = siteStore()
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
