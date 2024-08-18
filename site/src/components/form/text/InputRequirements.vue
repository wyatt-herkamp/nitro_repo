<template>
  <Transition name="requirements">
    <div class="requirements" :data-show="show" v-if="show">
      <div
        v-for="validation in validations"
        :key="validation.message"
        :data-valid="validation.valid"
        class="requirement"
      >
        <font-awesome-icon icon="fa-solid fa-circle-check" v-if="validation.valid" />
        <font-awesome-icon icon="fa-solid fa-circle-xmark" v-else />
        <span>{{ validation.message }}</span>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, type PropType } from 'vue'
interface Validation {
  message: string
  valid: boolean
}
defineProps({
  show: {
    type: Boolean,
    required: true
  },
  validations: {
    type: Array as PropType<Validation[]>,
    required: true
  }
})
</script>
<style scoped lang="scss">
@import '@/assets/styles/theme.scss';

.requirements-enter-active,
.requirements-leave-active {
  transition: opacity 1s ease-in;
  opacity: 1;
}

.requirements-enter-from,
.requirements-leave-to {
  transition: opacity 1s ease-out;
  opacity: 0;
}
.requirement[data-valid='false'] {
  svg {
    color: $accent-90;
  }
}
.requirement[data-valid='true'] {
  svg {
    color: $valid-value;
  }
}
</style>
