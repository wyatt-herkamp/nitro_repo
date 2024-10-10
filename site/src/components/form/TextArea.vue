<template>
  <div>
    <label :for="id"><slot /></label>
    <textarea
      :id="id"
      v-model="value"
      v-bind="$attrs" />
  </div>
</template>
<script setup lang="ts">
import "@/assets/styles/form.scss";
defineProps<{
  id: string;
}>();
const value = defineModel<string | undefined>({
  required: true,
});
</script>

<style scoped lang="scss">
@import "@/assets/styles/variables";

label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: bold;
}

textarea {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid $table-odd-row;
  border-radius: 0.25rem;
  box-sizing: border-box;
  font-size: 1rem;
  line-height: 1.5rem;
  resize: vertical;
  background-color: $background-color;
  color: $text-color;
}
</style>
