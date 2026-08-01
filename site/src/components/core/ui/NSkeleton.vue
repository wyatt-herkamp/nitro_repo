<template>
  <span
    class="nSkeleton"
    :style="{ width, height }"
    aria-hidden="true" />
</template>

<script setup lang="ts">
/**
 * A shimmering placeholder block. The existing `SkeletonEntry` had no shimmer at all, so it read as
 * an empty row rather than as loading.
 */
withDefaults(defineProps<{ width?: string; height?: string }>(), {
  width: "100%",
  height: "1rem",
});
</script>

<style scoped lang="scss">
.nSkeleton {
  display: block;
  border-radius: var(--radius-sm);
  background-color: var(--surface-raised);
  background-image: linear-gradient(
    90deg,
    transparent 0%,
    var(--surface-hover) 50%,
    transparent 100%
  );
  background-size: 200% 100%;
  animation: shimmer 1.4s ease-in-out infinite;
}

@keyframes shimmer {
  from {
    background-position: 200% 0;
  }
  to {
    background-position: -200% 0;
  }
}

// A shimmer is decoration; without it the block still reads as a placeholder.
@media (prefers-reduced-motion: reduce) {
  .nSkeleton {
    animation: none;
  }
}
</style>
