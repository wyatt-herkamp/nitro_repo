<template>
  <main class="container">
    <NEmptyState
      title="Page not found"
      description="That page does not exist. It may have moved, or the link may be wrong."
      icon="triangle-exclamation">
      <NButton
        variant="primary"
        :to="{ name: 'home' }"
        >Go to repositories</NButton
      >
      <NButton :to="{ name: 'search' }">Search</NButton>
    </NEmptyState>
  </main>
</template>

<script setup lang="ts">
import NEmptyState from "@/components/core/ui/NEmptyState.vue";
import NButton from "@/components/core/ui/NButton.vue";
</script>
