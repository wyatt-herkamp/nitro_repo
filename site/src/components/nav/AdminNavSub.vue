<template>
  <div class="subBarParent">
    <slot name="button"></slot>
    <div class="subBar" :data-is-open="isOpen">
      <slot name="content"></slot>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, defineProps } from 'vue'
const props = defineProps({
  isOpen: Boolean
})
</script>
<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
.subBarParent {
  .subBar {
    padding-left: 1rem;
  }
}
.subBar[data-is-open='false'] {
  display: none;
}

.subBar[data-is-open='true'] {
  display: block;
}
.subBarParent:hover .subBar {
  display: block;
}
</style>
