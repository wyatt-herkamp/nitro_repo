<template>
  <div class="px-3">
    <label class="nitroLabel" for="name"> Repository Type </label>
    <select
      id="type"
      v-model="value.repository_type"
      required
      class="nitroSelectBox"
    >
      <option value="Hosted" selected>Hosted</option>
      <option value="Proxy">Proxy</option>
      <option value="Staging">Staging</option>
    </select>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, watch } from "vue";

export default defineComponent({
  name: "MavenCreate",
  props: {
    modelValue: {
      required: true,
      type: Object as () => { repository_type: string },
    },
  },
  setup(props, { emit }) {
    const value = ref({
      repository_type: "Hosted",
    });
    emit("update:modelValue", value.value);
    watch(value.value, (value) => {
      emit("update:modelValue", value);
      console.log(props.modelValue);
    });
    return {
      value,
    };
  },
});
</script>

<style scoped></style>
