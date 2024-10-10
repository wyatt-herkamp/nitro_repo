<template>
  <section
    :id="id + '-section'"
    class="inputWithRequirements"
    :data-valid="isValid">
    <label :for="id">
      <slot />
    </label>
    <input
      :type="type ?? 'text'"
      @focusin="isFocused = true"
      @focusout="isFocused = false"
      @keypress="keyPress"
      :id="id"
      v-model="internalValue"
      v-bind="$attrs" />
    <InputRequirements
      :show="isFocused"
      :validations="validations"
      :results="validationResults" />
  </section>
</template>
<script setup lang="ts">
import { ref, watch, type PropType } from "vue";
import { checkValidations, type KeyPressAction, type ValidationType } from "./validations";
import InputRequirements from "./InputRequirements.vue";

const props = defineProps({
  id: String,
  originalValue: {
    type: String,
    required: false,
  },
  type: {
    type: String,
    required: false,
  },
  validations: {
    type: Array as PropType<ValidationType[]>,
    required: true,
  },
  deniedKeys: {
    type: Array as PropType<KeyPressAction[]>,
    required: false,
  },
});

function keyPress(event: KeyboardEvent) {
  if (props.deniedKeys) {
    for (const deniedKey of props.deniedKeys) {
      if (typeof deniedKey === "string") {
        if (event.key === deniedKey) {
          event.preventDefault();
          return;
        }
      } else {
        if (deniedKey.badKey === event.key) {
          event.preventDefault();
          // Add the replacement character
          internalValue.value += deniedKey.replacedKey;
          return;
        }
      }
    }
  }
}
const isFocused = ref(false);
const validationResults = ref<Record<string, boolean>>({});

const internalValue = ref(props.originalValue ?? "");
const isValid = ref(false);
const value = defineModel<string | undefined>({
  required: true,
});
watch(internalValue, async () => {
  const { isValid: newIsValid, validationResults: newValidationResults } = await checkValidations(
    props.validations,
    internalValue.value,
  );
  validationResults.value = newValidationResults;

  if (newIsValid) {
    value.value = internalValue.value;
  } else {
    value.value = undefined;
  }
  isValid.value = newIsValid;
});
if (props.originalValue) {
  internalValue.value = props.originalValue;
  for (const validation of props.validations) {
    validationResults.value[validation.id] = true;
  }

  isValid.value = true;
}
</script>
<style lang="scss" scoped>
@import "@/assets/styles/form.scss";
@import "@/assets/styles/theme.scss";

.inputWithRequirements[data-valid="false"] {
  input {
    border-color: $invalid-form-field;
  }
}
</style>
