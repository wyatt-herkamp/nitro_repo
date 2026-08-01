<template>
  <NModal
    :model-value="modelValue"
    :title="title"
    @update:model-value="(value: boolean) => emit('update:modelValue', value)">
    <p class="message">{{ message }}</p>

    <div
      v-if="confirmText"
      class="confirmField">
      <label :for="fieldId">
        Type <code>{{ confirmText }}</code> to confirm
      </label>
      <input
        :id="fieldId"
        v-model="typed"
        autocomplete="off"
        spellcheck="false" />
    </div>

    <template #footer>
      <NButton
        variant="ghost"
        @click="cancel"
        >Cancel</NButton
      >
      <NButton
        :variant="destructive ? 'danger' : 'primary'"
        :disabled="!canConfirm"
        :loading="loading"
        @click="confirm">
        {{ confirmLabel }}
      </NButton>
    </template>
  </NModal>
</template>

<script setup lang="ts">
/**
 * Confirmation for an action that cannot be undone.
 *
 * Deleting a repository and deleting a token both used to fire straight from the click handler with
 * nothing in between. For the genuinely unrecoverable ones, `confirmText` additionally requires the
 * name to be typed — the standard guard against muscle-memory on a misread row.
 */
import { computed, ref, useId, watch } from "vue";
import NModal from "./NModal.vue";
import NButton from "./NButton.vue";

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    destructive?: boolean;
    loading?: boolean;
    /** When set, the confirm button stays disabled until this exact string is typed. */
    confirmText?: string;
  }>(),
  {
    confirmLabel: "Confirm",
    destructive: false,
    loading: false,
    confirmText: undefined,
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const fieldId = useId();
const typed = ref("");

// Reopening the dialog must not inherit what was typed last time, or a second delete would be
// pre-confirmed against a different target.
watch(
  () => props.modelValue,
  (open) => {
    if (open) typed.value = "";
  },
);

const canConfirm = computed(
  () => !props.loading && (!props.confirmText || typed.value === props.confirmText),
);

function confirm() {
  if (canConfirm.value) emit("confirm");
}

function cancel() {
  emit("cancel");
  emit("update:modelValue", false);
}
</script>

<style scoped lang="scss">
.message {
  color: var(--text-muted);
  font-size: var(--text-sm);
}

.confirmField {
  margin-top: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);

  code {
    color: var(--text);
    background-color: var(--bg-sunken);
    padding: 0.0625rem 0.25rem;
    border-radius: var(--radius-sm);
  }
}
</style>
