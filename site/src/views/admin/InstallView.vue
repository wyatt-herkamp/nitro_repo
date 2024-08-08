<template>
  <main>
    <h1>Install Page</h1>
    <form @submit.prevent="install">
      <TextInput
        id="username"
        v-model="input.username"
        autocomplete="username"
        required
        placeholder="admin"
        >Username</TextInput
      >
      <TextInput
        id="name"
        v-model="input.name"
        autocomplete="name"
        required
        placeholder="Admin User"
        >Name</TextInput
      >

      <EmailInput id="email" v-model="input.email" placeholder="admin@nitro-repo.dev" required
        >Email</EmailInput
      >
      <TwoByFormBox>
        <PasswordInput id="password" v-model="input.password" required :newPassword="true"
          >Password</PasswordInput
        >
        <PasswordInput
          id="confirmPassword"
          v-model="input.confirmPassword"
          required
          :newPassword="true"
          >Confirm Password</PasswordInput
        >
      </TwoByFormBox>
      <SubmitButton :disabled="formValid != ''" :title="installButtonTitle()">Install</SubmitButton>
    </form>
  </main>
</template>
<script setup lang="ts">
import SubmitButton from '@/components/form/SubmitButton.vue'
import EmailInput from '@/components/form/text/EmailInput.vue'
import PasswordInput from '@/components/form/text/PasswordInput.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import TwoByFormBox from '@/components/form/TwoByFormBox.vue'
import http from '@/http'
import { siteStore } from '@/stores/site'
import { notify } from '@kyvg/vue3-notification'
import { computed, ref } from 'vue'
const input = ref({
  username: '',
  email: '',
  name: '',
  password: '',
  confirmPassword: ''
})
function installButtonTitle() {
  return formValid.value === '' ? 'Install' : formValid.value
}
const formValid = computed(() => {
  if (input.value.username === '') {
    return 'Username is required.'
  }
  if (input.value.email === '') {
    return 'Email is required.'
  }
  if (input.value.name === '') {
    return 'Name is required.'
  }
  if (input.value.password === '') {
    return 'Password is required.'
  }
  if (input.value.password !== input.value.confirmPassword) {
    return 'Passwords do not match.'
  }
  return ''
})
const site = siteStore()
async function install() {
  const newUser = {
    username: input.value.username,
    email: input.value.email,
    name: input.value.name,
    password: input.value.password
  }
  const install = {
    user: newUser
  }
  await http
    .post('/api/install', install)
    .then((response) => {
      if (response.status === 204) {
        // Refresh and redirect to login
      }
    })
    .catch((error) => {
      console.error(error)
      notify({
        type: 'error',
        title: 'Error',
        text: 'An error occurred while installing the application.'
      })
    })
}
</script>
<style lang="scss" scoped>
main {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
}
form {
}
</style>
