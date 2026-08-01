<template>
  <button
    type="submit"
    :class="['submitButton', `is-${variant}`]"
    v-bind="$attrs"
    @click="(event: MouseEvent) => emit('click', event)">
    <slot />
  </button>
</template>

<script setup lang="ts">
/**
 * The submit button used by the existing forms.
 *
 * `NButton` is the one to reach for in new code; this stays for the forms already built on it, and
 * now takes a `variant` — the old `.dangerButton` class was defined inside this component's own
 * `<style scoped>` block, so a parent could never apply it and a destructive submit could never
 * look destructive.
 */
withDefaults(defineProps<{ variant?: "primary" | "danger" }>(), { variant: "primary" });

const emit = defineEmits<{
  (e: "click", event: MouseEvent): void;
}>();
</script>

<style scoped lang="scss">
.submitButton {
  width: 100%;
  padding: 0.5rem 0.875rem;
  font-family: inherit;
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  line-height: 1.25;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color var(--duration-fast) var(--ease-out);

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.is-primary {
  background-color: var(--accent);
  color: var(--accent-contrast);

  &:hover:not(:disabled) {
    background-color: var(--accent-hover);
  }
}

.is-danger {
  background-color: transparent;
  border-color: var(--danger);
  color: var(--danger);

  &:hover:not(:disabled) {
    background-color: var(--danger);
    color: var(--danger-contrast);
  }
}
</style>
