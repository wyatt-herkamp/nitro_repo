<template>
  <div class="datePicker">
    <label :for="id">
      <slot />
    </label>
    <VueDatePicker v-model="value" :id="id" :enable-time-picker="false">
      Received Date
    </VueDatePicker>
  </div>
</template>
<script setup lang="ts">
defineProps<{
  id: string
}>()
let value = defineModel<Date>({
  required: true
})
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
</style>
