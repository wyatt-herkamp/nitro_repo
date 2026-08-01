<template>
  <nav
    class="nBreadcrumb"
    aria-label="Breadcrumb">
    <ol>
      <li
        v-for="(crumb, index) in items"
        :key="index">
        <RouterLink
          v-if="crumb.to && index < items.length - 1"
          :to="crumb.to"
          >{{ crumb.label }}</RouterLink
        >
        <span
          v-else
          :aria-current="index === items.length - 1 ? 'page' : undefined"
          >{{ crumb.label }}</span
        >
        <span
          v-if="index < items.length - 1"
          class="separator"
          aria-hidden="true"
          >/</span
        >
      </li>
    </ol>
  </nav>
</template>

<script setup lang="ts">
import { RouterLink, type RouteLocationRaw } from "vue-router";

export interface Crumb {
  label: string;
  to?: RouteLocationRaw;
}

defineProps<{ items: Array<Crumb> }>();
</script>

<style scoped lang="scss">
// Set as a path rather than as chrome — for a repository browser the trail *is* the file path, and
// it should read like one.
.nBreadcrumb {
  font-family: var(--font-mono);
  font-size: var(--text-sm);

  ol {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    list-style: none;
    padding: 0;
  }

  li {
    display: flex;
    align-items: center;
    min-width: 0;
  }

  a {
    color: var(--text-muted);
    padding: 0.125rem 0.25rem;
    border-radius: var(--radius-sm);

    &:hover {
      color: var(--accent);
      background-color: var(--surface-hover);
    }
  }

  span[aria-current="page"] {
    color: var(--text);
    padding: 0.125rem 0.25rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.separator {
  color: var(--text-subtle);
  user-select: none;
}
</style>
