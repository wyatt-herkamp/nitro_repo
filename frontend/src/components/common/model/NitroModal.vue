<template>
  <vue-final-modal
    v-model="showModel"
    classes="flex justify-center items-center"
    @click-outside="handleChange()"
  >
    <div class="modal">
      <p class="header"><slot name="header"> </slot></p>
      <button class="xButton" @click="handleChange()">🗙</button>

      <slot name="content"></slot>
    </div>
  </vue-final-modal>
</template>
<style scoped>
.xButton {
  @apply absolute;
  @apply top-0;
  @apply right-0;
  @apply mt-5;
  @apply mr-5;
}
.header {
  @apply font-bold;
  @apply text-xl;
  @apply pb-4;
}
.modal {
  @apply relative;
  @apply border;
  @apply bg-slate-700;
  @apply border-black;
  @apply py-5;
  @apply px-10;
  @apply rounded-2xl;
  @apply shadow-xl;
  @apply text-center;
}
</style>
<script lang="ts">
import { defineComponent, onMounted, ref, toRef, watch } from "vue";

export default defineComponent({
  props: {
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const showModel = ref(false);
    onMounted(() => {
      if (props.modelValue) {
        showModel.value = true;
      }
    });
    watch(
      () => props.modelValue,
      (val) => {
        showModel.value = val;
      }
    );
    const handleChange = (): void => {
      emit("update:modelValue", !showModel.value);
    };
    return {
      showModel,
      handleChange,
    };
  },
  methods: {},
});
</script>