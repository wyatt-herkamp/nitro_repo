<template>
  <code
    class="nitroEditor"
    :class="'language-' + snippetInfo.lang"
    v-html="highlight"
  >
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
import markdown from "highlight.js/lib/languages/markdown";
import "highlight.js/styles/atom-one-dark.css";
export default defineComponent({
  props: {
    snippetInfo: {
      required: true,
      type: Object as () => SnippetInfo,
    },
  },
  setup(props) {
    hljs.registerLanguage("xml", xml);
    hljs.registerLanguage("kotlin", kotlin);
    hljs.registerLanguage("java", java);
    hljs.registerLanguage("groovy", groovy);
    hljs.registerLanguage("markdown", markdown);
    const highlight = computed((): string => {
      return hljs.highlight(props.snippetInfo.snippet, {
        language: props.snippetInfo.lang,
        ignoreIllegals: true,
      }).value;
    });
    return {
      highlight,
    };
  },
});
</script>
<style></style>
