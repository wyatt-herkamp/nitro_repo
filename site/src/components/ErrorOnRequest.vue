<template>
  <main>
    <h1>Error</h1>
    <div v-if="errorCode === 404">
      <p>{{ humanMessage }} not found</p>
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  error: String,
  errorCode: {
    type: Number,
    required: false
  },
  cause: {
    type: String,
    required: false
  }
})
const humanMessage = computed(() => {
  if (props.cause) {
    return props.cause
  } else {
    return 'Resource'
  }
})
</script>
