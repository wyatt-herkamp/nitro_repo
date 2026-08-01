<template>
  <div class="nitroEditorParent">
    <pre class="nitroEditPre hljs">
      <code class="nitroEditorCode" :class="'language-' + code.language" v-html="highlight"/>
    </pre>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";
import java from "highlight.js/lib/languages/java";
import groovy from "highlight.js/lib/languages/groovy";
import kotlin from "highlight.js/lib/languages/kotlin";
import markdown from "highlight.js/lib/languages/markdown";
// The npm snippets are shell commands, `package.json` fragments and `.npmrc` files. An
// unregistered language makes `hljs.highlight` throw, which took the whole card down.
import bash from "highlight.js/lib/languages/bash";
import json from "highlight.js/lib/languages/json";
import ini from "highlight.js/lib/languages/ini";
import javascript from "highlight.js/lib/languages/javascript";
import "highlight.js/styles/atom-one-dark.css";
import type { CodeSnippet } from "./code";
hljs.registerLanguage("xml", xml);
hljs.registerLanguage("kotlin", kotlin);
hljs.registerLanguage("java", java);
hljs.registerLanguage("groovy", groovy);
hljs.registerLanguage("markdown", markdown);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("json", json);
hljs.registerLanguage("ini", ini);
hljs.registerLanguage("javascript", javascript);

const props = defineProps({
  code: {
    required: true,
    type: Object as () => CodeSnippet,
  },
});
const highlight = computed((): string => {
  // A snippet in a language nothing registered would otherwise throw. Showing it unhighlighted
  // beats losing the block.
  if (!hljs.getLanguage(props.code.language)) {
    return escapeHtml(props.code.code);
  }
  return hljs.highlight(props.code.code, {
    language: props.code.language,
    ignoreIllegals: true,
  }).value;
});

// The result is rendered with `v-html`, so anything not passed through hljs has to be escaped
// here instead.
function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
</script>
<style scoped lang="scss">
pre {
  // Was a fixed `height: 10rem`, so a one-line `npm install` got the same 160px box as a ten-line
  // Maven block. The rest of #498 (trimmed templates, `createBlankLines`, cross-browser
  // scrollbars) is part of the frontend revamp; this much is needed for the npm snippets to look
  // like anything at all.
  min-height: 3rem;
  max-height: 25rem;
  overflow: auto;
}
</style>
