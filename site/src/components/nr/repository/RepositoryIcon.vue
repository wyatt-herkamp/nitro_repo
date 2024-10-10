<template>
  <a
    target="_blank"
    rel="noopener noreferrer"
    :title="icon.name"
    :href="icon.url">
    <component :is="icon.component"
  /></a>
</template>
<script setup lang="ts">
import type { RepositoryIconDef } from "@/types/repository";
import { defineProps, type PropType } from "vue";
defineProps({
  icon: {
    type: Object as PropType<RepositoryIconDef>,
    required: true,
  },
});
</script>

<style lang="scss" scoped>
a {
  display: inline-block;
  margin: 0.5rem;
}
</style>
