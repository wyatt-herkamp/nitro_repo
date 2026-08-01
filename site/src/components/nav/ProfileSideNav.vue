<template>
  <SideNav>
    <span class="sectionLabel">Your account</span>

    <SideNavElement
      to="/profile"
      routeName="profile">
      <font-awesome-icon icon="user" />
      <span>Profile</span>
    </SideNavElement>

    <SideNavElement
      to="/profile/login"
      routeName="profileLogin">
      <font-awesome-icon icon="right-to-bracket" />
      <span>Sign-in settings</span>
    </SideNavElement>

    <ExpandingSideNav :openIfHasTag="profileTokenTag">
      <template #button>
        <SideNavElement
          to="/profile/tokens"
          routeName="profileTokens">
          <font-awesome-icon icon="key" />
          <span>API tokens</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement
          to="/profile/token/create"
          routeName="profileTokenCreate">
          <font-awesome-icon icon="plus" />
          <span>Create token</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>
  </SideNav>
</template>

<script setup lang="ts">
// Every item here used to draw `fa-gear`, so the icons carried no information at all.
import { profileTokenTag } from "@/views/profile/profileRoutes";
import ExpandingSideNav from "./sideNav/ExpandingSideNav.vue";
import SideNav from "./sideNav/SideNav.vue";
import SideNavElement from "./sideNav/SideNavElement.vue";
</script>

<style scoped lang="scss">
.sectionLabel {
  padding: var(--space-1) var(--space-3) var(--space-2);
  font-size: var(--text-2xs);
  font-weight: var(--weight-semibold);
  letter-spacing: var(--tracking-label);
  text-transform: uppercase;
  color: var(--text-subtle);
}

@media (max-width: 48rem) {
  .sectionLabel {
    display: none;
  }
}
</style>
