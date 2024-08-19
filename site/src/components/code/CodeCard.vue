<template>
  <div class="nitroEditorParent">
    <pre class="nitroEditor hljs">
    <code class="nitroEditor" :class="'language-' + snippetInfo.language" v-html="highlight"/>
  </pre>
  </div>
</template>

<script lang="ts" setup>
import { computed, defineComponent, onMounted } from 'vue'
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
  snippetInfo: {
    required: true,
    type: Object as () => CodeSnippet
  }
})
console.log(props.snippetInfo)
const highlight = computed((): string => {
  return hljs.highlight(props.snippetInfo.code, {
    language: props.snippetInfo.language,
    ignoreIllegals: true
  }).value
})
</script>
<style>
pre.nitroEditor {
  white-space: pre-wrap;
}
code.nitroEditor {
  white-space: pre;
}
.nitroEditorParent {
  width: 100%;
  overflow-x: auto;
}
</style>
