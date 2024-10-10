<template>
  <Transition name="requirements">
    <div
      class="requirements"
      :data-show="show"
      v-if="show">
      <div
        v-for="validation in validations"
        :key="validation.message"
        :data-valid="isValid(validation)"
        class="requirement">
        <font-awesome-icon
          icon="fa-solid fa-circle-check"
          v-if="isValid(validation)" />
        <font-awesome-icon
          icon="fa-solid fa-circle-xmark"
          v-else />
        <span>{{ validation.message }}</span>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { type PropType } from "vue";
import type { BaseValidationType } from "./validations";
const props = defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  validations: {
    type: Array as PropType<BaseValidationType[]>,
    required: true,
  },
  results: {
    type: Object as PropType<Record<string, boolean>>,
    required: true,
  },
});
function isValid(validation: BaseValidationType): boolean {
  if (!props.results) {
    return true;
  }
  return props.results[validation.id] ?? false;
}
</script>
<style scoped lang="scss">
@import "@/assets/styles/theme.scss";

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
.requirement[data-valid="false"] {
  svg {
    color: $accent-90;
  }
}
.requirement[data-valid="true"] {
  svg {
    color: $valid-value;
  }
}
</style>
