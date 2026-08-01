<template>
  <main class="logout">
    <SpinnerElement size="lg" />
    <p>Signing out…</p>
  </main>
</template>

<script setup lang="ts">
import { sessionStore } from "@/stores/session";
import SpinnerElement from "@/components/spinner/SpinnerElement.vue";

const session = sessionStore();

async function logout() {
  // A failed logout still has to clear the client's session and land somewhere sensible; leaving
  // someone on a spinner because the server refused the request is the worse outcome.
  try {
    await session.logout();
  } finally {
    window.location.href = "/";
  }
}
logout();
</script>

<style lang="scss" scoped>
.logout {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-4);
  padding: var(--space-16) var(--space-4);
  color: var(--text-muted);
}
</style>
