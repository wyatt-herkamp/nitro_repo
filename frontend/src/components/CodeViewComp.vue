<template>
  <prism-editor
    class="card-editor font-mono text-ssm absolute"
    v-model="highlighterComputed.snippet"
    :highlight="highlighterComputed.highlighter"
    line-numbers
  ></prism-editor>
</template>

<script lang="ts">
import { computed, defineComponent } from "vue";
import { PrismEditor } from "vue-prism-editor";
import "vue-prism-editor/dist/prismeditor.min.css";
import prism from "prismjs";
import { SnippetInfo } from "@/api/CodeGenGeneral";

import "@/styles/prism-atom-dark.css";

export default defineComponent({
  components: { PrismEditor },
  props: {
    snippetInfo: {
      required: true,
      type: Object as () => SnippetInfo,
    },
  },
  setup(props) {
    const highlighterComputed = computed(() => ({
      highlighter: (code: string) =>
        prism.highlight(
          code,
          prism.languages[props.snippetInfo.lang] ?? prism.languages.js
        ),
      ...props.snippetInfo,
    }));
    return { highlighterComputed };
  },
});
</script>
