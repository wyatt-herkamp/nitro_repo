<template>
  <div>
    <label for="dropdown"><slot></slot></label>
    <select id="dropdown" v-model="value">
      <option v-for="option in props.options" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import type { PropType } from 'vue'
import '@/assets/styles/form.scss'

const props = defineProps({
  options: {
    type: Array as PropType<{ label: string; value: string }[]>,
    required: true
  }
})
const value = defineModel<string>({
  required: true
})
</script>
<style lang="scss" scoped>
@import '@/assets/styles/theme.scss';
div {
  display: flex;
  flex-direction: column;
  margin-bottom: 1rem;
}
</style>
