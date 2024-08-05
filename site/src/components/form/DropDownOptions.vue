<template>
  <section :id="id + '-section'">
    <label :for="id"><slot /></label>
    <DropDownOptionsInner
      :id="id"
      v-model="value"
      :values="props.values"
      v-bind="$attrs"
      :required="required"
    />
  </section>
</template>
<script setup lang="ts" generic="T">
import { type PropType, ref, watch } from 'vue'
import type { DropDownOption } from '@/components/form/FormTypes'
import DropDownOptionsInner from '@/components/form/DropDownOptionsInner.vue'

const props = defineProps({
  id: String,
  values: Array as PropType<DropDownOption<T>[]>,
  required: Boolean
})

let value = defineModel<T>({
  required: true
})
</script>
<style scoped lang="scss">
section {
  label {
    margin-bottom: 1rem;
  }
  select {
    margin-top: 0.5rem;
  }
}
</style>
