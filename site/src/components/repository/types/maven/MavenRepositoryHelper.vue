<template>
  <div class="mavenRepository">
    <div @click="clickToCopyURL" title="Click to Copy URL" class="clickToCopy">
      <h2>Repository URL</h2>
      <code>
        {{ url }}
      </code>
    </div>
    <div>
      <h2>Accessing The Repository</h2>
      <CodeMenu :snippets="snippets" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { apiURL } from '@/config'
import { createRepositoryRoute, type RepositoryWithStorageName } from '@/types/repository'
import { computed, type PropType } from 'vue'
import { createSnippetsForPulling } from './MavenRepositoryHelpers'
import CodeMenu from '@/components/code/CodeMenu.vue'
import { notify } from '@kyvg/vue3-notification'
function clickToCopyURL() {
  window.navigator.clipboard.writeText(url.value)
  notify({
    type: 'success',
    title: 'Copied',
    text: 'Repository URL copied to clipboard'
  })
}
const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true
  }
})
const snippets = createSnippetsForPulling(props.repository)
const url = computed(() => {
  return createRepositoryRoute(props.repository)
})
</script>
<style lang="scss" scoped>
.clickToCopy {
  cursor: pointer;
  code {
    padding: 0.5rem;
    border-radius: 0.25rem;
    display: block;
    margin-top: 0.5rem;
  }
}
</style>
