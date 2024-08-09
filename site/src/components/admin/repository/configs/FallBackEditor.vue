<template>
  <div v-if="schema && validator" class="editorBox">
    <h2 class="settingHeader">Generic Config Editor: {{ settingName }}</h2>

    <JsonEditorVue class="editor" :validator="validator" v-model="model" />
    <SubmitButton>Update Settings</SubmitButton>
  </div>
</template>
<script setup lang="ts">
import SubmitButton from '@/components/form/SubmitButton.vue'
import http from '@/http'
import { createAjvValidator, type JSONSchema } from 'vanilla-jsoneditor'
import { computed, ref, type PropType } from 'vue'
import JsonEditorVue from 'json-editor-vue'

const schema = ref<JSONSchema | undefined>(undefined)
const validator = computed(() => {
  if (schema.value) {
    return createAjvValidator({
      schema: schema.value
    })
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

async function load() {
  await http
    .get(`/api/repository/config/${props.settingName}/schema`)
    .then((response) => {
      schema.value = response.data
    })
    .catch((error) => {
      console.error(error)
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
    .get(`/api/repository/config/${props.settingName}/default`)
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
