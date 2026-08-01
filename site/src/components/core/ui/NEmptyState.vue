<template>
  <div class="nEmptyState">
    <font-awesome-icon
      v-if="icon"
      :icon="icon"
      class="icon" />
    <h3>{{ title }}</h3>
    <p v-if="description">{{ description }}</p>
    <div
      v-if="$slots.default"
      class="actions">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * Shown where a list has nothing in it. An empty table with headers and no rows leaves someone
 * wondering whether it failed to load; this says which it is, and offers the action that fixes it.
 */
withDefaults(
  defineProps<{
    title: string;
    description?: string;
    icon?: string | Array<string>;
  }>(),
  { description: undefined, icon: undefined },
);
</script>

<style scoped lang="scss">
.nEmptyState {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-12) var(--space-6);
  text-align: center;

  h3 {
    font-size: var(--text-md);
    font-weight: var(--weight-medium);
  }

  p {
    max-width: 32rem;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }
}

.icon {
  font-size: 1.75rem;
  color: var(--text-subtle);
  margin-bottom: var(--space-2);
}

.actions {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-3);
}
</style>
