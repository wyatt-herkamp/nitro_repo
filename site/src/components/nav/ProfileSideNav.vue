<template>
  <SideNav>
    <SideNavElement to="/profile" routeName="profile">
      <font-awesome-icon icon="fa-solid fa-gear" />
      <span>Profile</span>
    </SideNavElement>
    <SideNavElement to="/profile/login" routeName="profileLogin">
      <font-awesome-icon icon="fa-solid fa-gear" />
      <span>Login</span>
    </SideNavElement>
    <ExpandingSideNav :openIfHasTag="profileTokenTag">
      <template #button>
        <SideNavElement to="/profile/tokens" routeName="profileTokens">
          <font-awesome-icon icon="fa-solid fa-gear" />
          <span>Tokens</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement to="/profile/token/create" routeName="profileTokenCreate">
          <font-awesome-icon icon="fa-solid fa-box-open" />
          <span>Create Token</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>
  </SideNav>
</template>
<script setup lang="ts">
import { profileTokenTag } from '@/views/profile/profileRoutes'
import ExpandingSideNav from './sideNav/ExpandingSideNav.vue'
import SideNav from './sideNav/SideNav.vue'
import SideNavElement from './sideNav/SideNavElement.vue'
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
</style>
