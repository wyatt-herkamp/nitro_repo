<template>
  <RouterLink
    :to="to"
    :data-active="isActive"
    :aria-current="isActive ? 'page' : undefined"
    class="navLink">
    <slot />
  </RouterLink>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";

const props = defineProps({
  to: {
    type: String,
    required: true,
  },
  routeName: {
    type: String,
    required: false,
  },
});

const router = useRouter();
const isActive = computed(() => props.routeName === router.currentRoute.value.name);
</script>

<style scoped lang="scss">
.navLink {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-muted);
  border-radius: var(--radius-md);
  transition:
    color var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);

  &:hover {
    color: var(--text);
    background-color: var(--surface-hover);
  }

  // A fixed width so the labels line up whatever glyph each item uses.
  :deep(svg) {
    width: 1rem;
    flex-shrink: 0;
  }
}

.navLink[data-active="true"] {
  color: var(--accent);
  background-color: var(--accent-muted);
  cursor: default;
}
</style>
