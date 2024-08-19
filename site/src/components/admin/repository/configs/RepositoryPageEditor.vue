<template>
  <div>
    <DropDown v-model="input.page_type" :options="pageTypes" required>Page Type</DropDown>
    <div v-if="input.page_type === PageType.Markdown" id="markdownEditor">
      <MilkdownProvider>
        <MarkdownEditor
          :value="input.content"
          @update="
            (content: string) => {
              input.content = content
            }
          " />
      </MilkdownProvider>
    </div>
    <SubmitButton @click="save">Save </SubmitButton>
  </div>
</template>
<script setup lang="ts">
import DropDown from '@/components/form/dropdown/DropDown.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import http from '@/http'
import { type RepositoryPage, type MavenConfigType, PageType } from '@/types/repository'
import { MilkdownProvider } from '@milkdown/vue'
import { computed, defineProps, ref, watch } from 'vue'
import MarkdownEditor from './MarkdownEditor.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
const pageTypes = [
  {
    value: PageType.None,
    label: 'None'
  },
  {
    value: PageType.Markdown,
    label: 'Markdown'
  }
]
const props = defineProps({
  settingName: String,
  repository: {
    type: String,
    required: true
  }
})
const input = ref<RepositoryPage>({
  page_type: PageType.None,
  content: '# Hello World'
})

const value = defineModel<MavenConfigType>()
watch(input.value, () => {
  console.log('input changed')
  console.log(input.value.content)
})
async function load() {
  if (props.repository) {
    await http
      .get(`/api/repository/${props.repository}/config/page`)
      .then((response) => {
        input.value = response.data
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
      .put(`/api/repository/${props.repository}/config/page`, input.value)
      .then(() => {
        console.log('Saved')
      })
      .catch((error) => {
        console.error(error)
      })
  }
}
</script>
<style scoped lang="scss">
#markdownEditor {
  margin-top: 20px;
  min-height: 20rem;
}
</style>
