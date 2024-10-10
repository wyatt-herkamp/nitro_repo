<template>
  <main>
    <SpinnerElement />
  </main>
</template>
<script setup lang="ts">
import { sessionStore } from "@/stores/session";
import SpinnerElement from "@/components//spinner/SpinnerElement.vue";
const session = sessionStore();
async function logout() {
  // Pause for 1 second to allow the spinner to show
  await new Promise((resolve) => setTimeout(resolve, 1000));
  session
    .logout()
    .then(() => {
      window.location.href = "/";
    })
    .catch((error) => {
      console.log(error);
    });
}
logout();
</script>

<style lang="scss" scoped>
main {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
}
</style>
