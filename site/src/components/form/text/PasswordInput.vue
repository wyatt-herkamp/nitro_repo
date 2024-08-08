<template>
  <div class="formBox">
    <label :for="id">
      <slot />
    </label>

    <div class="input-container">
      <input
        :type="passwordType"
        :id="id"
        v-model="value"
        v-bind="$attrs"
        :autocomplete="autocomplete"
      />
      <button @click="togglePasswordVisibility">
        <font-awesome-icon icon="fa-solid fa-eye-slash" v-if="showPassword" />
        <font-awesome-icon icon="fa-solid fa-eye" v-else />
      </button>
    </div>
  </div>
</template>
<script setup lang="ts">
import '@/assets/styles/form.scss'
import { computed, ref } from 'vue'
const props = defineProps({
  id: String,
  newPassword: {
    type: Boolean,
    default: false
  }
})
let value = defineModel<string>({
  required: true
})
const autocomplete = computed(() => (props.newPassword ? 'new-password' : 'password'))
const showPassword = ref(false)
const passwordType = computed(() => (showPassword.value ? 'text' : 'password'))
function togglePasswordVisibility() {
  showPassword.value = !showPassword.value
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
.input-container {
  position: relative;
  display: inline-block;
  width: 100%;

  input {
    width: 100%;
  }

  button {
    position: absolute;
    right: 10px;
    top: 35%;
    transform: translateY(-25%);
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 16px;
    color: $text;
    transition: color 0.3s ease;
  }
}
</style>
