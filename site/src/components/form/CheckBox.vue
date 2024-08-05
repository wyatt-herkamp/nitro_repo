<template>
  <div class="CheckBox">
    <label :for="id" class="Label">
      <slot />
    </label>
    <input type="checkbox" :id="id" v-model="checkboxOne" />
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { CheckboxIndicator, CheckboxRoot } from 'radix-vue'
import { Icon } from '@iconify/vue'
const props = defineProps<{
  id: string
}>()
const checkboxOne = defineModel<boolean>({
  required: true
})
</script>
<style scoped lang="scss">
@import '@/assets/styles/variables';

.CheckBox {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin: 0.5rem 2rem;

  // Line up the checkbox with the label
  input {
    margin: 0;
  }

  .Label {
    margin: auto 0;
    padding-bottom: 0;
  }

  label {
    // Disable copy

    line-height: 1;
    // Fit the label to one line
    white-space: nowrap;
  }
}
</style>
