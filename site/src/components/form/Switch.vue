<template>
  <div class="switchBox">
    <label :for="id" class="switchBoxText">
      <slot />
    </label>
    <label class="switch">
      <input v-model="value" v-bind="$attrs" :id="id" type="checkbox" />
      <span class="slider" :data-checked="value ? 'true' : 'false'"></span>
    </label>
  </div>
</template>
<script setup lang="ts">
import { ref, watch } from 'vue'
const props = defineProps<{
  id: string
}>()
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
@import '@/assets/styles/variables';

.switchBox {
  margin: 1rem 2rem;
  display: flex;
  flex-direction: row;
  align-items: center;
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
  background-color: $slider-background;
  transition: 0.4s;
}

.slider:before {
  position: absolute;
  content: '';
  height: 26px;
  width: 26px;
  left: 4px;
  bottom: 4px;
  background-color: $slider-background-before;
  transition: 0.4s;
}

.slider[data-checked='true'] {
  background-color: $slider-background-checked;
}

input:focus + .slider {
  box-shadow: 0 0 1px $slider-background-checked;
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
