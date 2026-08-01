<template>
  <span :class="['nBadge', `is-${variant}`, { isDot: dot }]">
    <span
      v-if="dot"
      class="dot" />
    <slot />
  </span>
</template>

<script setup lang="ts">
/** A small status pill. Semantic rather than decorative — the variant is the meaning. */
withDefaults(
  defineProps<{
    variant?: "neutral" | "accent" | "success" | "warning" | "danger" | "info";
    /** Prefixes a filled dot, for states like active/inactive where the colour is the message. */
    dot?: boolean;
  }>(),
  { variant: "neutral", dot: false },
);
</script>

<style scoped lang="scss">
.nBadge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0.1875rem 0.5rem;
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  line-height: 1.4;
  white-space: nowrap;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
}

.dot {
  width: 0.4375rem;
  height: 0.4375rem;
  border-radius: 50%;
  background-color: currentColor;
  flex-shrink: 0;
}

.is-neutral {
  background-color: var(--surface-raised);
  border-color: var(--border);
  color: var(--text-muted);
}
.is-accent {
  background-color: var(--accent-muted);
  border-color: var(--accent-border);
  color: var(--accent);
}
.is-success {
  background-color: var(--success-muted);
  border-color: var(--success);
  color: var(--success);
}
.is-warning {
  background-color: var(--warning-muted);
  border-color: var(--warning);
  color: var(--warning);
}
.is-danger {
  background-color: var(--danger-muted);
  border-color: var(--danger);
  color: var(--danger);
}
.is-info {
  background-color: var(--info-muted);
  border-color: var(--info);
  color: var(--info);
}
</style>
