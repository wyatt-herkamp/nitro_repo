<template>
  <VueFinalModal
    :model-value="modelValue"
    class="modalWrapper"
    content-class="modalContent"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
    :click-to-close="closeOnOutsideClick"
    :esc-to-close="true"
    @update:model-value="(value: boolean) => emit('update:modelValue', value)">
    <header class="modalHeader">
      <h2>{{ title }}</h2>
      <button
        type="button"
        class="closeButton"
        aria-label="Close"
        @click="close">
        <font-awesome-icon icon="x" />
      </button>
    </header>

    <div class="modalBody">
      <slot />
    </div>

    <footer
      v-if="$slots.footer"
      class="modalFooter">
      <slot name="footer" />
    </footer>
  </VueFinalModal>
</template>

<script setup lang="ts">
/**
 * `vue-final-modal` has been a dependency, installed and registered, with zero usages. This is the
 * shared shell so that the destructive actions in the app — deleting a repository, deleting a token
 * — can finally ask before they fire.
 */
import { VueFinalModal } from "vue-final-modal";

withDefaults(
  defineProps<{
    modelValue: boolean;
    title: string;
    /**
     * Off by default: these dialogs mostly wrap destructive confirmations and short forms, where a
     * stray click outside should not discard what someone typed.
     */
    closeOnOutsideClick?: boolean;
  }>(),
  { closeOnOutsideClick: false },
);

const emit = defineEmits<{ (e: "update:modelValue", value: boolean): void }>();

function close() {
  emit("update:modelValue", false);
}
</script>

<style lang="scss">
// Unscoped because `content-class` lands on an element `vue-final-modal` renders, which scoped
// styles cannot reach. Both class names are specific enough not to collide.
.modalWrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-4);
}

.modalWrapper .vfm__overlay {
  background-color: var(--overlay);
  backdrop-filter: blur(2px);
}

.modalContent {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 32rem;
  max-height: calc(100vh - var(--space-8));
  background-color: var(--surface);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}
</style>

<style scoped lang="scss">
.modalHeader {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border);

  h2 {
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
  }
}

.closeButton {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  color: var(--text-muted);
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;

  &:hover {
    color: var(--text);
    background-color: var(--surface-hover);
  }
}

.modalBody {
  padding: var(--space-5);
  overflow-y: auto;
}

.modalFooter {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5);
  background-color: var(--bg-sunken);
  border-top: 1px solid var(--border);
}
</style>
