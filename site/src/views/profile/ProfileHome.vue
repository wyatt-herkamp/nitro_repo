<template>
  <main v-if="user">
    <div id="userProfile">
      <h1>Welcome, {{ user.name }}</h1>
      <KeyAndValue
        label="Email"
        :value="user.email" />
      <KeyAndValue
        label="Username"
        :value="user.username" />
    </div>
  </main>
</template>

<script setup lang="ts">
import KeyAndValue from "@/components/form/KeyAndValue.vue";
import { sessionStore } from "@/stores/session";

const session = sessionStore();
const user = session.user;
</script>

<style scoped lang="scss">
#userProfile {
  padding: 1rem;
  width: 50%;
}
</style>
