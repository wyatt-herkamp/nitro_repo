<template>
  <div class="formBox">
    <label :for="id">
      <slot />
    </label>
    <input
      type="email"
      :id="id"
      v-model="value"
      v-bind="$attrs"
      autocomplete="email" />
  </div>
</template>
<script setup lang="ts">
import "@/assets/styles/form.scss";
defineProps<{
  id: string;
}>();
const value = defineModel<string>({
  required: true,
});
</script>

<style scoped lang="scss"></style>
