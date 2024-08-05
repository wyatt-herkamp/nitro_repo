<template>
  <div>
    <label :for="id">
      <slot />
    </label>
    <input type="password" :id="id" v-model="value" v-bind="$attrs" />
  </div>
</template>
<script setup lang="ts">
import '@/assets/styles/form.scss'
defineProps<{
  id: string
}>()
let value = defineModel<string>({
  required: true
})
</script>

<style scoped lang="scss"></style>
