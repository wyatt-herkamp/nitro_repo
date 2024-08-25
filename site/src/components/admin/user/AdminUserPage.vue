<template>
  <div v-if="user" class="tabs">
    <div class="tabs-header">
      <div class="tab" :data-active="currentTab === 'main'" @click="currentTab = 'main'">User</div>
      <div class="tab" :data-active="currentTab === 'password'" @click="currentTab = 'password'">
        Password
      </div>
      <div
        class="tab"
        :data-active="currentTab === 'user-permissions'"
        @click="currentTab = 'user-permissions'">
        User Permissions
      </div>
      <div
        class="tab"
        :data-active="currentTab === 'repository-permissions'"
        @click="currentTab = 'repository-permissions'">
        Repository Permissions
      </div>
    </div>
    <div class="tabs-content">
      <div class="tab-content" :data-active="currentTab === 'main'">
        <div id="userMain">
          <form>
            <TextInput v-model="changeUser.name"> Name</TextInput>
            <NewEmailInput v-model="changeUser.email" :original-value="user.email">
              Email</NewEmailInput
            >
            <UsernameInput v-model="changeUser.username" :original-value="user.username">
              Username</UsernameInput
            >
            <SubmitButton>Save</SubmitButton>
          </form>
          <div>
            <KeyAndValue :label="'ID #'" :value="user.id.toLocaleString()" />
            <KeyAndValue
              :label="'Created At'"
              :value="new Date(user.created_at).toLocaleString()" />
          </div>
        </div>
      </div>
      <div class="tab-content" :data-active="currentTab === 'password'">
        <form id="setPassword" @submit.prevent="changePassword">
          <NewPasswordInput
            id="password"
            v-if="passwordRules"
            v-model="newPassword"
            :passwordRules="passwordRules">
            Password</NewPasswordInput
          >
          <SubmitButton>Save</SubmitButton>
        </form>
      </div>
      <div class="tab-content" :data-active="currentTab === 'user-permissions'">
        <UserPermissions :user="user" />
      </div>
      <div class="tab-content" :data-active="currentTab === 'repository-permissions'">
        <RepositoryPermissions :user="user" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import KeyAndValue from '@/components/form/KeyAndValue.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
import EmailInput from '@/components/form/text/EmailInput.vue'
import NewEmailInput from '@/components/form/text/NewEmailInput.vue'
import NewPasswordInput from '@/components/form/text/NewPasswordInput.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import UsernameInput from '@/components/form/text/UsernameInput.vue'
import { siteStore } from '@/stores/site'
import type { UserResponseType } from '@/types/base'
import { ref, type PropType } from 'vue'
import UserPermissions from './UserPermissions.vue'
import RepositoryPermissions from './RepositoryPermissions.vue'
import http from '@/http'
import { notify } from '@kyvg/vue3-notification'
const props = defineProps({
  user: {
    type: Object as PropType<UserResponseType>,
    required: true
  }
})
const currentTab = ref('main')
const changeUser = ref({
  name: props.user.name,
  email: props.user.email,
  username: props.user.username
})
const newPassword = ref('')

const passwordRules = siteStore().siteInfo?.password_rules
async function changePassword() {
  console.log('Changing Password')

  if (!newPassword.value) {
    console.log('Password is required')
    return
  }

  console.log('Password is valid')

  http
    .put(`/api/user-management/update/${props.user.id}/password`, {
      password: newPassword.value
    })
    .then(() => {
      notify({
        type: 'success',
        title: 'Password Changed',
        text: 'Password has been changed'
      })
      newPassword.value = ''
      console.log('Password Changed')
    })
    .catch((error) => {
      console.error(error)
    })
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme';
.tabs {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 90vh;

  background-color: $background-30;
}
@media screen and (max-width: 800px) {
  .tabs-header {
    flex-direction: column;
  }
}
.tabs-header {
  display: flex;
  gap: 1rem;
  width: 100%;
  background-color: $primary-30;
}
.tab {
  padding: 1rem;
  cursor: pointer;
  border-radius: 0.5rem 0.5rem 0 0;
  border: 1px solid $primary-50;
  &:hover {
    background-color: $accent;
    color: white;
  }
}
.tab[data-active='true'] {
  background-color: $accent;
  color: white;
  cursor: default;
}
.tab-content {
  display: flex;
  width: 100%;
  height: 100%;
  margin: auto 0;
  border: 1px solid $primary-50;
  padding: 1rem;
}
.tab-content[data-active='false'] {
  display: none;
}
.tabs-content[data-active='true'] {
  display: block;
}
.config {
  width: 100%;
  height: 100%;
  margin: auto 0;
}
#userMain {
  width: 100%;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
}
</style>
