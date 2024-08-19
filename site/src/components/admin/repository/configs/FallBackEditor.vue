<template>
  <div v-if="form && model" class="editorBox">
    <h2 class="settingHeader">Generic Config Editor: {{ settingName }}</h2>

    <JsonSchemaForm :form="form" v-model="model" />
    <SubmitButton>Update Settings</SubmitButton>
  </div>
</template>
<script setup lang="ts">
import SubmitButton from '@/components/form/SubmitButton.vue'
import http from '@/http'
import { computed, ref, type PropType } from 'vue'
import { repositoriesStore } from '@/stores/repositories'
import JsonSchemaForm from '@/components/form/JsonSchemaForm.vue'
import { createForm, type RootSchema } from 'nitro-jsf'

const schema = ref<RootSchema | undefined>(undefined)
const form = computed(() => {
  if (schema.value) {
    return createForm(schema.value)
  }
  return undefined
})
const props = defineProps({
  settingName: String,
  repository: {
    type: Object as PropType<string>,
    required: false
  }
})
const model = defineModel<any>()
const repositoryTypeStore = repositoriesStore()
async function load() {
  if (!props.settingName) {
    throw new Error('settingName is required')
  }
  await repositoryTypeStore.getConfigSchema(props.settingName).then((response) => {
    schema.value = response as RootSchema
    console.log(schema.value)
  })
  if (props.repository) {
    await http
      .get(`/api/repository/repository/${props.repository}/config/${props.settingName}`)
      .then((response) => {
        model.value = response.data
      })
      .catch((error) => {
        console.error(error)
      })
    if (!model.value) {
      loadDefault()
    }
  } else {
    loadDefault()
  }
}
async function loadDefault() {
  await http
    .get(`/api/repository/repository/${props.repository}/config/${props.settingName}`)
    .then((response) => {
      model.value = response.data
    })
    .catch((error) => {
      console.error(error)
    })
}
load()
</script>
<style lang="scss"></style>
