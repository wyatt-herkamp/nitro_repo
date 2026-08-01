<template>
  <div class="subBarParent">
    <slot name="button" />
    <div
      class="subBar"
      :data-is-open="isOpen">
      <slot name="content" />
    </div>
  </div>
</template>

<script setup lang="ts">
import router from "@/router";
import { computed } from "vue";

const props = defineProps({
  isOpen: {
    type: Boolean,
    required: false,
  },
  openIfHasTag: {
    type: String,
  },
});

const isOpen = computed(() => {
  if (props.openIfHasTag) {
    return router.currentRoute.value.meta.tag === props.openIfHasTag;
  }
  return props.isOpen ?? false;
});
</script>

<style scoped lang="scss">
.subBar {
  // Indented under its parent and marked with a rule, so a sub-item reads as belonging to the
  // section above rather than as another top-level entry.
  margin-left: var(--space-4);
  padding-left: var(--space-2);
  border-left: 1px solid var(--border);
}

.subBar[data-is-open="false"] {
  display: none;
}

.subBar[data-is-open="true"] {
  display: block;
}

.subBarParent:hover .subBar,
.subBarParent:focus-within .subBar {
  display: block;
}
</style>
