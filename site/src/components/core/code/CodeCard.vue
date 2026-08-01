<template>
  <figure class="codeCard">
    <figcaption class="codeHeader">
      <span class="codeLanguage">{{ code.language }}</span>
      <button
        type="button"
        class="copyButton"
        :aria-label="copied ? 'Copied' : 'Copy to clipboard'"
        @click="copy">
        <font-awesome-icon :icon="copied ? 'check' : 'copy'" />
        <span>{{ copied ? "Copied" : "Copy" }}</span>
      </button>
    </figcaption>

    <pre class="codeBody hljs"><code
      :class="'language-' + code.language"
      v-html="highlighted" /></pre>
  </figure>
</template>

<script lang="ts" setup>
/**
 * A syntax-highlighted snippet. (#498)
 *
 * The whole style block used to be `pre { height: 10rem; overflow: auto; }` — a *fixed* height, so
 * a one-line `npm install` got the same 160px box as a ten-line Maven block. There was a
 * `createBlankLines()` helper whose only job was padding short snippets out to fill it.
 */
import { computed, ref } from "vue";
import { notify } from "@kyvg/vue3-notification";
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

const copied = ref(false);

const highlighted = computed((): string => {
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

// The result is rendered with `v-html`, so anything not passed through hljs has to be escaped here
// instead.
function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

async function copy() {
  try {
    await navigator.clipboard.writeText(props.code.code);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1600);
  } catch {
    // Clipboard access is refused outside a secure context, which includes plain-HTTP instances.
    notify({ type: "error", title: "Could not copy", text: "Select the snippet and copy it." });
  }
}
</script>

<style scoped lang="scss">
// Set as an instrument readout: a labelled header strip over a sunken panel.
.codeCard {
  margin: 0;
  background-color: var(--bg-sunken);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.codeHeader {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  background-color: var(--surface);
  border-bottom: 1px solid var(--border);
}

.codeLanguage {
  font-size: var(--text-2xs);
  font-weight: var(--weight-semibold);
  letter-spacing: var(--tracking-label);
  text-transform: uppercase;
  color: var(--text-subtle);
}

.copyButton {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0.1875rem 0.5rem;
  font-family: inherit;
  font-size: var(--text-xs);
  color: var(--text-muted);
  background: transparent;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    color var(--duration-fast) var(--ease-out),
    border-color var(--duration-fast) var(--ease-out);

  &:hover {
    color: var(--accent);
    border-color: var(--accent-border);
  }
}

.codeBody {
  // `min-height`/`max-height` rather than a fixed height, so the box follows the snippet.
  min-height: 2.5rem;
  max-height: 25rem;
  margin: 0;
  padding: var(--space-3);
  overflow: auto;
  font-size: var(--text-sm);
  line-height: var(--leading-normal);
  // highlight.js ships its own background; the card owns that.
  background: transparent;

  code {
    background: transparent;
  }
}
</style>
