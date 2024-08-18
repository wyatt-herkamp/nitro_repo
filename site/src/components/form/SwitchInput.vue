<template>
  <div class="switchBox">
    <label :for="id" class="switchBoxText">
      <slot />
    </label>
    <span v-if="$slots.comment" class="comment">
      <slot name="comment"></slot>
    </span>
    <BaseSwitch :id="id" v-model="value" />
  </div>
</template>
<script setup lang="ts">
import { ref, watch } from 'vue'
import BaseSwitch from './BaseSwitch.vue'
const props = defineProps({
  id: {
    type: String,
    required: true
  }
})
let value = defineModel<boolean>({
  required: true
})
const emit = defineEmits<{
  (e: 'change', newValue: boolean): void
}>()
watch(value, (newValue) => {
  emit('change', newValue)
})
</script>

<style scoped lang="scss">
@import '@/assets/styles/theme';

.switchBox {
  margin: 1rem 2rem;
  display: flex;
  flex-direction: column;
  align-items: left;
  justify-content: space-between;
}

.switchBox > label {
  margin-right: 1rem;
}

.switchBoxText {
  font-size: 1.5rem;
  margin: auto 0;
  padding-bottom: 0;
}

.switch {
  position: relative;
  display: inline-block;
  width: 60px;
  height: 34px;
}

/* Hide default HTML checkbox */
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

/* The slider */
.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: red;
  transition: 0.4s;
}

.slider:before {
  position: absolute;
  content: '';
  height: 26px;
  width: 26px;
  left: 4px;
  bottom: 4px;
  background-color: green;
  transition: 0.4s;
}

.slider[data-checked='true'] {
  background-color: blue;
}

input:focus + .slider {
  box-shadow: 0 0 1px blue;
}

input:checked + .slider:before {
  transform: translateX(26px);
}

/* Rounded sliders */
.slider {
  border-radius: 34px;
}

.slider:before {
  border-radius: 50%;
}
</style>
