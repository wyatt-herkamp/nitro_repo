<template>
  <div v-if="!error && users.length >= 1">
    <UserListInner :users="users" />
  </div>
</template>

<script setup lang="ts">
import http from '@/http'
import type { User } from '@/types/base'
import { ref } from 'vue'
import UserListInner from './UserListInner.vue'

const users = ref<User[]>([])
const error = ref<string | null>(null)

async function fetchUsers() {
  await http
    .get<User[]>('/api/user-management/list')
    .then((response) => {
      users.value = response.data
    })
    .catch((error) => {
      console.error(error)
      error.value = 'Failed to fetch users'
    })
}
fetchUsers()
</script>
