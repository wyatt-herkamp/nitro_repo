<template>
  <div v-if="!repository">
    <form @submit.prevent="">
      <DropDown v-model="input.mavenType" :options="mavenTypes" required>Maven Type</DropDown>
      <div v-if="value && value.type === 'Proxy'">
        <MavenProxyConfig v-model="value.config as MavenProxyConfigType" />
      </div>
    </form>
  </div>
  <div v-else-if="value">
    <form @submit.prevent="save">
      <TextInput v-model="value.type" required disabled>Repository Type</TextInput>
      <div v-if="value && value.type === 'Proxy'">
        <MavenProxyConfig v-model="value.config as MavenProxyConfigType" />
      </div>
      <SubmitButton>Save</SubmitButton>
    </form>
  </div>
</template>
<script setup lang="ts">
import DropDown from '@/components/form/dropdown/DropDown.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import http from '@/http'
import { defaultProxy, type MavenConfigType, type MavenProxyConfigType } from './maven'
import { computed, defineProps, ref, watch } from 'vue'
import { notify } from '@kyvg/vue3-notification'
import MavenProxyConfig from './MavenProxyConfig.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'

const mavenTypes = [
  {
    value: 'Hosted',
    label: 'Hosted'
  },
  {
    value: 'Proxy',
    label: 'Proxy'
  }
]
const props = defineProps({
  settingName: String,
  repository: {
    type: String,
    required: false
  }
})
const input = ref({
  mavenType: ''
})
const isCreate = computed(() => {
  return !props.repository
})
const value = defineModel<MavenConfigType>()
watch(input.value, () => {
  console.log(input.value)

  if (input.value.mavenType === '') {
    return
  }
  if (isCreate.value) {
    if (input.value.mavenType === 'Hosted') {
      value.value = {
        type: 'Hosted'
      }
    } else if (input.value.mavenType === 'Proxy') {
      value.value = {
        type: 'Proxy',
        config: defaultProxy()
      }
    } else {
      notify({
        type: 'error',
        title: 'Error',
        text: 'Invalid maven type'
      })
      input.value.mavenType = ''
    }
  }
  console.log(value.value)
})
async function load() {
  if (props.repository) {
    await http
      .get(`/api/repository/${props.repository}/config/maven`)
      .then((response) => {
        value.value = response.data
      })
      .catch((error) => {
        console.error(error)
      })
  }
}
load()
async function save() {
  if (props.repository) {
    await http
      .put(`/api/repository/${props.repository}/config/maven`, value.value)
      .then(() => {
        console.log('Saved')
      })
      .catch((error) => {
        console.error(error)
      })
  }
}
</script>
