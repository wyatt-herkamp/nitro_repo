<template>
  <code
    style="white-space: pre-line"
    :class="'nitroEditor language-' + snippetInfo.lang"
  >
    {{ snippetInfo.snippet }}
  </code>
</template>

<script lang="ts">
import { computed, defineComponent, onMounted } from "vue";
import { SnippetInfo } from "@/api/CodeGenGeneral";
import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";
import java from "highlight.js/lib/languages/java";
import groovy from "highlight.js/lib/languages/groovy";
import kotlin from "highlight.js/lib/languages/kotlin";
import "highlight.js/styles/atom-one-dark.css";
export default defineComponent({
  props: {
    snippetInfo: {
      required: true,
      type: Object as () => SnippetInfo,
    },
  },
  setup() {
    onMounted(() => {
      hljs.registerLanguage("xml", xml);
      hljs.registerLanguage("kotlin", kotlin);
      hljs.registerLanguage("java", java);
      hljs.registerLanguage("groovy", groovy);
      hljs.highlightAll();
    });
  },
});
</script>
<style>
.nitroEditor {
  font-family: "Fira Code", monospace;
  font-size: 16px;
  @apply text-white;
}
</style>
