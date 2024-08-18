<template>
  <section :id="id + '-section'" class="inputWithRequirements" :data-valid="isValid">
    <label :for="id">
      <slot />
    </label>
    <input
      type="email"
      @focusin="isFocused = true"
      @focusout="isFocused = false"
      :id="id"
      v-model="internalValue"
      v-bind="$attrs"
    />
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
      type: 'Email',
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
  console.log(`Email ${value} is aviailable: ${isAvailable}`)
  return isAvailable
}
const validations = ref([
  {
    message: 'Email is available.',
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
