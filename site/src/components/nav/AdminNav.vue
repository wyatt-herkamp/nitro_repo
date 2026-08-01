<template>
  <SideNav>
    <span class="sectionLabel">Administration</span>

    <SideNavElement
      to="/admin"
      routeName="admin">
      <font-awesome-icon icon="gauge-high" />
      <span>Overview</span>
    </SideNavElement>

    <ExpandingSideNav :openIfHasTag="adminUserTag">
      <template #button>
        <!-- `routeName` was "UserList"; the route is named "UsersList", so this item never
             highlighted on the page it links to. -->
        <SideNavElement
          to="/admin/users"
          routeName="UsersList">
          <font-awesome-icon icon="users" />
          <span>Users</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement
          to="/admin/user/create"
          routeName="UserCreate">
          <font-awesome-icon icon="user-plus" />
          <span>Create user</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>

    <ExpandingSideNav :openIfHasTag="adminStorageTag">
      <template #button>
        <SideNavElement
          to="/admin/storages"
          routeName="StorageList">
          <font-awesome-icon icon="database" />
          <span>Storages</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement
          to="/admin/storage/create"
          routeName="StorageCreate">
          <font-awesome-icon icon="plus" />
          <span>Create storage</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>

    <ExpandingSideNav :openIfHasTag="adminRepositoryTag">
      <template #button>
        <SideNavElement
          to="/admin/repositories"
          routeName="RepositoriesList">
          <font-awesome-icon icon="boxes-packing" />
          <span>Repositories</span>
        </SideNavElement>
      </template>
      <template #content>
        <SideNavElement
          to="/admin/repositories/create"
          routeName="RepositoryCreate">
          <font-awesome-icon icon="plus" />
          <span>Create repository</span>
        </SideNavElement>
      </template>
    </ExpandingSideNav>

    <SideNavElement
      to="/admin/system"
      routeName="SystemSettings">
      <font-awesome-icon icon="gear" />
      <span>System</span>
    </SideNavElement>
  </SideNav>
</template>

<script setup lang="ts">
import { type PropType } from "vue";
import type { UserResponseType } from "@/types/base";
import { adminRepositoryTag, adminStorageTag, adminUserTag } from "@/views/admin/adminRoutes";
import SideNav from "./sideNav/SideNav.vue";
import ExpandingSideNav from "./sideNav/ExpandingSideNav.vue";
import SideNavElement from "./sideNav/SideNavElement.vue";

defineProps({
  user: Object as PropType<UserResponseType>,
});
</script>

<style scoped lang="scss">
// This file used to repeat SideNav's and SideNavElement's entire stylesheets, which did nothing
// those components were not already doing.
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
