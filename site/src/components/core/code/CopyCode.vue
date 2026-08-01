<template>
  <div class="copyURL">
    <label v-if="$slots.default">
      <slot />
    </label>
    <button
      type="button"
      class="value"
      :title="copied ? 'Copied' : 'Copy'"
      @click="copy">
      <span class="mono">{{ code }}</span>
      <font-awesome-icon :icon="copied ? 'check' : 'copy'" />
    </button>
  </div>
</template>

<script setup lang="ts">
// A `<button>` rather than a `<span>` with a click handler, so it is reachable by keyboard and
// announced as something you can activate.
import { ref } from "vue";
import { notify } from "@kyvg/vue3-notification";

const props = defineProps({
  code: {
    type: String,
    required: true,
  },
});

const copied = ref(false);

async function copy() {
  try {
    await navigator.clipboard.writeText(props.code);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1600);
  } catch {
    // Clipboard access is refused outside a secure context, which includes plain-HTTP instances.
    notify({ type: "error", title: "Could not copy", text: "Select the text and copy it." });
  }
}
</script>

<style lang="scss" scoped>
.copyURL {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.value {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  width: 100%;
  padding: var(--space-2) var(--space-3);
  font-family: inherit;
  font-size: var(--text-sm);
  color: var(--text);
  text-align: left;
  background-color: var(--bg-sunken);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-out);

  &:hover {
    border-color: var(--accent-border);
  }

  span {
    overflow-wrap: anywhere;
  }

  svg {
    flex-shrink: 0;
    color: var(--text-subtle);
  }
}
</style>
