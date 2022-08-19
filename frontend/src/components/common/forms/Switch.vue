<template>
  <label
    :for="id"
    class="flex items-center h-min m-1 w-fit"
    :class="disabled ? '' : 'cursor-pointer'"
  >
    <div class="relative">
      <input
        ref="input"
        type="checkbox"
        :id="id"
        class="sr-only"
        @change="handleChange()"
      />
      <div class="block bg-gray-600 w-14 h-8 rounded-full"></div>
      <div ref="core" class="switchDot"></div>
    </div>
    <slot></slot>
  </label>
</template>
<style scoped>
.switchDot {
  @apply absolute;
  @apply left-1;
  @apply top-1;
  @apply bg-white;
  @apply w-6;
  @apply h-6;
  @apply rounded-full;
  @apply transition;
}
</style>
<script lang="ts">
import { defineComponent, nextTick, onMounted, ref, watch } from "vue";

export default defineComponent({
  props: {
    modelValue: Boolean,
    id: String,
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  setup(props, { emit }) {
    const input = ref<HTMLInputElement>();
    const core = ref<HTMLDivElement>();
    const checked = ref(props.modelValue);
    watch(checked, () => {
      setBackgroundColor();
    });
    const handleChange = (): void => {
      if (props.disabled) {
        return;
      }
      const value = !props.modelValue;
      checked.value = value;
      emit("update:modelValue", value);
      nextTick(() => {
        input.value!.checked = value;
      });
    };
    onMounted(() => {
      input.value!.checked = checked.value;
      setBackgroundColor();
    });
    const setBackgroundColor = (): void => {
      const dotElement = core.value;
      if (dotElement !== undefined && input.value !== undefined) {
        if (input.value.checked) {
          dotElement.style.backgroundColor = "#48bb78";
          dotElement.style.transform = "translateX(100%)";
        } else {
          dotElement.style.backgroundColor = "";
          dotElement.style.transform = "translateX(0%)";
        }
      } else {
        console.log(dotElement);
      }
    };

    return { handleChange, input, checked, core };
  },
});
</script>
