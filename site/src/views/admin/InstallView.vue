<template>
  <main>
    <h1>Install Page</h1>
    <SpinnerElement />
  </main>
</template>
<script setup lang="ts">
import SpinnerElement from '@/components/spinner/SpinnerElement.vue'
import { siteStore } from '@/stores/site'

const site = siteStore()
</script>
