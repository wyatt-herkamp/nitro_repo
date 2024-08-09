<template>
  <div class="dropDown">
    <button class="dropDown-button">
      <font-awesome-icon icon="fa-solid fa-user" />
      <span>User</span>
    </button>
    <div class="dropDown-content">
      <RouterLink class="navLink right" v-if="session.user" :to="{ name: 'profile' }"
        >Profile</RouterLink
      >
      <RouterLink class="navLink right" v-if="session.isAdmin()" :to="{ name: 'admin' }"
        >Admin</RouterLink
      >
      <RouterLink
        v-if="session.user"
        class="navLink right"
        :to="{
          name: 'logout'
        }"
        >Logout</RouterLink
      >
    </div>
  </div>
</template>

<script setup lang="ts">
import { sessionStore } from '@/stores/session'

let session = sessionStore()
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
.dropDown-button {
  background-color: $primary-50;
  border-radius: 0.5rem;
  color: $text;
  padding: 16px;
  font-size: 16px;
  border: none;
  cursor: pointer;
  &:hover {
    background-color: $primary-70;
    transition: background-color 0.3s ease;
  }
}

.dropDown {
  position: relative;
  display: inline-block;
}

.dropDown-content {
  display: none;
  position: absolute;
  background-color: $primary-50;
  min-width: 160px;
  box-shadow: 0px 8px 16px 0px rgba(0, 0, 0, 0.2);
  z-index: 1;
  a {
    color: $text;
    padding: 12px 16px;
    text-decoration: none;
    display: block;
    &:hover {
      background-color: $secondary;
    }
  }
}

.dropDown:hover .dropDown-content {
  display: block;
}
</style>
