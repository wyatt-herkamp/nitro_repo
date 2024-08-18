<template>
  <main v-if="user">
    <AdminUserPage :user="user" />
  </main>
  <ErrorOnRequest v-else-if="error" :error="error" :errorCode="errorCode" />
</template>

<script setup lang="ts">
import AdminUserPage from '@/components/admin/user/AdminUserPage.vue'
import ErrorOnRequest from '@/components/ErrorOnRequest.vue'
import http from '@/http'
import router from '@/router'
import type { User } from '@/types/base'
import { computed, ref } from 'vue'
const userId = router.currentRoute.value.params.id as string
const user = ref<User | undefined>(undefined)
const error = ref<string | null>(null)
const errorCode = ref<number | undefined>(undefined)
async function fetchUser() {
  await http
    .get<User>(`/api/user-management/get/${userId}`)
    .then((response) => {
      user.value = response.data
    })
    .catch((error) => {
      console.error(error)
      errorCode.value = error.response.status
      error.value = 'Failed to fetch user'
    })
}
fetchUser()
</script>
