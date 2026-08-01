<template>
  <div
    class="spinner"
    :class="`is-${size}`"
    role="status"
    :aria-label="label">
    <span class="visually-hidden">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
/**
 * A spinner sized by its container.
 *
 * The previous one was a fixed `10em` square with `margin: 55px auto` baked in, so it could only
 * ever be a full-page loader — putting one inside a button was not possible.
 */
withDefaults(
  defineProps<{
    size?: "inline" | "sm" | "md" | "lg";
    label?: string;
  }>(),
  { size: "inline", label: "Loading" },
);
</script>

<style scoped lang="scss">
.spinner {
  display: inline-block;
  flex-shrink: 0;
  border-radius: 50%;
  border: 2px solid var(--border-strong);
  border-top-color: var(--accent);
  animation: spin 0.7s linear infinite;
}

// Matches the surrounding text, which is what a button or a table cell wants.
.is-inline {
  width: 1em;
  height: 1em;
  border-width: 1.5px;
}
.is-sm {
  width: 1rem;
  height: 1rem;
}
.is-md {
  width: 1.5rem;
  height: 1.5rem;
}
.is-lg {
  width: 2.5rem;
  height: 2.5rem;
  border-width: 3px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .spinner {
    animation-duration: 1.6s;
  }
}
</style>
