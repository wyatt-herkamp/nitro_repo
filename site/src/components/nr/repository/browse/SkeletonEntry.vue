<template>
  <div
    class="browseItem"
    aria-hidden="true">
    <span class="itemAndName">
      <NSkeleton
        width="1rem"
        height="1rem" />
      <NSkeleton
        :width="nameWidth"
        height="0.75rem" />
    </span>
    <span class="itemMeta"
      ><NSkeleton
        width="3rem"
        height="0.75rem"
    /></span>
    <span class="itemMeta itemModified"
      ><NSkeleton
        width="5rem"
        height="0.75rem"
    /></span>
  </div>
</template>

<script setup lang="ts">
/**
 * A placeholder row, shown while the browse websocket is still streaming entries.
 *
 * It used to be a folder icon on its own with no shimmer, which read as a real (empty) folder row
 * rather than as something still loading. Widths vary a little per row so the block looks like a
 * list of names rather than a stack of identical bars.
 */
import { computed } from "vue";
import NSkeleton from "@/components/core/ui/NSkeleton.vue";
import "./browse.scss";

const props = defineProps({
  index: {
    type: Number,
    default: 0,
  },
});

const WIDTHS = ["12rem", "8rem", "15rem", "10rem", "6rem"];
const nameWidth = computed(() => WIDTHS[props.index % WIDTHS.length]);
</script>
