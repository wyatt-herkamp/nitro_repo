<template>
  <section :id="sectionId">
    <label :for="actualId"><slot></slot></label>
    <select
      :id="actualId"
      v-model="value">
      <option
        v-for="option in props.options"
        :key="option.value"
        :value="option.value">
        {{ option.label }}
      </option>
    </select>
  </section>
</template>

<script setup lang="ts">
import { computed, type PropType } from "vue";
import "@/assets/styles/form.scss";

const props = defineProps({
  options: {
    type: Array as PropType<{ label: string; value: string }[]>,
    required: true,
  },
  id: {
    type: String,
    required: false,
  },
});
const actualId = computed(() => props.id ?? "dropdown");
const sectionId = computed(() => `section-${actualId.value}`);
const value = defineModel<string>({
  required: true,
});
</script>
<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";
section {
  display: flex;
  flex-direction: column;
  margin-bottom: 1rem;
}
</style>
