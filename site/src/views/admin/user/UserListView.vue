<template>
  <main class="container">
    <div class="page-header">
      <div class="page-header-text">
        <h1>Users</h1>
        <p>Accounts that can sign in to this instance.</p>
      </div>
      <div class="page-header-actions">
        <NButton
          variant="primary"
          icon="user-plus"
          :to="{ name: 'UserCreate' }">
          New user
        </NButton>
      </div>
    </div>

    <UserTable
      :users="users"
      :loading="loading"
      :error="error" />
  </main>
</template>

<script setup lang="ts">
import { ref } from "vue";
import UserTable from "@/components/admin/user/UserTable.vue";
import NButton from "@/components/core/ui/NButton.vue";
import http from "@/http";
import type { UserResponseType } from "@/types/base";

const users = ref<Array<UserResponseType>>([]);
const error = ref<string | undefined>(undefined);
const loading = ref(true);

async function load() {
  try {
    const response = await http.get<Array<UserResponseType>>("/api/user-management/list");
    users.value = response.data;
  } catch {
    error.value = "Failed to load users.";
  } finally {
    loading.value = false;
  }
}
load();
</script>
