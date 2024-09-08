<template>
  <main>
    <h1>Storage Create</h1>
    <form @submit.prevent="createStorage()">
      <TwoByFormBox>
        <TextInput v-model="input.name" autocomplete="none" required placeholder="Storage Name"
          >Storage Name</TextInput
        >
        <DropDown v-model="input.storageType" :options="storageOptions" required
          >Storage Type</DropDown
        >
      </TwoByFormBox>
      <div v-if="storageConfig" class="storageConfig">
        <h2>{{ storageConfig.title }}</h2>
        <h3 v-if="error != ''">
          {{ error }}
        </h3>
        <component :is="storageConfig.component" v-model="input.storageConfigValue"></component>
      </div>
      <SubmitButton v-if="storageConfig">Create</SubmitButton>
    </form>
  </main>
</template>

<script lang="ts" setup>
import DropDown from '@/components/form/dropdown/DropDown.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import TwoByFormBox from '@/components/form/TwoByFormBox.vue'
import { getStorageType, storageTypes } from '@/components/nr/storage/storageTypes'
import http from '@/http'
import router from '@/router'
import { notify } from '@kyvg/vue3-notification'
import { computed, ref } from 'vue'
const input = ref({
  name: '',
  storageType: '',
  storageConfigValue: {}
})
const error = ref('')
const storageOptions = ref(storageTypes)
const storageConfig = computed(() => {
  if (input.value.storageType === '') {
    return undefined
  }
  const current = getStorageType(input.value.storageType)
  return current
})
async function createStorage() {
  console.log(input.value)
  let data = {
    name: input.value.name,
    config: {
      type: input.value.storageType,
      settings: input.value.storageConfigValue
    }
  }

  await http
    .post(`/api/storage/new/${input.value.storageType}`, data)
    .then((response) => {
      console.log(response)
      notify({
        type: 'success',
        title: 'Storage Created',
        text: 'The storage has been created.'
      })
      router.push({
        name: 'ViewStorage',
        params: { id: response.data.id }
      })
    })
    .catch((error) => {
      console.log(error)
      if (error.response.status === 400) {
        console.log(error.response.data)
      }
    })
}
</script>
<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 50%;
  padding: 1rem;
  margin: 0 auto;
}
.storageConfig {
  padding: 1rem;
  border: 1px solid $secondary;
  border-radius: 0.5rem;
}
@media screen and (max-width: 1200px) {
  form {
    width: 100%;
  }
}
main {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
</style>
