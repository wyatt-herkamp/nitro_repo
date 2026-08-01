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
// Optional, so this can back a field that may legitimately be left blank.
const value = defineModel<number | undefined>();
</script>

<style scoped lang="scss">
// This used to also `@import "@/assets/styles/variables.scss"`, which does not exist. The build
// only survived because nothing imported this component; referencing it broke the build.
@import "@/assets/styles/form.scss";
</style>
