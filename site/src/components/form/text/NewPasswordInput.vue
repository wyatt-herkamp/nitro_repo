<template>
  <div id="newPasswordInput">
    <div class="firstPasswordInput">
      <label :for="id">
        <slot />
      </label>
      <input
        @focusin="isFocused = true"
        @focusout="isFocused = false"
        type="password"
        :id="id"
        v-model="internalValue.value"
        v-bind="$attrs" />
      <InputRequirements :validations="validations" :show="isFocused || !isValid" />
    </div>
    <label :for="id + '-confirm'" class="confirmPassword">
      Confirm Password
      <span v-if="passwordsMatchMessage">
        <font-awesome-icon :icon="passwordsMatchMessage.icon" />
        {{ passwordsMatchMessage.message }}
      </span>
    </label>

    <input
      type="password"
      :id="id + '-confirm'"
      v-model="internalValue.confirmValue"
      v-bind="$attrs" />
  </div>
</template>
<script setup lang="ts">
import type { PasswordRules, SiteInfo } from '@/types/base'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { computed, ref, watch, type PropType, type Ref } from 'vue'
import InputRequirements from './InputRequirements.vue'

const props = defineProps({
  id: {
    type: String,
    required: true
  },
  passwordRules: {
    type: Object as PropType<PasswordRules>,
    required: true
  }
})
const passwordsMatch = ref(false)
const isFocused = ref(false)
const isValid = ref(false)
const passwordsMatchMessage = computed(() => {
  if (internalValue.value.value === '' && internalValue.value.confirmValue === '') {
    return undefined
  }
  return passwordsMatch.value
    ? {
        message: 'Passwords Match',
        icon: 'fa-solid fa-circle-check'
      }
    : {
        message: 'Passwords do not match',
        icon: 'fa-solid fa-circle-xmark'
      }
})
const internalValue = ref({
  value: '',
  confirmValue: ''
})
let value = defineModel<string | undefined>({
  required: true
})
watch(
  value,
  (newValue) => {
    internalValue.value = {
      value: newValue || '',
      confirmValue: newValue || ''
    }
  },
  { immediate: true }
)

const validations: Ref<
  {
    message: string
    valid: boolean
    test: (value: string) => boolean
  }[]
> = ref([])

if (props.passwordRules.min_length !== 0) {
  validations.value.push({
    message: `Password must be at least ${props.passwordRules.min_length} characters long`,
    valid: false,
    test: (value: string) => {
      return value.length >= props.passwordRules.min_length
    }
  })
}
if (props.passwordRules.require_uppercase) {
  validations.value.push({
    message: 'Password must contain at least one uppercase letter',
    valid: false,
    test: (value: string) => {
      return /[A-Z]/.test(value)
    }
  })
}
if (props.passwordRules.require_lowercase) {
  validations.value.push({
    message: 'Password must contain at least one lowercase letter',
    valid: false,
    test: (value: string) => {
      return /[a-z]/.test(value)
    }
  })
}
if (props.passwordRules.require_number) {
  validations.value.push({
    message: 'Password must contain at least one number',
    valid: false,
    test: (value: string) => {
      return /\d/.test(value)
    }
  })
}
if (props.passwordRules.require_special) {
  validations.value.push({
    message: 'Password must contain at least one special character',
    valid: false,
    test: (value: string) => {
      return /[!@#$%^&*(),.?":{}|<>]/.test(value)
    }
  })
}

watch(
  internalValue,
  (newValue) => {
    let valid = true
    if (newValue.value !== newValue.confirmValue) {
      passwordsMatch.value = false
    } else {
      passwordsMatch.value = true
    }
    console.log(validations.value)
    for (const validation of validations.value) {
      if (!validation.test(newValue.value)) {
        validation.valid = false
        valid = false
      } else {
        validation.valid = true
      }
      console.log(validation)
    }
    if (value.value !== newValue.value) {
      if (valid && passwordsMatch.value) {
        value.value = newValue.value
      } else {
        value.value = undefined
      }
    }

    isValid.value = valid
  },
  { deep: true }
)
</script>
<style scoped lang="scss">
@import '@/assets/styles/form.scss';
@import '@/assets/styles/theme.scss';
#newPasswordInput {
  min-width: 25rem;
}
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
