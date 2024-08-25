<template>
  <main>
    <h1>User Create</h1>
    <p>Create a new user here.</p>
    <p v-if="createError">{{ createError }}</p>
    <form @submit.prevent="create">
      <TextInput v-model="user.name"> Name</TextInput>
      <NewEmailInput v-model="user.email"> Email</NewEmailInput>
      <UsernameInput v-model="user.username"> Username</UsernameInput>
      <SwitchInput id="setPassword" v-model="setPassword">Set Password</SwitchInput>
      <div v-if="setPassword">
        <NewPasswordInput
          v-if="passwordRules"
          id="password"
          :passwordRules="passwordRules"
          v-model="password" />
      </div>
      <SubmitButton>Create User</SubmitButton>
    </form>
  </main>
</template>
<script lang="ts" setup>
import SubmitButton from '@/components/form/SubmitButton.vue'
import SwitchInput from '@/components/form/SwitchInput.vue'
import NewEmailInput from '@/components/form/text/NewEmailInput.vue'
import NewPasswordInput from '@/components/form/text/NewPasswordInput.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import UsernameInput from '@/components/form/text/UsernameInput.vue'
import http from '@/http'
import router from '@/router'
import { siteStore } from '@/stores/site'
import type { PasswordRules } from '@/types/base'
import { computed, watch, type Ref } from 'vue'
import { ref } from 'vue'
const createError = ref<string | null>(null)
const user = ref({
  name: '',
  email: '',
  username: ''
})
const passwordRules = ref<PasswordRules | undefined>(undefined)
siteStore()
  .getInfo()
  .then((siteInfo) => {
    if (!siteInfo) {
      return
    }
    passwordRules.value = siteInfo.password_rules
  })

const setPassword = ref(false)

const password: Ref<string | undefined> = ref(undefined)

async function create() {
  if (setPassword.value && !password.value) {
    createError.value = 'Password is required'
    return
  }
  console.log('Creating user')
  const requestBody = {
    name: user.value.name,
    email: user.value.email,
    username: user.value.username,
    password: password.value
  }
  console.log(requestBody)

  await http
    .post('/api/user-management/create', requestBody)
    .then(() => {
      router.push('/admin/users')
    })
    .catch((error) => {
      createError.value = error.response.data
    })
}
</script>
