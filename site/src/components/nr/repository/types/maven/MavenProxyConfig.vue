<template>
  <ul v-auto-animate class="proxyConfig">
    <li class="proxyRoute" v-for="route in value?.routes" :key="route.url">
      <input v-model="route.url" />
      <input v-model="route.name" />
      <button class="actionButton" @click="removeRoute(route)">Remove</button>
    </li>
    <li class="proxyRoute add">
      <input v-model="newRoute.url" placeholder="https://repo1.maven.org/maven2/" />
      <input v-model="newRoute.name" placeholder="Maven Central" />
      <button class="actionButton" @click="addRoute">Add</button>
    </li>
  </ul>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { type MavenProxyRoute, type MavenProxyConfigType } from './maven'
import { notify } from '@kyvg/vue3-notification'
const newRoute = ref<MavenProxyRoute>({
  url: '',
  name: ''
})

const value = defineModel<MavenProxyConfigType>({
  required: true
})
function removeRoute(route: MavenProxyRoute) {
  value.value.routes = value.value.routes.filter((r) => r !== route)
}
function addRoute() {
  try {
    new URL(newRoute.value.url)
  } catch (e) {
    console.error('Invalid URL', e)
    notify({
      type: 'error',
      title: 'Invalid URL',
      text: 'Please enter a valid URL'
    })
    return
  }

  value.value.routes.push({
    url: newRoute.value.url,
    name: newRoute.value.name
  })
  newRoute.value.url = ''
  newRoute.value.name = ''
}
</script>

<style lang="scss" scoped>
@import '@/assets/styles/theme.scss';
.proxyRoute {
  display: flex;
  margin: 0.5rem;
  input {
    margin-right: 0.5rem;
  }
}
.actionButton {
  margin-left: 0.5rem;
}
.proxyConfig {
  list-style-type: none;
  padding: 0;
}
</style>
