<template>
  <section :id="id + '-section'">
    <label :for="id"><slot /></label>
    <input
      type="number"
      :id="id"
      v-model="value"
      v-bind="$attrs" />
  </section>
</template>
<script setup lang="ts">
import "@/assets/styles/form.scss";
defineProps({
  id: String,
});
const value = defineModel<number>({
  required: true,
});
</script>

<style scoped lang="scss">
@import "@/assets/styles/variables.scss";
@import "@/assets/styles/form.scss";
</style>
