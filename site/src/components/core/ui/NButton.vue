<template>
  <component
    :is="tag"
    v-bind="linkProps"
    :type="tag === 'button' ? type : undefined"
    :disabled="tag === 'button' ? disabled || loading : undefined"
    :aria-busy="loading || undefined"
    :class="['nButton', `is-${variant}`, `is-${size}`, { isBlock: block, isLoading: loading }]">
    <SpinnerElement
      v-if="loading"
      class="spinner" />
    <font-awesome-icon
      v-else-if="icon"
      :icon="icon" />
    <span
      v-if="$slots.default"
      class="labelText"
      ><slot
    /></span>
  </component>
</template>

<script setup lang="ts">
/**
 * The one button.
 *
 * Before this there was only `SubmitButton`, which was always 100% wide and whose `.dangerButton`
 * class was defined inside its own `<style scoped>` block — so it could never be applied by a
 * parent, and the destructive actions it was written for were never able to look destructive.
 *
 * Renders as a `<button>`, a `<RouterLink>` or an `<a>` depending on what it is given, because an
 * action that navigates should be a link: middle-click and "open in new tab" should work.
 */
import { computed } from "vue";
import { RouterLink, type RouteLocationRaw } from "vue-router";
import SpinnerElement from "@/components/spinner/SpinnerElement.vue";

const props = withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md" | "lg";
    type?: "button" | "submit" | "reset";
    disabled?: boolean;
    loading?: boolean;
    block?: boolean;
    icon?: string | Array<string>;
    to?: RouteLocationRaw;
    href?: string;
  }>(),
  {
    variant: "secondary",
    size: "md",
    type: "button",
    disabled: false,
    loading: false,
    block: false,
    icon: undefined,
    to: undefined,
    href: undefined,
  },
);

const tag = computed(() => {
  if (props.disabled || props.loading) return "button";
  if (props.to) return RouterLink;
  if (props.href) return "a";
  return "button";
});

const linkProps = computed(() => {
  if (props.disabled || props.loading) return {};
  if (props.to) return { to: props.to };
  if (props.href) return { href: props.href };
  return {};
});
</script>

<style scoped lang="scss">
.nButton {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  font-family: inherit;
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  line-height: 1;
  white-space: nowrap;
  text-decoration: none;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition:
    background-color var(--duration-fast) var(--ease-out),
    border-color var(--duration-fast) var(--ease-out),
    color var(--duration-fast) var(--ease-out);

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.is-sm {
  padding: 0.3125rem 0.625rem;
  font-size: var(--text-xs);
}
.is-md {
  padding: 0.4688rem 0.875rem;
}
.is-lg {
  padding: 0.625rem 1.125rem;
  font-size: var(--text-base);
}

.isBlock {
  width: 100%;
}

.is-primary {
  background-color: var(--accent);
  color: var(--accent-contrast);

  &:hover:not(:disabled) {
    background-color: var(--accent-hover);
  }
  &:active:not(:disabled) {
    background-color: var(--accent-active);
  }
}

.is-secondary {
  background-color: var(--surface-raised);
  border-color: var(--border-strong);
  color: var(--text);

  &:hover:not(:disabled) {
    background-color: var(--surface-hover);
    border-color: var(--text-subtle);
  }
  &:active:not(:disabled) {
    background-color: var(--surface-active);
  }
}

.is-ghost {
  background-color: transparent;
  color: var(--text-muted);

  &:hover:not(:disabled) {
    background-color: var(--surface-hover);
    color: var(--text);
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

// A loading button keeps its width so the row it sits in does not reflow the moment it is clicked.
.isLoading .labelText {
  opacity: 0.7;
}

.spinner {
  width: 1em;
  height: 1em;
}
</style>
