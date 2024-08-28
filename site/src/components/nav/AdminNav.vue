<template>
  <SideNav>
    <ExpandingSideNav :openIfHasTag="adminUserTag">
      <template #button>
        <SideNavElement to="/admin/users" routeName="UserList">
          <font-awesome-icon icon="fa-solid fa-users" />
          <span>Users</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement to="/admin/user/create" routeName="UserCreate">
          <font-awesome-icon icon="fa-solid fa-user-plus" />
          <span>Create User</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>
    <ExpandingSideNav :openIfHasTag="adminStorageTag">
      <template #button>
        <SideNavElement to="/admin/storages" routeName="StorageList">
          <font-awesome-icon icon="fa-solid fa-box-open" />
          <span>Storages</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement to="/admin/storage/create" routeName="StorageCreate">
          <font-awesome-icon icon="fa-solid fa-box-open" />
          <span>Create Storage</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>
    <ExpandingSideNav :openIfHasTag="adminRepositoryTag">
      <template #button>
        <SideNavElement to="/admin/repositories" routeName="RepositoriesList">
          <font-awesome-icon icon="fa-solid fa-boxes-packing" />
          <span>Repositories</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement to="/admin/repositories/create" routeName="RepositoryCreate">
          <font-awesome-icon icon="fa-solid fa-boxes-packing" />
          <span>Create Repository</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>

    <SideNavElement to="/admin/system" routeName="SystemSettings">
      <font-awesome-icon icon="fa-solid fa-gear" />
      <span>System</span>
    </SideNavElement>
  </SideNav>
</template>

<script setup lang="ts">
import { sessionStore } from '@/stores/session'
import { RouterLink } from 'vue-router'
import { computed, type PropType } from 'vue'
import router from '@/router'
import type { UserResponseType } from '@/types/base'
import { adminRepositoryTag, adminStorageTag, adminUserTag } from '@/views/admin/adminRoutes'
import SideNav from './sideNav/SideNav.vue'
import ExpandingSideNav from './sideNav/ExpandingSideNav.vue'
import SideNavElement from './sideNav/SideNavElement.vue'
const props = defineProps({
  user: Object as PropType<UserResponseType>
})

const activeLink = computed(() => {
  return router.currentRoute.value.name
})
function hasTag(tag: string) {
  return router.currentRoute.value.meta.tag === tag
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
