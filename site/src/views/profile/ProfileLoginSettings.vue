<template>
  <main v-if="user" class="loginSettings">
    <form
      id="changePassword"
      v-if="site.siteInfo && site.siteInfo.password_rules"
      @submit.prevent="changePassword">
      <h2>Change Password</h2>
      <PasswordInput v-model="oldPassword" label="Current Password" />
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
import WorkInProgress from '@/components/core/WorkInProgress.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
import NewPasswordInput from '@/components/form/text/NewPasswordInput.vue'
import PasswordInput from '@/components/form/text/PasswordInput.vue'
import { sessionStore } from '@/stores/session'
import { siteStore } from '@/stores/site'
import { ref } from 'vue'
const site = siteStore()
const passwordRules = site.siteInfo?.password_rules
const session = sessionStore()
const user = session.user
const oldPassword = ref('')
const newPassword = ref('')

async function changePassword() {
  if (!passwordRules) {
    return
  }
  if (newPassword.value == undefined || newPassword.value == '') {
    return
  }
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
