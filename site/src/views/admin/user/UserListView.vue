<template>
  <main v-if="!error && users.length >= 1">
    <UserList :users="users" />
  </main>
  <ErrorOnRequest
    v-else-if="error"
    :error="error" />
</template>

<script setup lang="ts">
import UserList from "@/components/admin/user/UserList.vue";
import ErrorOnRequest from "@/components/ErrorOnRequest.vue";
import http from "@/http";
import type { UserResponseType } from "@/types/base";
import { ref } from "vue";

const users = ref<UserResponseType[]>([]);
const error = ref<string | null>(null);

async function fetchUsers() {
  await http
    .get<UserResponseType[]>("/api/user-management/list")
    .then((response) => {
      users.value = response.data;
    })
    .catch((error) => {
      console.error(error);
      error.value = "Failed to fetch users";
    });
}
fetchUsers();
</script>
