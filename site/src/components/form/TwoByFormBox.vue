<template>
  <div class="twoBy">
    <slot />
  </div>
</template>
<script setup lang="ts"></script>

<style scoped lang="scss">
.twoBy {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  padding-top: 1rem;
}
</style>
