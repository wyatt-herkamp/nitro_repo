<template>
  <select :id="id" v-model="value" v-bind="$attrs" :required="required">
    <option v-for="option in values" :key="option.value as string" :value="option.value">
      {{ option.name }}
    </option>
  </select>
</template>
<script setup lang="ts" generic="T">
import { type PropType, ref, watch } from 'vue'
import type { DropDownOption } from '@/components/form/FormTypes'

defineProps({
  id: String,
  values: Array as PropType<DropDownOption<T>[]>,
  required: Boolean
})
let value = defineModel<T>({
  required: true
})
</script>
<style scoped lang="scss">
@import '@/assets/styles/form.scss';
@import '@/assets/styles/variables.scss';
select {
  width: 100%;
  padding: 0.5rem;
  border-radius: 0.5rem;
  border: 1px solid $text-color;
  background-color: $table-background;
  color: $text-color;
  font-size: 1rem;
  &:focus {
    outline: none;
    border: 1px solid $text-color;
  }
}
</style>
