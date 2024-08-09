<template>
  <nav>
    <router-link to="/" id="logoAndHome" class="navLink">
      <img src="/icon-128.png" alt="Logo" />
      <span>Nitro Repository</span>
    </router-link>
    <RouterLink class="navLink" to="/">
      <font-awesome-icon icon="fa-solid fa-users" />
      <span>Users</span>
    </RouterLink>
    <RouterLink class="navLink" to="/admin/storages">
      <font-awesome-icon icon="fa-solid fa-box-open" />
      <span>Storages</span>
    </RouterLink>
    <RouterLink class="navLink" to="/admin/repositories">
      <font-awesome-icon icon="fa-solid fa-boxes-packing" />
      <span>Repositories</span>
    </RouterLink>
    <RouterLink class="navLink" to="/admin/system">
      <font-awesome-icon icon="fa-solid fa-gear" />
      <span>System</span>
    </RouterLink>
  </nav>
</template>

<script setup lang="ts">
import { sessionStore } from '@/stores/session'
import { RouterLink } from 'vue-router'
const session = sessionStore()
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
nav {
  margin: 0.5rem;
  margin-right: 0.5rem;
  border-top-right-radius: 8px;
  border-bottom-right-radius: 8px;
  width: 200px;
  background-color: $primary-50;
  height: 100%;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.navLink {
  text-decoration: none;
  color: $text;
  font-weight: bold;
  padding: 0.5rem;
  // Align text vertically
  display: flex;
  align-items: center;
  gap: 0.5rem;
  // Box
  border-radius: 0.5rem;
  &:hover {
    background-color: $primary-70;
    transition: background-color 0.3s ease;
  }
}
#logoAndHome {
  img {
    width: 2rem;
    height: 2rem;
  }
  span {
    color: $text;
  }
}
.bottom {
  margin-top: auto;
}
</style>
