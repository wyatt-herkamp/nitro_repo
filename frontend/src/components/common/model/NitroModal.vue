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

</style>
<script lang="ts">
import { defineComponent, onMounted, ref, watch } from "vue";

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
