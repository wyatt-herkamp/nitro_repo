<template>
  <section class="firstPasswordInput">
    <label :for="id">
      <slot />
    </label>
    <input
      @focusin="isFocused = true"
      @focusout="isFocused = false"
      type="password"
      :id="id"
      autocomplete="new-password"
      v-model="internalValue.value"
      v-bind="$attrs" />
    <InputRequirements
      :show="isFocused"
      :validations="validations"
      :results="validationResults" />
  </section>
  <section>
    <label
      :for="id + '-confirm'"
      class="confirmPassword">
      Confirm Password
      <span v-if="passwordsMatchMessage">
        <font-awesome-icon :icon="passwordsMatchMessage.icon" />
        {{ passwordsMatchMessage.message }}
      </span>
    </label>

    <input
      type="password"
      :id="id + '-confirm'"
      autocomplete="new-password"
      v-model="internalValue.confirmValue"
      v-bind="$attrs" />
  </section>
</template>
<script setup lang="ts">
import type { PasswordRules } from "@/types/base";
import { computed, ref, watch, type PropType, type Ref } from "vue";
import InputRequirements from "./InputRequirements.vue";
import { siteStore } from "@/stores/site";
import { checkValidations, passwordValidationRules, type SyncValidationType } from "./validations";

const props = defineProps({
  id: {
    type: String,
    required: true,
  },

  passwordRules: {
    type: Object as PropType<PasswordRules>,
    required: false,
  },
});
const actualPasswordRules = computed(() => {
  if (props.passwordRules) {
    return props.passwordRules;
  }
  const site = siteStore();
  return site.getPasswordRulesOrDefault();
});
const passwordsMatch = ref(false);
const isFocused = ref(false);
const isValid = ref(false);
const passwordsMatchMessage = computed(() => {
  if (internalValue.value.value === "" && internalValue.value.confirmValue === "") {
    return undefined;
  }
  return passwordsMatch.value
    ? {
        message: "Passwords Match",
        icon: "fa-solid fa-circle-check",
      }
    : {
        message: "Passwords do not match",
        icon: "fa-solid fa-circle-xmark",
      };
});
const internalValue = ref({
  value: "",
  confirmValue: "",
});
const validationResults = ref<Record<string, boolean>>({});
const value = defineModel<string | undefined>({
  required: true,
});
watch(
  value,
  (newValue) => {
    internalValue.value = {
      value: newValue || "",
      confirmValue: newValue || "",
    };
  },
  { immediate: true },
);

const validations: Ref<Array<SyncValidationType>> = ref(
  passwordValidationRules(actualPasswordRules.value),
);

watch(
  internalValue,
  async (newValue) => {
    if (newValue.value !== newValue.confirmValue) {
      passwordsMatch.value = false;
    } else {
      passwordsMatch.value = true;
    }
    console.log(validations.value);
    const { isValid: newIsValid, validationResults: newValidationResults } = await checkValidations(
      validations.value,
      internalValue.value.value,
    );
    validationResults.value = newValidationResults;
    isValid.value = newIsValid;

    if (value.value === newValue.value) {
      return;
    }
    if (newIsValid && passwordsMatch.value) {
      console.log("Setting value");
      value.value = newValue.value;
    } else {
      console.log("Setting value to undefined");
      value.value = undefined;
    }
  },
  { deep: true },
);
</script>
<style scoped lang="scss">
@import "@/assets/styles/form.scss";
@import "@/assets/styles/theme.scss";

.inputs {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  label {
    width: 100%;
    text-align: left;
  }
  input {
    width: 100%;
  }
}
.firstPasswordInput {
  margin: 1rem 0;
}
.confirmPassword {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  justify-content: space-between;
}
</style>
