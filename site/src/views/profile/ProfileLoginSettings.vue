<template>
  <main v-if="user" class="loginSettings">
    <form
      id="changePassword"
      v-if="site.siteInfo && site.siteInfo.password_rules"
      @submit.prevent="changePassword">
      <h2>Change Password</h2>
      <input id="email" type="hidden" name="email" autocomplete="email" :value="user.email" />
      <input
        id="username"
        type="hidden"
        name="username"
        autocomplete="username"
        :value="user.username" />
      <PasswordInput id="currentPassword" v-model="oldPassword" label="Current Password" />
      <NewPasswordInput
        id="newPassword"
        v-model="newPassword"
        :passwordRules="site.siteInfo.password_rules">
        New Password
      </NewPasswordInput>
      <SubmitButton>Change Password</SubmitButton>
    </form>
  </main>
</template>
<script setup lang="ts">
import SubmitButton from '@/components/form/SubmitButton.vue'
import NewPasswordInput from '@/components/form/text/NewPasswordInput.vue'
import PasswordInput from '@/components/form/text/PasswordInput.vue'
import http from '@/http'
import { sessionStore } from '@/stores/session'
import { siteStore } from '@/stores/site'
import { notify } from '@kyvg/vue3-notification'
import { ref } from 'vue'
const site = siteStore()
const session = sessionStore()
const user = session.user
const oldPassword = ref('')
const newPassword = ref('')

async function changePassword() {
  if (newPassword.value == undefined || newPassword.value == '') {
    return
  }
  const request = {
    old_password: oldPassword.value,
    new_password: newPassword.value
  }
  await http
    .post('/api/user/change-password', request)
    .then(() => {
      oldPassword.value = ''
      newPassword.value = ''
      notify({
        type: 'success',
        text: 'Password changed successfully'
      })
    })
    .catch((error) => {
      console.error(error)
      notify({
        type: 'error',
        text: 'Failed to change password'
      })
    })
}
</script>

<style scoped lang="scss">
#loginSettings {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 90vh;
}
#changePassword {
  padding: 1rem;
  width: 50%;
}
</style>
