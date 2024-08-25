<template>
  <section :id="id + '-section'" class="inputWithRequirements" :data-valid="isValid">
    <label :for="id">
      <slot />
    </label>
    <input
      type="text"
      @focusin="isFocused = true"
      @focusout="isFocused = false"
      :id="id"
      v-model="internalValue"
      v-bind="$attrs" />
    <InputRequirements :show="isFocused" :validations="validations" />
  </section>
</template>
<script setup lang="ts">
import http from '@/http'
import { ref, watch } from 'vue'
import InputRequirements from './InputRequirements.vue'

const props = defineProps({
  id: String,
  firstUser: {
    type: Boolean,
    default: false
  },
  originalValue: {
    type: String,
    required: false
  }
})
const isFocused = ref(false)
async function isTaken(value: string): Promise<boolean> {
  if (props.firstUser) {
    return true
  }
  if (props.originalValue) {
    if (value === props.originalValue) {
      return true
    }
  }
  let isAvailable = true
  await http
    .post(`/api/user-management/is-taken`, {
      type: 'Username',
      value: value
    })
    .then(() => {
      isAvailable = true
    })
    .catch((response) => {
      if (response.response.status === 409) {
        isAvailable = false
      } else if (response.response.status === 400) {
        console.error('The other checks should have caught this.')
        isAvailable = false
      }
    })
  console.log(`Username ${value} is aviailable: ${isAvailable}`)
  return isAvailable
}
const validations = ref([
  {
    message: 'Must be at least 3 characters long.',
    test: (value: string) => Promise.resolve(value.length >= 3),
    valid: false,
    ignoreOnFirstUser: false
  },
  {
    message: 'No Longer than 32 characters.',
    test: (value: string) => Promise.resolve(value.length <= 32),
    valid: false
  },
  {
    message: 'Can only contain letters, numbers, underscores and dashes.',
    test: (value: string) => Promise.resolve(/^[a-zA-Z0-9_-]*$/.test(value)),
    valid: false,
    ignoreOnFirstUser: false
  },
  {
    message: 'Username is available.',
    test: (value: string) => isTaken(value),
    valid: false,

    ignoreOnFirstUser: true
  }
])
const internalValue = ref(props.originalValue ?? '')
const isValid = ref(false)
let value = defineModel<string | undefined>({
  required: true
})
watch(internalValue, async () => {
  let valid = true
  for (const validation of validations.value) {
    console.log(validation)
    if (!(await validation.test(internalValue.value))) {
      validation.valid = false
      valid = false
    } else {
      validation.valid = true
    }
  }
  if (valid) {
    value.value = internalValue.value
  } else {
    value.value = undefined
  }
  isValid.value = valid
})
if (props.originalValue) {
  internalValue.value = props.originalValue
  for (const validation of validations.value) {
    validation.valid = true
  }
  isValid.value = true
}
</script>
<style lang="scss" scoped>
@import '@/assets/styles/form.scss';
@import '@/assets/styles/theme.scss';

.inputWithRequirements[data-valid='false'] {
  input {
    border-color: $invalid-form-field;
  }
}
</style>
