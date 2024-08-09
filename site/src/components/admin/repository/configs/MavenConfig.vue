<template>
  <div v-if="!repository">
    <form @submit.prevent="">
      <DropDown v-model="input.mavenType" :options="mavenTypes" required>Maven Type</DropDown>
    </form>
  </div>
  <div v-else-if="value">
    <form>
      <TextInput v-model="value.type" required disabled>Repository Type</TextInput>
    </form>
  </div>
</template>
<script setup lang="ts">
import DropDown from '@/components/form/dropdown/DropDown.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import http from '@/http'
import type { MavenConfigType } from '@/types/repository'
import { computed, defineProps, ref, watch } from 'vue'

const mavenTypes = [
  {
    value: 'Hosted',
    label: 'Hosted'
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
  if (isCreate.value) {
    if (input.value.mavenType === 'Hosted') {
      value.value = {
        type: 'Hosted'
      }
    } else if (input.value.mavenType === 'Proxy') {
      value.value = {
        type: 'Proxy',
        config: {
          goTo: 'TODO'
        }
      }
    } else {
      alert('Illegal state')
    }
  }
  console.log(value.value)
  console.log(input.value)
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
</script>
