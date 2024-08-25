<template>
  <div class="nitroEditorParent">
    <pre class="nitroEditPre hljs betterScroll">
      <code class="nitroEditorCode" :class="'language-' + code.language" v-html="highlight"/>
    </pre>
  </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue'
import hljs from 'highlight.js/lib/core'
import xml from 'highlight.js/lib/languages/xml'
import java from 'highlight.js/lib/languages/java'
import groovy from 'highlight.js/lib/languages/groovy'
import kotlin from 'highlight.js/lib/languages/kotlin'
import markdown from 'highlight.js/lib/languages/markdown'
import 'highlight.js/styles/atom-one-dark.css'
import type { CodeSnippet } from './code'
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('kotlin', kotlin)
hljs.registerLanguage('java', java)
hljs.registerLanguage('groovy', groovy)
hljs.registerLanguage('markdown', markdown)

const props = defineProps({
  code: {
    required: true,
    type: Object as () => CodeSnippet
  }
})
console.log(props.code)
const highlight = computed((): string => {
  return hljs.highlight(props.code.code, {
    language: props.code.language,
    ignoreIllegals: true
  }).value
})
</script>
<style scoped lang="scss">
@import '@/assets/styles/scroll.scss';

pre {
  height: 10rem;
  overflow: auto;
}
</style>
