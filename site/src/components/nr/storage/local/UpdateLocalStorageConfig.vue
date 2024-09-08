<template>
  <div>
    <TextInput v-model="model.path" required autocomplete="none" spellcheck="false" disabled>
      Path
    </TextInput>
  </div>
</template>

<script setup lang="ts">
import TextInput from '@/components/form/text/TextInput.vue'

const model = defineModel<any>()
console.log(model)
</script>
