<template>
  <prism-editor
    class="nitroEditor"
    v-model="highlighterComputed.snippet"
    :highlight="highlighterComputed.highlighter"
    :line-numbers="false"
    readonly
  ></prism-editor>
</template>

<script lang="ts">
import { computed, defineComponent } from "vue";
import { PrismEditor } from "vue-prism-editor";
import "vue-prism-editor/dist/prismeditor.min.css";
import "@/prismjs/themes/prism-material-light.css";
import prism from "prismjs";
import { SnippetInfo } from "@/api/CodeGenGeneral";
import "prismjs/components/prism-markdown";
import "prismjs/components/prism-groovy";
import "prismjs/components/prism-kotlin";

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
          prism.languages[props.snippetInfo.grammar] ?? prism.languages.js,
          props.snippetInfo.lang
        ),
      ...props.snippetInfo,
    }));
    return { highlighterComputed };
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
