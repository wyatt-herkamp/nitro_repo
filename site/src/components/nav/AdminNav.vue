<template>
  <nav>
    <router-link to="/" id="logoAndHome" class="navLink">
      <img src="/icon-128.png" alt="Logo" />
      <span>Nitro Repository</span>
    </router-link>
    <AdminNavSub :isOpen="isOnUsersPage">
      <template #button>
        <RouterLink
          class="navLink"
          :data-active="isActive('UsersList')"
          :to="{
            name: 'UsersList'
          }">
          <font-awesome-icon icon="fa-solid fa-users" />
          <span>Users</span>
        </RouterLink>
      </template>
      <template #content>
        <RouterLink
          class="navLink"
          :to="{ name: 'UserCreate' }"
          :data-active="isActive('UserCreate')">
          <font-awesome-icon icon="fa-solid fa-user-plus" />
          <span>Create User</span>
        </RouterLink>
      </template>
    </AdminNavSub>
    <AdminNavSub :isOpen="isOnStoragesPage">
      <template #button>
        <RouterLink
          class="navLink"
          :data-active="isActive('StorageList')"
          :to="{
            name: 'StorageList'
          }">
          <font-awesome-icon icon="fa-solid fa-box-open" />
          <span>Storages</span>
        </RouterLink>
      </template>
      <template #content>
        <RouterLink
          class="navLink"
          :to="{ name: 'StorageCreate' }"
          :data-active="isActive('StorageCreate')">
          <font-awesome-icon icon="fa-solid fa-box-open" />
          <span>Create Storage</span>
        </RouterLink>
      </template>
    </AdminNavSub>
    <AdminNavSub :isOpen="isOnRepositoriesPage">
      <template #button>
        <RouterLink
          class="navLink"
          to="/admin/repositories"
          :data-active="isActive('RepositoriesList')">
          <font-awesome-icon icon="fa-solid fa-boxes-packing" />
          <span>Repositories</span>
        </RouterLink>
      </template>
      <template #content>
        <RouterLink
          class="navLink"
          :to="{ name: 'RepositoryCreate' }"
          :data-active="isActive('RepositoryCreate')">
          <font-awesome-icon icon="fa-solid fa-boxes-packing" />
          <span>Create Repository</span>
        </RouterLink>
      </template>
    </AdminNavSub>

    <RouterLink class="navLink" to="/admin/system">
      <font-awesome-icon icon="fa-solid fa-gear" />
      <span>System</span>
    </RouterLink>
  </nav>
</template>

<script setup lang="ts">
import { sessionStore } from '@/stores/session'
import { RouterLink } from 'vue-router'
import AdminNavSub from './AdminNavSub.vue'
import { computed, type PropType } from 'vue'
import router from '@/router'
import type { UserResponseType } from '@/types/base'
const props = defineProps({
  user: Object as PropType<UserResponseType>
})
const isOnUsersPage = computed(() => {
  const name = router.currentRoute.value.path
  return name.startsWith('/admin/users') || name.startsWith('/admin/user')
})
const isOnStoragesPage = computed(() => {
  const name = router.currentRoute.value.path
  return name.startsWith('/admin/storages') || name.startsWith('/admin/storage')
})
const isOnRepositoriesPage = computed(() => {
  const name = router.currentRoute.value.path
  return name.startsWith('/admin/repositories') || name.startsWith('/admin/repository')
})
const activeLink = computed(() => {
  return router.currentRoute.value.name
})

function isActive(name: string) {
  return router.currentRoute.value.name === name
}
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
.navLink[data-active='true'] {
  background-color: $primary-70;
  &:hover {
    cursor: default;
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
